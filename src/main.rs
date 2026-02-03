//!
//! ;
//! 
//! 
//! 

mod port_prepare;
use port_prepare::port_prepare;

mod sender;
use sender::send_data;

fn main() {

    // ポート準備
    let ports = match port_prepare() {
        Ok(p) => {
            println!("✅ ポート作成成功");
            println!("送信用（source）: {}", p.source);
            println!("受信用（dist）  : {}", p.dist);          
            p
        }
        Err(e) => {
            eprintln!("❌ ポート準備失敗: {}", e);
            std::process::exit(1);
        }
    };

    send_data(99., &ports);


    println!("✅ 送信完了。Enterキーを押すと終了します（ポートを閉じます）...");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();

}