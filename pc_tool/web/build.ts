// build.ts
import { transpile } from "jsr:@deno/emit";

const url = new URL("browser_gui.ts", import.meta.url);
// transpile関数を使ってTypeScriptの型注釈を消去
const result = await transpile(url);
const code = result.get(url.href);

if (code) {
  await Deno.writeTextFile("browser_gui.js", code);
  console.log("✅ browser_gui.js の出力に成功しました！");
} else {
  console.error("❌ トランスパイルに失敗しました。");
}