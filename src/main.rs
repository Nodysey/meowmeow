mod api;
mod install;
mod config;

use std::env;
use std::io::stdin;

use colored::Colorize;

#[tokio::main]
async fn main() {
    let args : Vec<String> = env::args().collect();
    
    if args[1] == String::from("search")
    {
        search(args[2].to_owned()).await;
    }

    if args[1] == "install"
    {
        install(args[2].to_owned()).await;
    }

    if args[1] == "test"
    {
        install::download_pkg(String::from("pacman")).await;
    }
 }

/// Function for the "Search" argument
async fn search(pkg_name: String)
{
    let results : Vec<api::SearchResult> = api::search_packages_loose(pkg_name).await;

    for i in results
    {
        println!("{} {}{}{}\n:: {} | {}", ":::".green(), i.repo.red(), "/".green(), i.pkgname.blue(), i.pkgdesc, i.pkgver.yellow());
    } 
}

async fn install(pkg_name: String)
{
    let pkg : api::SearchResult = api::search_packages_exact(pkg_name).await;

    println!("{} {}{}{}", ":::".bold().green(), pkg.repo.red(), "/".green(), pkg.pkgname.blue());
    println!("==> Compressed size: {}\n==> Installed Size: {}", pkg.compressed_size.to_string().red(), pkg.installed_size.to_string().red());
    println!("{}", "Depends On:".bold().green());
    
    for d in pkg.depends
    {
        println!("{} {}", "::".bold().green(), d.blue());
    }

    println!("Do you want to continue with package installation? [Y/N]");
    
    let mut install_verif = String::new();
    stdin().read_line(&mut install_verif).unwrap(); 
    
    if install_verif.trim().to_lowercase() != "y" && install_verif.trim().to_lowercase() != ""
    {
        println!("{}", "Installation Cancelled".red());
        return;
    }

    // Start installing package + dependencies
    
}