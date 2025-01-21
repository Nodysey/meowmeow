// mod api;
mod install;
mod remove;
mod upgrade;
mod config;
mod database;
mod cache;
mod user_util;
mod util;
mod operations;

use std::env;

#[tokio::main]
async fn main() {
    // TODO: Make an arc for the program config

    let args : Vec<String> = env::args().collect();
    
    if args.len() == 1
    {
        // TODO: Print help
        println!("meow v0.1-DEV");
        println!("mrrp – Install a Package");
        println!("");
        println!("hsss – Remove a Package");
        println!("");
        println!("upgrade – Upgrade Package(s)");
        println!("");
        println!("sniff – Search for a Package");
        println!("");
        println!("sync – i forget");
        return;
    }

    let operation : &str = &args[1].as_str();


    match operation
    {
        "sniff" => operations::search(args).await,
        "mrrp"=> operations::install(args).await,
        "sync" => operations::sync_databases().await,
        _=>println!("Invalid operation.\nRun 'meow -h' for help.")
    }

 }
