use crate::database;
use crate::api;
use crate::install;
use std::io::stdin;
use colored::Colorize;


/// Checks all of the packages in the database for updates
pub async fn check_for_updates() -> Vec<database::ArchDesc>
{
    let packages: Vec<database::PackageDesc> = database::get_installed_packages();
    let mut upgradable_packages : Vec<database::ArchDesc> = Vec::new();

    for pkg in packages 
    {
        let db_pkg = database::search_db(&pkg.pkgname).await.unwrap();

        if db_pkg.version == pkg.pkgver {continue;}

        upgradable_packages.push(db_pkg);
    }

    return upgradable_packages;
}

pub async fn upgrade_all()
{
    if nix::unistd::geteuid() != 0.into()
    {
        println!("{}", "Upgrade needs to be ran as root.".red().bold());
        return;
    }
    
    database::sync().await;

    let upgradable_packages = check_for_updates().await;

    if upgradable_packages.is_empty()
    {
        println!("All packages are up to date!");
        return;
    }

    println!("{} The following packages are available to upgrade:", "::".bold().green());
    for pkg in &upgradable_packages
    {
        println!("{} {}", "::".green(), &pkg.name);
    }

    println!("Would you like to upgrade all {} packages? [Y/N]", &upgradable_packages.len());

    let mut upgrade_verif = String::new();
    stdin().read_line(&mut upgrade_verif).unwrap();

    if upgrade_verif.trim().to_lowercase() != "y" && upgrade_verif.trim() != "" {return;}

    install::upgrade_packages(&upgradable_packages).await;
}
