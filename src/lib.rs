 //!
 //! 
 //! 
 //! 
 //! 
 // src/lib.rs
pub mod caliper_sim {
    use std::sync::mpsc::Sender;
    use std::time::Duration;
    use std::thread;

    // データを生成してSenderに送る「脳みそ」
    pub fn run_simulation(tx: Sender<String>) {
        let mut val = 0.0;
        loop {
            let data = format!("{:.2}\n", val);
            if tx.send(data).is_err() { break; }
            val += 0.01;
            thread::sleep(Duration::from_millis(100));
        }
    }
}