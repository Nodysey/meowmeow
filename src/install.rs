use std::io::Write;
use std::fs::{File, self, read};
use std::path::Path;

use colored::Colorize;
use tar::Archive;

use crate::api;
use crate::config;
use crate::database;

/// Downloads a package from one of the mirrors in /etc/meow.d/mirrorlist
async fn download_pkg(pkg: &api::PackageDetails)
{
    let config = config::get_config();
    let mirrors : Vec<String> = config::get_mirrors();
    let download_path = config.general.download_path;
    
    // TODO: Test mirror latency and determine the best one to download from
    let mirror : String = mirrors[1].to_owned();
    
    if !mirror.contains("$arch") && !mirror.contains("$repo")
    {
        println!("Mirror {} is invalid.\nMake sure all of the mirrors in /etc/meow.d/mirrorlist contain the keys {} and {}",
            mirror, "$arch".yellow().bold(), "$repo".yellow().bold());
        return;
    }


    if Path::new(&format!("{}{}", &download_path, pkg.filename)).exists()
    {
        fs::remove_file(format!("{}{}", &download_path, pkg.filename)).expect("Failed to remove file\nBad privileges?");
    }

    let download_url = format!("{}/{}", 
        mirror.replace("$arch", &config.general.arch).replace("$repo", &pkg.repo.to_string()),
        pkg.filename);

    // Download main archive
    let res = reqwest::get(&download_url).await.expect("WHOOPS!");
    let body = res.bytes().await.unwrap();
    let mut out = File::create(format!("{}{}", download_path, pkg.filename)).expect("Failed to create file!");
    out.write_all(&body).expect("Failed to write bytes!");

    // TODO: Download archive signature
    
}

/// Installs a pkg & its dependencies.
pub async fn install_pkg(pkg_name: String)
{
    let config = config::get_config();
    let package_details : api::PackageDetails = api::search_packages_exact(&pkg_name).await;
    let pkg_path = format!("{}{}", config.general.download_path, &package_details.filename);
 
    for dependency in &package_details.depends
    {
        let x : api::PackageDetails = api::search_packages_exact(&dependency).await;
        let depend_path = format!("{}{}", config.general.download_path, &x.filename);
        println!("{} Downloading {}..", "::".green().bold(), &dependency.to_string().blue());
        download_pkg(&x).await;
        install_files(&depend_path);
        database::add_pkg_to_database(&x).await;
    }
    
    println!("{} Downloading {}..", "::".green().bold(), &package_details.pkgname.to_string().blue());
    download_pkg(&package_details).await;
    install_files(&pkg_path);
    database::add_pkg_to_database(&package_details).await;
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