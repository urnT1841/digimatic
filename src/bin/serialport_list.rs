use serialport;

fn main() {
    // 1. 利用可能なシリアルポートの一覧を取得
    //    available_ports() は Result を返すので、まずは match か expect で捌く必要があります
    match serialport::available_ports() {
        Ok(ports) => {
            if ports.is_empty() {
                println!("利用可能なポートが見つかりませんでした。");
            } else {
                println!("見つかったポート一覧:");
                for p in ports {
                    println!("  - 名前: {}", p.port_name);
                    // 型によって追加情報（USBのメーカー名など）がある場合もある
                    println!("    種類: {:?}", p.port_type);
                }
            }
        }
        Err(e) => {
            eprintln!("ポート取得中にエラーが発生しました: {}", e);
        }
    }
}