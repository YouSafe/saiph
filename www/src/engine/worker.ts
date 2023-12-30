import init, { Engine } from "../../pkg";

let engine: Engine | undefined = undefined;

self.onmessage = async function (event) {
  if (!engine) {
    await initEngine();
  }
  engine?.receive_command(event.data);
  console.log("--> " + event.data);
};

async function initEngine() {
  await init();
  engine = Engine.new();
}
