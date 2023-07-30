mod api;

use std::env;

use colored::Colorize;

#[tokio::main]
async fn main() {
    let args : Vec<String> = env::args().collect();
    
    if args[1] == String::from("search")
    {
        search(args[2].to_owned()).await;
    }
}

/// Function for the "Search" argument
async fn search(pkg_name: String)
{
    let results : Vec<api::SearchResults> = api::search_packages_loose(pkg_name).await;

    for i in results
    {
        println!("{} {}{}{}\n:: {} | {}", ":::".green(), i.repo.red(), "/".green(), i.pkgname.blue(), i.pkgdesc, i.pkgver.yellow());
    } 
}