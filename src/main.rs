#![feature(result_flattening)]

mod theme;
mod config;
mod optimize;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
