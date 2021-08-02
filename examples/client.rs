use std::str;
use stun::client::*;
// const REMOTE_ADDRESS: &str = "stun.l.google.com:19302";
const REMOTE_ADDRESS: &str = "142.250.21.127:19302";
// const REMOTE_ADDRESS: &str = "0.0.0.0:3478";

#[tokio::main]
async fn main() {
    let my_global_ip: String = get_global_ip(&REMOTE_ADDRESS.to_string()).await;
    println!("{}", my_global_ip);
}
