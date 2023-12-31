<script setup lang="ts">
import { Engine } from "@/engine/engine";
import { ref } from "vue";

import { BoardApi, TheChessboard, type BoardConfig } from "vue3-chessboard";
import "vue3-chessboard/style.css";

let engine: Engine | undefined;
let boardAPI: BoardApi | undefined;

const pgn = ref<string | undefined>(undefined);
const fen = ref<string | undefined>(undefined);

const boardConfig: BoardConfig = {
  events: {
    select: () => {
      engine?.showEngineMove();
    },
    move: () => {}
  },
  coordinates: true
};

function handleBoardCreated(boardApi: BoardApi) {
  boardAPI = boardApi;
  engine = new Engine(boardApi);
  fen.value = boardAPI.getFen();
  pgn.value = boardAPI.getPgn();
}

function handleMove() {
  pgn.value = boardAPI?.getPgn();
  fen.value = boardAPI?.getFen();

  const history = boardAPI?.getHistory(true);
  const moves = history?.map((move) => {
    if (typeof move === "object") {
      return move.lan;
    } else {
      return move;
    }
  });

  if (moves && !boardAPI?.getIsGameOver()) {
    engine?.sendPosition(moves.join(" "));
  }
}
</script>

<template>
  <div class="layout">
    <h1>My Chess Engine</h1>
    <TheChessboard
      :board-config="boardConfig"
      @board-created="handleBoardCreated"
      @move="handleMove"
      player-color="white"
      class="board"
    />
    <div class="button-group">
      <button @click="engine?.toggleDisplayEngineMove()">Display Engine Move</button>
      <button @click="boardAPI?.viewStart()">Start</button>
      <button @click="boardAPI?.viewPrevious()">Previous Move</button>
      <button @click="boardAPI?.viewNext()">Next Move</button>
      <button @click="boardAPI?.stopViewingHistory()">End</button>
      <a :href="`https://lichess.org/analysis/pgn/${pgn}`"><button>Lichess Analysis</button></a>
    </div>
    <div class="info">
      <p>FEN: {{ fen }}</p>
      <p>PGN: {{ pgn }}</p>
    </div>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.board {
  width: 100%;
  height: 100%;
}
.button-group {
  padding-top: 1em;
  display: flex;
  gap: 0.5em;
}
.info {
  max-width: 32em;
}
</style>
