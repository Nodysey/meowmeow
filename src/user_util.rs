use crate::config;
use crate::api;

use colored::Colorize;
use std::fs::OpenOptions;
use std::io::Write;

pub fn add_mirror(mirror: &str)
{

    if !config::validate_mirror(&mirror)
    {
        println!("Invalid mirror format.");
        return;
    }

    let mirrorlist_path = config::get_config().mirrors.mirrorlist;
    let mut file = OpenOptions::new()
        .append(true)
        .open(&mirrorlist_path)
        .unwrap();

    writeln!(file, "{}", &mirror).unwrap();

    println!("Added {} to {}", &mirror, &mirrorlist_path);
}

pub async fn search(pkg_name: String)
{
    let results = api::search_packages_loose(&pkg_name).await
        .expect("Package not found.");

    for i in results
    {
        println!("{} {}{}{}\n:: {} | {}", ":::".green(), i.repo.red(), "/".green(), i.pkgname.blue(), i.pkgdesc, i.pkgver.yellow());
    } 
}