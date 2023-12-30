<script setup lang="ts">
import { Engine } from "@/engine/engine";
import { ref } from "vue";

import { BoardApi, TheChessboard, type BoardConfig, type SquareKey } from "vue3-chessboard";
import "vue3-chessboard/style.css";

let engine: Engine | undefined;
let boardAPI: BoardApi | undefined;

let pgn = ref<string | undefined>(undefined);

const boardConfig: BoardConfig = {
  events: {
    select: () => {
      // if (engine?.bestMove) {
      //   boardAPI?.drawMove(
      //     engine.bestMove.slice(0, 2) as SquareKey,
      //     engine.bestMove.slice(2, 4) as SquareKey,
      //     "paleBlue"
      //   );
      // }
    },
    move: () => {
      // boardAPI?.hideMoves();
    }
  },
  coordinates: true
};

function handleBoardCreated(boardApi: BoardApi) {
  boardAPI = boardApi;
  engine = new Engine(boardApi);
}

function handleMove() {
  pgn.value = boardAPI?.getPgn();
  const history = boardAPI?.getHistory(true);
  const moves = history?.map((move) => {
    if (typeof move === "object") {
      return move.lan;
    } else {
      return move;
    }
  });

  if (moves) {
    engine?.sendPosition(moves.join(" "));
  }
}
</script>

<template>
  <TheChessboard
    :board-config="boardConfig"
    @board-created="handleBoardCreated"
    @move="handleMove"
    player-color="white"
  />

  {{ pgn }}
</template>
