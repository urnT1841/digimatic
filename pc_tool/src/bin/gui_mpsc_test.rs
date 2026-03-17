use std::sync::mpsc::Receiver;
use std::time::Duration;
use rand::Rng;
use eframe::egui;

struct MyAppData {
    dice_value: u32,
    receiver: Receiver<u32>, // 受信機を格納
}

impl MyAppData {
    // 起動時に受信機を受け取るための専用の初期化関数
    fn with_receiver(rx: Receiver<u32>) -> Self {
        Self {
            dice_value: 0,
            receiver: rx,
        }
    }
}
impl eframe::App for MyAppData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- データ受信チェック ---
        // try_recv() は「データがあれば取る、なければすぐ次へ」という動き
        // while を使うことで、溜まっているデータをすべて処理する
        while let Ok(new_val) = self.receiver.try_recv() {
            self.dice_value = new_val;
            // データが更新されたので、次のフレームを待たずに再描画を予約する
            // ctx.request_repaint();
        }

        // --- 描画エリア ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("自動計測シミュレーション中").size(20.0));

                let val_mm = self.dice_value as f64 / 100.0;
                ui.label(
                    egui::RichText::new(format!("{:.2} mm", val_mm))
                        .size(120.0)
                        .strong(),
                );
            });
        });
        // 次の update を1秒後に予約
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

fn main() -> eframe::Result {
    let (tx, rx) = std::sync::mpsc::channel::<u32>();

    // スレッドを起動  move をつけることで、tx（送信機）の所有権をスレッドの中に移します
    std::thread::spawn(move || {
        loop {
            // 自作の「インチキサイコロ」を振る
            let val = big_num_dice();

            // GUI側に送信。もしGUIが閉じられたらエラーになるのでループを抜ける
            if tx.send(val).is_err() {
                break;
            }

            // 700ms 待機（Simの WATI_TIME_MS に合わせるイメージ）
            std::thread::sleep(std::time::Duration::from_millis(1200));
        }
    });

    // 3. アプリ起動
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Digimatic Sim",
        options,
        // 受信機（rx）を構造体に託す
        Box::new(|_cc| Ok(Box::new(MyAppData::with_receiver(rx)))),
    )
}
fn big_num_dice() -> u32 {
    let mut rng = rand::rng();
    let dice = rng.random_range(1..=150_00);
    dice
}
