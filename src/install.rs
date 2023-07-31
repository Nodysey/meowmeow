use std::io::Write;
use std::fs::{File, self, read};
use std::path::Path;

use colored::Colorize;
use tar::Archive;

#[path="../src/config.rs"]
mod config;

#[path="../src/api.rs"]
mod api;

/// Downloads a package from one of the mirrors in /etc/meow.d/mirrorlist
/// TODO: This needs better error handling at some point.
async fn download_pkg(pkg_name: String)
{
    let mirrors : Vec<String> = config::get_mirrors();
    // TODO: Test mirror latency and determine the best one to download from
    let mirror : String = mirrors[0].to_owned();
    
    if !mirror.contains("$arch") && !mirror.contains("$repo")
    {
        println!("Mirror {} is invalid.\nMake sure all of the mirrors in /etc/meow.d/mirrorlist contain the keys {} and {}",
            mirror, "$arch".yellow().bold(), "$repo".yellow().bold());
        return;
    }
    
    let package = api::search_packages_exact(pkg_name).await;

    if Path::new(&format!("/tmp/meow/{}", package.filename)).exists()
    {
        fs::remove_file(format!("/tmp/meow/{}", package.filename)).expect("Failed to remove file\nBad privileges?");
    }

    let download_url = format!("{}/{}", 
        mirror.replace("$arch", &package.arch.to_string()).replace("$repo", &package.repo.to_string()),
        package.filename);


    let res = reqwest::get(&download_url).await.expect("WHOOPS!");
    let body = res.bytes().await.unwrap();
    let mut out = File::create(format!("/tmp/meow/{}", package.filename)).expect("Failed to create file!");
    out.write_all(&body).expect("Failed to write bytes!");

}

/// Installs a package & its dependencies.
pub async fn install_pkg(pkg_name: String)
{
    let package_details : api::PackageDetails = api::search_packages_exact(pkg_name).await;
    
    for dependency in package_details.depends
    {
        println!("{} Downloading {}..", "::".green().bold(), &dependency.to_string().blue());
        download_pkg(dependency).await;
    }

    // TODO: INSTALL DEPENDENCIES
    println!("{} Downloading {}..", "::".green().bold(), &package_details.pkgname.to_string().blue());
    download_pkg(package_details.pkgname).await;
    
}

// TODO: This should be async.
/// Decompresses the .tar.zst file into a standard tar file for expansion into the main filesystem
fn decompress_zstd(path: String)
{
    let decompressed_path = &path.replace(".tar.zst", ".tar");
    let mut compressed = read(path).expect("Failed to read bytes of file.");
    let mut decompressed = File::create(decompressed_path.to_owned()).unwrap();
    let a = zstd::bulk::decompress(&mut compressed, 99999999 as usize).unwrap();
    let mut c : &[u8] = &a;

    decompressed.write_all(&mut c).expect("Failed to write the bytes lol");
}

fn expand_tar(path: String)
{
    let tar = File::open(path).unwrap();
    let mut archive = Archive::new(tar);
    archive.unpack(".").unwrap();
}