mod api;
mod install;
mod config;

use std::env;
use std::cmp;
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
        // install::install_pkg(String::from("neofetch")).await;
        // install::decompress_zstd("/tmp/meow/bash-5.1.016-4-x86_64.pkg.tar.zst".to_string()).await;
        println!("{}", bytes_to_readable(2312332313_f64));
    }
 }

/// Function for the "Search" argument
async fn search(pkg_name: String)
{
    let results : Vec<api::PackageDetails> = api::search_packages_loose(pkg_name).await;

    for i in results
    {
        println!("{} {}{}{}\n:: {} | {}", ":::".green(), i.repo.red(), "/".green(), i.pkgname.blue(), i.pkgdesc, i.pkgver.yellow());
    } 
}

async fn install(pkg_name: String)
{
    let pkg : api::PackageDetails = api::search_packages_exact(pkg_name).await;
    let size_compressed = bytes_to_readable(pkg.compressed_size as f64);
    let size_installed = bytes_to_readable(pkg.installed_size as f64);

    println!("{} {}{}{}", ":::".bold().green(), pkg.repo.red(), "/".green(), pkg.pkgname.blue());
    println!("==> Compressed size: {}\n==> Installed Size: {}", size_compressed.red(), size_installed.red());
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


// Borrowed from rust-pretty-bytes
pub fn bytes_to_readable(bytes: f64) -> String
{
    let neg = if bytes.is_sign_positive() {""} else {"-"};
    let bytes = bytes.abs();
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    if bytes < 1_f64 {
        return format!("{}{} {}", neg, bytes, "B");
    }

    let delimiter = 1000_f64;
    let exponent = cmp::min((bytes.ln() / delimiter.ln()).floor() as i32, (units.len() - 1) as i32);
    let readable = format!("{:.2}", bytes / delimiter.powi(exponent)).parse::<f64>().unwrap() * 1_f64;

    let unit = units[exponent as usize];
    return format!("{}{} {}", neg, readable, unit);
}