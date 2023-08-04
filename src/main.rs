mod api;
mod install;
mod remove;
mod upgrade;
mod config;
mod database;
mod cache;
mod user_util;
mod util;

use std::env;

use colored::Colorize;


#[tokio::main]
async fn main() {
    // TODO: Make an arc for the program config

    let args : Vec<String> = env::args().collect();
    

    if args.len() == 1
    {
        // TODO: Print help
        println!("meow v0.1-DEV");
        println!("(Run this with -h for help)");
        return;
    }

    if args[1] == String::from("search")
    {
        search(args[2].to_owned()).await;
    }

    if args[1] == "install"
    {
        install::install(args[2].to_owned()).await;
    }

    if args[1] == "installed-list" || args[1] == "packages"
    {
        for i in database::get_all_packages()
        {
            println!("{}", i);
        }
    }

    if args[1] == "upgrade" || args[1] == "upgrade-all"
    {
        upgrade::upgrade_all().await;
    }

    if args[1] == "test" {
        let mirror = &args[2];
        user_util::add_mirror(mirror)
    }   
 }

/// Function for the "Search" argument
async fn search(pkg_name: String)
{
    let results = api::search_packages_loose(&pkg_name).await
        .expect("Package not found.");

    for i in results
    {
        println!("{} {}{}{}\n:: {} | {}", ":::".green(), i.repo.red(), "/".green(), i.pkgname.blue(), i.pkgdesc, i.pkgver.yellow());
    } 
}
