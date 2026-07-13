// browser_gui.ts (TypeScript 7環境でサクッと書く)
interface MeasurementPayload {
  current_val: string;
  history: number[]; // Rust側のリングバッファからダンプされた配列
}

const ws = new WebSocket("ws://localhost:8080/ws");
const valueEl = document.getElementById("value");
const historyEl = document.getElementById("history");

ws.onmessage = (event) => {
  const data: MeasurementPayload = JSON.parse(event.data);
  
  // 1. 特大の現在値表示 (eguiのdraw_main_measurement相当)
  if (valueEl) valueEl.innerText = data.current_val;

  // 2. 履歴の更新 (eguiのdraw_main_history相当)
  if (historyEl) {
    historyEl.innerHTML = data.history
      .map((v, i) => `<div>${i === 0 ? '➡️' : '  '} [${i}]: ${v.toFixed(2)} mm</div>`)
      .join("");
  }
};