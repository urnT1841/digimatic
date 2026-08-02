// web/browser_gui.ts

interface MeasurementPayload {
  current_val: string;
  history: number[];
}

interface Point2D {
  x: number;
  y: number;
}

const ws = new WebSocket("ws://localhost:8080/ws");
const valueEl = document.getElementById("value");
const historyEl = document.getElementById("history");

const canvas = document.getElementById("graph") as HTMLCanvasElement | null;
const ctx = canvas ? canvas.getContext("2d") : null;

ws.onmessage = (event: MessageEvent) => {
  const data: MeasurementPayload = JSON.parse(event.data);

  if (valueEl) valueEl.innerText = data.current_val;

  if (historyEl) {
    historyEl.innerHTML = data.history
      .map((v, i) => `<div>${i === 0 ? '➡️' : '  '} [${i}]: ${v.toFixed(2)} mm</div>`)
      .join("");
  }

  if (canvas && ctx && data.history && data.history.length > 0) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // 安全ガード: 配列の中身があるときだけ計算する
    let Math_min: number = data.history[0];
    let Math_max: number = data.history[0];
    for (let i = 1; i < data.history.length; i++) {
      if (data.history[i] < Math_min) Math_min = data.history[i];
      if (data.history[i] > Math_max) Math_max = data.history[i];
    }

    if (Math_max === Math_min) {
      Math_max += 1.0;
      Math_min -= 1.0;
    } else {
      const margin = (Math_max - Math_min) * 0.15; // 上下余白 (15%)
      Math_max += margin;
      Math_min -= margin;
    }

    const rangeY = Math_max - Math_min;

    // 🌟 左側のパディング分(70px)を引いた、純粋な「グラフ描画の幅」を計算
    const paddingLeft = 70;
    const graphWidth = canvas.width - paddingLeft;

    const maxPoints = 50;
    const stepX = graphWidth / (maxPoints - 1);

    // 🌟 Y軸のグリッド線と数値ラベルの描画（左側の枠外へ配置）
    ctx.font = "12px monospace";
    ctx.fillStyle = "#888888"; // グレー
    ctx.textBaseline = "middle";
    ctx.textAlign = "left"; // 🌟 左揃えに設定

    // 上端（最大値）のラベルを左端（枠外）に配置
    const yMaxPos = 15;
    ctx.fillText(`${Math_max.toFixed(2)} mm`, 10, yMaxPos);

    // 下端（最小値）のラベルを左端（枠外）に配置
    const yMinPos = canvas.height - 15;
    ctx.fillText(`${Math_min.toFixed(2)} mm`, 10, yMinPos);

    // 折れ線グラフ描画
    ctx.beginPath();
    ctx.strokeStyle = "#00FF96"; // eguiの緑
    ctx.lineWidth = 1.4;

    const points: Point2D[] = [];

    for (let i = 0; i < data.history.length; i++) {
      const val = data.history[i];
      // 🌟 X座標の計算の起点を、左パディングの右側（枠の左端）にマッピング
      const x = canvas.width - (i * stepX);
      const y = canvas.height - ((val - Math_min) / rangeY) * canvas.height;

      // 座標を記憶
      points.push({ x, y });

      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }
    ctx.stroke();

    // すべてのプロット点（過去データ含む）
    for (let i = 0; i < points.length; i++) {
      const p = points[i];

      if (i === 0) {
        // 最新値[0]の特大マーカー
        ctx.beginPath();
        ctx.arc(p.x, p.y, 6, 0, 2 * Math.PI);
        ctx.fillStyle = "rgba(0, 255, 150, 0.4)";
        ctx.fill();

        ctx.beginPath();
        ctx.arc(p.x, p.y, 3, 0, 2 * Math.PI);
        ctx.fillStyle = "#ffffff";
        ctx.fill();
      } else {
        // 過去のポイント[1~49]のマーカー
        ctx.beginPath();
        ctx.arc(p.x, p.y, 3, 0, 2 * Math.PI);
        ctx.fillStyle = "#00FF96";
        ctx.fill();
      }
    }
  }
};