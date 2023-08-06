use std::io::Write;
use std::fs::{File, self, read};
use std::path::Path;

use colored::Colorize;
use tar::Archive;

use crate::api;
use crate::config;
use crate::database;
use crate::util::bytes_to_readable;

/// Downloads a package from one of the mirrors in /etc/meow.d/mirrorlist
async fn download_pkg(pkg: &database::ArchDesc)
{
    let config = config::get_config();
    let arch = config::get_cpu_arch();
    let mirrors : Vec<String> = config::get_mirrors();
    let download_path = config.general.download_path;
    
    // TODO: Test mirror latency and determine the best one to download from
    let mirror : String = mirrors[1].to_owned();
    
    if !config::validate_mirror(&mirror)
    {
        println!("Mirror {} is not valid!", &mirror);
        return;
    }    

    if Path::new(&format!("{}/{}", &download_path, pkg.file_name)).exists()
    {
        fs::remove_file(format!("{}{}", &download_path, pkg.file_name)).expect("Failed to remove file\nBad privileges?");
    }

    let download_url = format!("{}/{}", 
        mirror.replace("$arch", &arch).replace("$repo", &pkg.repo.to_string()),
        pkg.file_name);

    // Download main archive
    let res = reqwest::get(&download_url).await.expect("WHOOPS!");
    let body = res.bytes().await.unwrap();
    let mut out = File::create(format!("{}{}", download_path, pkg.file_name)).expect("Failed to create file!");
    out.write_all(&body).expect("Failed to write bytes!");

    // TODO: Download archive signature
    
}

/// Installs a pkg & its dependencies.
pub async fn install_pkg(pkg: &database::ArchDesc)
{
    let config = config::get_config();
    let pkg_path = format!("{}/{}", config.general.download_path, &pkg.file_name);
 
    for dependency in &pkg.depends
    {
        let x : database::ArchDesc = database::search_db(dependency).await.unwrap();

        if database::is_pkg_installed(&x).await {continue;}

        let depend_path = format!("{}/{}", config.general.download_path, &x.file_name);
        println!("{} Downloading {}..", "::".green().bold(), &dependency.to_string().blue());
        download_pkg(&x).await;
        install_files(&depend_path);
        // database::add_pkg(&x).await;
    }
    
    println!("{} Downloading {}..", "::".green().bold(), &pkg.name.to_string().blue());
    download_pkg(&pkg).await;
    install_files(&pkg_path);
    // database::add_pkg(&pkg).await;
}

pub async fn upgrade_packages(pkgs: &Vec<database::ArchDesc>)
{
    let config = config::get_config();

    for pkg in pkgs 
    {
        let pkg_path = format!("{}/{}", config.general.download_path, &pkg.file_name);
        println!("{} Upgrading {} to version {}..",
            "::".bold().green(), &pkg.name.blue(), &pkg.version);

        database::remove_pkg(&pkg.name).await;
        download_pkg(&pkg).await;
        install_files(&pkg_path);
        // database::add_pkg(&pkg).await;
    }

    println!("{} {} {}", "Successfully upgraded".bold().green(), &pkgs.len().to_string().bold().green(), "packages!".bold().green());
}

fn install_files(path: &str)
{
    let path_compressed = &path;
    let path_decompressed = &path.replace(".tar.zst", ".tar");

    decompress_zstd(&path_compressed);
    expand_tar(path_decompressed);
}

/// Decompresses the .tar.zst file into a standard tar file for expansion into the main filesystem
fn decompress_zstd(path: &str)
{
    let decompressed_path = &path.replace(".tar.zst", ".tar");
    let mut compressed = read(&path).expect("Failed to read bytes of file.");
    let mut decompressed = File::create(decompressed_path.to_owned()).unwrap();
    let a = zstd::bulk::decompress(&mut compressed, 99999999 as usize).unwrap();
    let mut c : &[u8] = &a;

    println!("==> Decompressing {}", &path.red());
    decompressed.write_all(&mut c).expect("Failed to write bytes.");
}

fn expand_tar(path: &str)
{
    let tar = File::open(&path).unwrap();
    let mut archive = Archive::new(tar);
    println!("==> Extracting {}", path.red());
    archive.unpack("/").unwrap();
    fs::remove_file(&path).expect("Failed to remove old file\nBad previleges?");
}

/// Command for running package installs.
pub async fn install(pkg_name: String)
{
    if nix::unistd::geteuid() != 0.into()
    {
        println!("{}", "Install needs to be ran as root.".red().bold());
        return;
    }

    database::sync().await;
    
    let pkg : database::ArchDesc = database::search_db(&pkg_name).await.unwrap();
    let size_compressed = bytes_to_readable(pkg.csize as f64);
    let size_installed = bytes_to_readable(pkg.size as f64);

    if database::is_pkg_installed(&pkg).await
    {
        let mut reinstall_verification = String::new();
        println!("!! {} - {} is already installed. Reinstall? [Y/N]",
            "WARNING".bold().yellow(), &pkg.name.bold().blue());

        std::io::stdin().read_line(&mut reinstall_verification).unwrap();

        if reinstall_verification.trim().to_lowercase() != "y" && reinstall_verification.trim().to_lowercase() != ""
        {
            return;
        }
    }

    println!("{} {}{}{}", ":::".bold().green(), pkg.repo.red(), "/".green(), pkg.name.blue());
    println!("==> Compressed size: {}\n==> Installed Size: {}", size_compressed.red(), size_installed.red());
    println!("{}", "Depends On:".bold().green());
    
    for d in &pkg.depends
    {
        println!("{} {}", "::".bold().green(), d.blue());
    }


    println!("Do you want to continue with package installation? [Y/N]");
    
    let mut install_verif = String::new();
    std::io::stdin().read_line(&mut install_verif).unwrap(); 
    
    if install_verif.trim().to_lowercase() != "y" && install_verif.trim().to_lowercase() != ""
    {
        println!("{}", "Installation Cancelled".red());
        return;
    }

    // Start installing package + dependencies
    install_pkg(&pkg).await;
}