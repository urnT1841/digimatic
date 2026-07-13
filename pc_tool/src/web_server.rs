//! `src/web_server.rs`
//! Linux環境において、ブラウザに対してWeb資産を配信し、
//! WebSocket経由でリアルタイムの計測履歴（リングバッファの内容）をブロードキャストするモジュール。

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

use crate::frame::{Measurement, Unit};
use crate::measurement_history::MeasurementHistory;
use crate::presentation::format_with_display_unit;

#[derive(Serialize)]
struct WebPayload {
    current_val: String,
    history: Vec<f64>,
}

// ブラウザのアクセス先
const INDEX_HTML: &str = include_str!("../web/index.html");
const BROWSER_JS: &str = include_str!("../web/browser_gui.js");

pub fn launch_web_server(rx: Receiver<Measurement>) -> Result<(), crate::errors::DigimaticError> {
    // 非同期サーバーを動かすための Tokio 実行環境（Runtime）を生成
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| crate::errors::DigimaticError::Comm(crate::errors::CommError::Io(e)))?;

    rt.block_on(async move {
        // スレッド間で共有する安全な履歴管理（内部のリングバッファをラップ）
        let history = Arc::new(Mutex::new(MeasurementHistory::new()));
        let history_for_ws = Arc::clone(&history);
        // バックグラウンドでMPSCキューを監視し、リングバッファを更新し続ける非同期タスク
        tokio::spawn(async move {
            loop {
                // ブロッキングを避けるため try_recv で引っこ抜く
                while let Ok(new_data) = rx.try_recv() {
                    if let Ok(mut h) = history.lock() {
                        h.add(new_data);
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // 約60FPS
            }
        });

        // ルーターの定義
        let app = Router::new()
            .route("/ws", get(move |ws| ws_handler(ws, history_for_ws)))
            .route("/", get(|| async { axum::response::Html(INDEX_HTML) }))
            .route(
                "/browser_gui.js",
                get(|| async {
                    axum::response::Response::builder()
                        .header("content-type", "application/javascript")
                        .body(axum::body::Body::from(BROWSER_JS))
                        .unwrap()
                }),
            );

        // サーバー起動 (localhost:8080)
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
            .await
            .unwrap();
        println!("\n🚀 Linux Web GUI Server started at http://localhost:8080");
        println!("ブラウザを開いて上記URLにアクセスしてください。");

        axum::serve(listener, app).await.unwrap();
    });

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    history: Arc<Mutex<MeasurementHistory>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, history))
}

async fn handle_socket(mut socket: WebSocket, history: Arc<Mutex<MeasurementHistory>>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50)); // 20Hzで配信

    loop {
        interval.tick().await;

        let payload = {
            let h = history.lock().unwrap();
            if h.is_empty() {
                continue;
            }

            // 最新の値を取り出して文字列整形
            let newest_meas = h.iter_newest().next().unwrap();
            let current_val = format_with_display_unit(newest_meas, Unit::Mm);

            // リングバッファからグラフ/履歴用のf64配列をダンプ
            let history_vec: Vec<f64> = h.iter_newest().map(|m| m.to_f64()).collect();

            WebPayload {
                current_val,
                history: history_vec,
            }
        };

        if let Ok(json) = serde_json::to_string(&payload) {
            // クライアントにテキスト送信（切断されたらループ終了）
            if socket.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    }
}
