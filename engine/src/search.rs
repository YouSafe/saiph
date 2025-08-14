use crate::board::Board;
use crate::clock::Clock;
use crate::evaluation::Evaluation;
use crate::evaluation::hce::board_value;
use crate::moveord::mmv_lva;
use crate::pv::PrincipleVariation;
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

pub struct Search {
    board: Board,
    limits: SearchLimits,
    local_stop: bool,
    clock: Clock,
    root_moves: Vec<RootMove>,
    multipv: u8,

    engine_tx: Sender<EngineMessage>,
    tt: Arc<TranspositionTable>,
    stop_sync: Arc<StopSync>,

    pv_index: usize,
    pv_last: usize,

    thread_id: u8,
    nodes_buffer: Arc<NodeCountBuffer>,
    call_cnt: i16,
    completed_depth: u8,
}

impl Search {
    pub fn new(
        board: Board,
        limits: SearchLimits,
        clock: Clock,
        multipv: u8,
        root_moves: Vec<RootMove>,
        engine_tx: Sender<EngineMessage>,
        tt: Arc<TranspositionTable>,
        stop_sync: Arc<StopSync>,
        thread_id: u8,
        nodes_buffer: Arc<NodeCountBuffer>,
    ) -> Self {
        Search {
            board,
            limits,
            local_stop: false,
            clock,
            root_moves,
            multipv,

            pv_index: 0,
            pv_last: 0,

            engine_tx,
            tt,
            stop_sync,
            thread_id,
            nodes_buffer,
            call_cnt: 0,
            completed_depth: 0,
        }
    }

    pub fn search(mut self, is_main: bool) -> Move {
        self.iterative_deepening(is_main);

        let _guard = self.stop_sync.cond_var.wait_while(
            self.stop_sync.wait_for_stop.lock().unwrap(),
            |wait_for_stop| *wait_for_stop,
        );

        let best_move = self.root_moves[0].pv.best_move();

        if is_main {
            self.engine_tx
                .send(EngineMessage::Response(format!("bestmove {best_move}")))
                .unwrap();
        }

        best_move
    }

    fn iterative_deepening(&mut self, is_main: bool) {
        for depth in 1..u8::MAX {
            for pv_index in 0..self.multipv {
                self.pv_index = pv_index as usize;
                self.pv_last = self.root_moves.len() - 1;

                let mut pv = PrincipleVariation::default();

                self.negamax_search::<true, true>(
                    Evaluation::MIN,
                    Evaluation::MAX,
                    depth,
                    0,
                    &mut pv,
                );

                self.root_moves[(self.pv_index)..=(self.pv_last)].sort();

                if self.local_stop {
                    break;
                }

                if is_main {
                    let output = self
                        .info_string(depth, pv_index, &self.root_moves[pv_index as usize])
                        .unwrap();

                    self.engine_tx
                        .send(EngineMessage::Response(output))
                        .unwrap();
                }
            }

            if self.local_stop {
                break;
            }

            if depth >= self.limits.depth.unwrap_or(u8::MAX) {
                break;
            }

            if let Some(optimum) = self.clock.optimum {
                if optimum < Instant::now() {
                    break;
                }
            }

            self.completed_depth += 1;
        }
    }

    /// Fail soft variant of negamax search
    fn negamax_search<const PV: bool, const ROOT: bool>(
        &mut self,
        mut alpha: Evaluation,
        mut beta: Evaluation,
        mut depth: u8,
        ply: u8,
        pv: &mut PrincipleVariation,
    ) -> Evaluation {
        let mut child_pv = PrincipleVariation::default();

        if PV {
            pv.clear();
        }

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

        let mut move_count = 0;
        for chess_move in moves {
            if ROOT && !self.root_moves[self.pv_index..=self.pv_last].includes_root(chess_move) {
                continue;
            }

            move_count += 1;

            self.board.apply_move(chess_move);
            let score =
                -self.negamax_search::<PV, false>(-beta, -alpha, depth - 1, ply + 1, &mut child_pv);
            self.board.undo_move();

            if self.local_stop {
                return Evaluation::INVALID;
            }

            if ROOT {
                let root_move = self.root_moves.find_root_mut(chess_move).unwrap();

                if move_count == 1 || score > alpha {
                    root_move.score = score;
                    root_move.pv.load_from(chess_move, &child_pv);
                } else {
                    root_move.score = Evaluation::MIN;
                    root_move.pv.truncate_to_root();
                }
            }

            if score > best_score {
                best_score = score;
                best_move = chess_move;

                if score > alpha {
                    alpha = score;

                    if !ROOT && PV {
                        pv.load_from(chess_move, &child_pv);
                    }
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
        if self.completed_depth == 0 {
            return false;
        }

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
        &self,
        depth: u8,
        pv_index: u8,
        root_move: &RootMove,
    ) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;

        let mut output = String::with_capacity(120);

        let pv = &root_move.pv;
        let evaluation = root_move.score;

        write!(output, "info depth {depth} multipv {pv_index} score ")?;
        if evaluation.is_mate() {
            write!(output, "mate {}", evaluation.mate_full_moves())?;
        } else {
            write!(output, "cp {evaluation}")?;
        }
        write!(output, " time {}", self.clock.start.elapsed().as_millis())?;
        write!(output, " nodes {} pv", self.nodes_buffer.accumulate())?;
        for mov in pv.line() {
            write!(output, " {mov}")?;
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

#[derive(Debug, Clone)]
pub struct RootMove {
    pub score: Evaluation,
    pub pv: PrincipleVariation,
}

impl PartialEq for RootMove {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for RootMove {}

impl PartialOrd for RootMove {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RootMove {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score).reverse()
    }
}

trait RootMovesExt {
    fn includes_root(&self, mv: Move) -> bool;
    fn find_root_mut(&mut self, mv: Move) -> Option<&mut RootMove>;
}

impl RootMovesExt for [RootMove] {
    fn includes_root(&self, mv: Move) -> bool {
        self.iter().any(|r| r.pv.line().starts_with(&[mv]))
    }

    fn find_root_mut(&mut self, mv: Move) -> Option<&mut RootMove> {
        self.iter_mut().find(|r| r.pv.line().starts_with(&[mv]))
    }
}

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
