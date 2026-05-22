//!
//! BCD変換お試し
//!
//!

fn main() {
    const EPSIRON: f64 = 1E-5;
    let rx_num: f64 = 123.405;
    println!("受信データ： {}", rx_num);

    // 本来は小数点の位置を求めるとかする必要があるが，ノギスからのデータは小数点2桁固定なので
    // 受信データを下記で整数に変えて，各桁を10の剰余で求めて詰めていく
    // 実際には小数表現の揺らぎを吸収するため 100倍 → 丸め -> 整数へ型変換 を実施
    // 測定データは d6 - d11 の 6つなので 6回まわす
    // 10の剰余(いわゆるMod)は下の桁からの入手になるので，配列の後ろから詰めることに注意 u8にすることも忘れずに
    let mut bcd_frame = [0x0Fu8; 6];
    let mut to_bcd = (rx_num * 100.0 + EPSIRON).round() as u32; // .round() 四捨五入
    //let mut to_bcd = (rx_num * 100.0 + EPSIRON).trunc() as u32; // .trunc() 切り捨て

    // まずは分解するとどうなるかを観察
    println!("変換対象:{} ", to_bcd);
    for i in (0..=5).rev() {
        //.rev() で逆順にする
        bcd_frame[i] = (to_bcd % 10) as u8;
        to_bcd /= 10; //桁ずらす

        print!("{} 回めの数：{}\t", i, to_bcd);
        println!("BCD配列 {:?}", bcd_frame);
    }
}
