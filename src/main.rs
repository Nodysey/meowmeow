mod api;
mod install;

use std::env;

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

    println!("Do you want to continue with package installation? [Y/N]")
}