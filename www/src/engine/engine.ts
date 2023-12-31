import { type BoardApi, type Promotion, type SquareKey } from "vue3-chessboard";

export class Engine {
  private worker: Worker;
  private boardApi: BoardApi;

  public bestMove: string | null = null;

  constructor(boardApi: BoardApi) {
    this.boardApi = boardApi;
    this.worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });

    this.setupListeners();
    this.worker.postMessage("uci");
  }

  private setupListeners() {
    this.worker.addEventListener("message", (data) => this.handleEngineStdout(data));
  }

  private handleEngineStdout(data: MessageEvent<unknown>) {
    const uciStringSplitted = (data.data as string).split(" ");

    console.log("<-- " + data.data);

    if (uciStringSplitted[0] === "uciok") {
      this.worker?.postMessage("ucinewgame");
      this.worker?.postMessage("isready");
      return;
    }

    if (uciStringSplitted[0] === "readyok") {
      this.worker?.postMessage("go movetime 1000");
      return;
    }

    if (uciStringSplitted[0] === "bestmove" && uciStringSplitted[1]) {
      this.bestMove = uciStringSplitted[1];
      const orig = this.bestMove.slice(0, 2) as SquareKey;
      const dest = this.bestMove.slice(2, 4) as SquareKey;
      const promotion = (this.bestMove.slice(4, 5) || undefined) as Promotion | undefined;
      if (this.boardApi.getTurnColor() === "black") {
        this.boardApi.move({
          from: orig,
          to: dest,
          promotion: promotion
        });
      }
      // this.boardApi.drawMove(orig, dest, "paleBlue");
    }
  }

  public sendPosition(position: string) {
    this.worker.postMessage(`position startpos moves ${position}`);
    this.worker.postMessage(`go movetime 2000`);
  }
}
