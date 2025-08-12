use crate::board::Board;
use crate::clock::Clock;
use crate::evaluation::Evaluation;
use crate::evaluation::hce::board_value;
use crate::movegen::MoveList;
use crate::moveord::mmv_lva;
use crate::pv_table::PrincipleVariationTable;
use crate::threadpool::StopSync;
use crate::transposition::{Entry, TranspositionTable, ValueType};
use crate::types::chess_move::Move;
use crate::types::color::Color;
use crate::types::search_limits::SearchLimits;
use crate::uci::EngineMessage;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use web_time::Instant;

pub struct NodeCountBuffer {
    inner: Vec<AtomicU64>,
}

impl NodeCountBuffer {
    pub fn new(num_threads: u8) -> Self {
        Self {
            inner: vec![0u64; num_threads as usize]
                .into_iter()
                .map(AtomicU64::new)
                .collect(),
        }
    }

    pub fn get(&self, thread_id: u8) -> &AtomicU64 {
        &self.inner[thread_id as usize]
    }

    pub fn accumulate(&self) -> u64 {
        self.inner.iter().map(|v| v.load(Ordering::Relaxed)).sum()
    }
}

pub struct Search {
    board: Board,
    limits: SearchLimits,
    pv_table: PrincipleVariationTable,
    local_stop: bool,
    clock: Clock,
    root_moves: MoveList,

    engine_tx: Sender<EngineMessage>,
    tt: Arc<TranspositionTable>,
    stop_sync: Arc<StopSync>,

    thread_id: u8,
    nodes_buffer: Arc<NodeCountBuffer>,
    call_cnt: i16,
}

impl Search {
    pub fn new(
        board: Board,
        limits: SearchLimits,
        clock: Clock,
        root_moves: MoveList,
        engine_tx: Sender<EngineMessage>,
        tt: Arc<TranspositionTable>,
        stop_sync: Arc<StopSync>,
        thread_id: u8,
        nodes_buffer: Arc<NodeCountBuffer>,
    ) -> Self {
        Search {
            board,
            limits,
            pv_table: PrincipleVariationTable::new(),
            local_stop: false,
            clock,
            root_moves,

            engine_tx,
            tt,
            stop_sync,
            thread_id,
            nodes_buffer,
            call_cnt: 0,
        }
    }

    pub fn search(mut self, is_main: bool) -> Move {
        self.iterative_deepening(is_main);

        let _guard = self.stop_sync.cond_var.wait_while(
            self.stop_sync.wait_for_stop.lock().unwrap(),
            |wait_for_stop| *wait_for_stop,
        );

        let best_move = self.pv_table.best_move();

        if is_main {
            self.engine_tx
                .send(EngineMessage::Response(format!("bestmove {best_move}")))
                .unwrap();
        }

        best_move
    }

    fn iterative_deepening(&mut self, is_main: bool) {
        let mut evaluation;

        for depth in 1..u8::MAX {
            evaluation =
                self.negamax_search::<true, true>(Evaluation::MIN, Evaluation::MAX, depth, 0);

            if self.local_stop {
                break;
            }

            if is_main {
                let output = self.info_string(evaluation, depth).unwrap();

                self.engine_tx
                    .send(EngineMessage::Response(output))
                    .unwrap();
            }

            if depth >= self.limits.depth.unwrap_or(u8::MAX) {
                break;
            }

            if let Some(optimum) = self.clock.optimum {
                if optimum < Instant::now() {
                    break;
                }
            }
        }
    }

