mod api;

use std::env;

use colored::Colorize;

#[tokio::main]
async fn main() {
    let args : Vec<String> = env::args().collect();
    
    if args[1] == String::from("search")
    {
        let search_results : Vec<api::SearchResults> = api::search_packages_loose(args[2].to_owned()).await;

        for i in search_results
        {
            println!("{} {}\n:: {} | {}", ":::".green() ,i.pkgname.bold().blue(), i.pkgdesc, i.pkgver.yellow());
        }
    }
}
