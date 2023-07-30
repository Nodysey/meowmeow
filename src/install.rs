use std::io::{self, Write};
use std::fs::File;
use std::path::Path;

#[path="../src/config.rs"]
mod config;

#[path="../src/api.rs"]
mod api;

/// Downloads a package from one of the mirrors in /etc/meow.d/mirrorlist
pub async fn download_pkg(pkg_name: String)
{
    let mirrors : Vec<String> = config::get_mirrors();
    // TODO: Test mirror latency and determine the best one to download from
    let mirror : String = mirrors[0].to_owned();
    
    if !mirror.contains("$arch") && !mirror.contains("$repo")
    {
        return;
    }
    
    let package = api::search_packages_exact(pkg_name).await;

    // Check to see if the file already exists in /tmp/meow/
    if Path::new(&format!("/tmp/meow/{}", package.filename)).exists()
    {
        println!("File already exists in the temporary directory -- skipping.");
        return;
    }

    let download_url = format!("{}/{}", 
        mirror.replace("$arch", &package.arch.to_string()).replace("$repo", &package.repo.to_string()),
        package.filename);

    dbg!(&download_url);

    let res = reqwest::get(&download_url).await.expect("WHOOPS!");
    let body = res.bytes().await.unwrap();
    let mut out = File::create(format!("/tmp/meow/{}", package.filename)).expect("Failed to create file!");
    out.write_all(&body).expect("Failed to write bytes!");

}