    /// Fail soft variant of negamax search
    fn negamax_search<const PV: bool, const ROOT: bool>(
        &mut self,
        mut alpha: Evaluation,
        mut beta: Evaluation,
        mut depth: u8,
        ply: u8,
    ) -> Evaluation {
        self.pv_table.clear(ply as usize);

        if self.should_interrupt() {
            return Evaluation::INVALID;
        }

        if !ROOT {
            if self.board.is_repetition() || self.board.is_draw_by_fifty_move_rule() {
                return Evaluation::EQUALITY;
            }

            alpha = alpha.max(Evaluation::mated_in(ply));
            beta = beta.min(Evaluation::mate_in(ply + 1));

            if alpha >= beta {
                return alpha;
            }

            if !self.board.checkers().is_empty() {
                depth += 1;
            }
        }

        if depth == 0 {
            return self.quiescence(alpha, beta, ply);
        }

        self.nodes_buffer
            .get(self.thread_id)
            .fetch_add(1, Ordering::Relaxed);

        let entry = self.tt.probe(&self.board, ply);
        if let Some(entry) = &entry {
            if !PV && entry.depth >= depth && tt_cutoff(entry, alpha, beta) {
                return entry.value;
            }
        }

        let mut moves = self.board.generate_moves();
        if moves.is_empty() {
            if !self.board.checkers().is_empty() {
                return Evaluation::mated_in(ply);
            } else {
                return Evaluation::EQUALITY;
            }
        }

        let original_alpha = alpha;
        let mut best_score = Evaluation::MIN;
        let mut best_move = Move::NULL;

        moves.sort_by_key(|mov| {
            if let Some(entry) = &entry {
                if &entry.best_move == mov {
                    return -200000;
                }
            }

            let src_piece = self.board.piece_at(mov.from()).unwrap();
            let dst_piece = self.board.piece_at(mov.to());
            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece.ty(), dst_piece.ty());
            }
            0
        });

        for chess_move in moves {
            if ROOT && !self.root_moves.contains(&chess_move) {
                continue;
            }

            self.board.apply_move(chess_move);
            let score = -self.negamax_search::<PV, false>(-beta, -alpha, depth - 1, ply + 1);
            self.board.undo_move();

            if self.local_stop {
                return Evaluation::INVALID;
            }

            if score > best_score {
                best_score = score;
                best_move = chess_move;

                if score > alpha {
                    alpha = score;

                    self.pv_table.update(ply as usize, chess_move);
                }
            }

            if alpha >= beta {
                break;
            }
        }

        let value_type = get_value_type(best_score, original_alpha, beta);

        self.tt
            .store(&self.board, best_move, depth, best_score, value_type, ply);

        best_score
    }

    fn quiescence(&mut self, mut alpha: Evaluation, beta: Evaluation, ply: u8) -> Evaluation {
        self.nodes_buffer
            .get(self.thread_id)
            .fetch_add(1, Ordering::Relaxed);

        let mut moves = self.board.generate_moves();
        if moves.is_empty() {
            if !self.board.checkers().is_empty() {
                return Evaluation::mated_in(ply);
            } else {
                return Evaluation::EQUALITY;
            }
        }

        let evaluation = match self.board.side_to_move() {
            Color::White => board_value(&self.board),
            Color::Black => -board_value(&self.board),
        };

        alpha = alpha.max(evaluation);

        if alpha >= beta {
            return evaluation;
        }

        moves.retain(|m| m.is_capture());

        moves.sort_by_key(|mov| {
            let src_piece = self.board.piece_at(mov.from()).unwrap();
            let dst_piece = self.board.piece_at(mov.to());
            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece.ty(), dst_piece.ty());
            }
            0
        });

        let mut best_score = evaluation;
        for chess_move in moves {
            self.board.apply_move(chess_move);
            let score = -self.quiescence(-beta, -alpha, ply + 1);
            self.board.undo_move();

            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;
                }
            }

            if alpha >= beta {
                break;
            }
        }
        best_score
    }

    fn should_interrupt(&mut self) -> bool {
        self.call_cnt -= 1;
        if self.call_cnt > 0 {
            return self.local_stop;
        }

        self.call_cnt = 512;

        if let Some(maximum) = self.clock.maximum {
            if maximum < Instant::now() {
                self.local_stop = true;
            }
        } else if self.stop_sync.stop.load(Ordering::Relaxed) {
            self.local_stop = true;
        }

        let nodes = self.nodes_buffer.accumulate();

        if let Some(max_nodes) = self.limits.nodes {
            if nodes >= max_nodes {
                self.local_stop = true;
                return true;
            }
        }

        self.local_stop
    }

    fn info_string(
        &mut self,
        evaluation: Evaluation,
        depth: u8,
    ) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;

        let mut output = String::with_capacity(120);

        write!(output, "info depth {} score ", depth)?;
        if evaluation.is_mate() {
            write!(output, "mate {}", evaluation.mate_full_moves())?;
        } else {
            write!(output, "cp {}", evaluation)?;
        }
        write!(output, " time {}", self.clock.start.elapsed().as_millis())?;
        write!(output, " nodes {} pv", self.nodes_buffer.accumulate())?;
        for mov in self.pv_table.variation() {
            write!(output, " {}", mov)?;
        }

        Ok(output)
    }

    pub fn limits(&self) -> &SearchLimits {
        &self.limits
    }
}

fn get_value_type(score: Evaluation, alpha: Evaluation, beta: Evaluation) -> ValueType {
    if score <= alpha {
        ValueType::Upperbound
    } else if score >= beta {
        ValueType::Lowerbound
    } else {
        ValueType::Exact
    }
}

fn tt_cutoff(entry: &Entry, alpha: Evaluation, beta: Evaluation) -> bool {
    match entry.value_type {
        ValueType::Exact => true,
        ValueType::Lowerbound => entry.value >= beta,
        ValueType::Upperbound => entry.value <= alpha,
    }
}
