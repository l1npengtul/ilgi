#![feature(result_flattening)]

mod theme;
mod config;
mod file_ops;
mod sitebuild;
mod db;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
