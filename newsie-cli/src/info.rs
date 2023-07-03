//! Info

use crate::config;

/// Returns the client info
pub fn get_info() {
    let cfg = config::load_or_request();
    println!();
    println!("Client info");
    println!("  endpoint: {}", cfg.url);
    println!("  token: {}", cfg.token.unwrap_or("none".to_string()));
}
