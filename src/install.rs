use futures::future::join_all;
use tokio::task::JoinHandle;
use std::fs::{File, read};
use std::io::{Write, BufReader};
use flate2::bufread;
use mtree::MTree;


use crate::database::{ArchDesc, search_db, add_pkg, is_pkg_installed};
use crate::config;


/// Asynchronously downloads files from a list of arch linux package descriptions.
pub async fn download_packages(pkgs: Vec<ArchDesc>)
{   
    let mut tasks : Vec<JoinHandle<Result<(), ()>>> = vec![];
    let cfg = config::get_config();
    let mirror_list = config::get_mirrors();
    let mirror = &mirror_list[0];

    for pkg in pkgs {
        let download_url = format!("{}/{}", mirror.replace("$repo", &pkg.repo).replace("$arch", &cfg.general.arch), &pkg.file_name);
        let download_path = format!("{}/{}", &cfg.general.download_path, &pkg.file_name);
        dbg!(&download_url);
        tasks.push(tokio::spawn(async move {
            match reqwest::get(download_url).await
            {
                Ok(resp) => {
                    println!(":: Downloading {}", &pkg.name);
                    match resp.bytes().await
                    {
                        Ok(b) => {
                            let mut out = File::create(&download_path).unwrap();
                            out.write_all(&b).expect("Failed to write bytes! Bad permissions?");
                        }

                        Err(_) => println!("!! Failed to get bytes for package {}", &pkg.name)
                    }
                }

                Err(_) => println!("!! Failed to download {}", &pkg.name)
            }

            Ok(())
        }));
    }
    join_all(tasks).await;
}

/// Synchronously goes through a package tarball and processes it for installation.
/// Returns a String vector that contains the list of files processed from the package's
/// mtree file.
fn process_tarball(pkg_path: &str) -> Vec<String>
{
    // Decompress
    let pkg_decompressed_path = pkg_path.replace(".tar.zst", ".tar");
    let mut compressed = read(&pkg_path).expect("Failed to read bytes of compressed file.");
    let mut decompressed = File::create(&pkg_decompressed_path).unwrap();
    let z = zstd::bulk::decompress(&mut compressed, 1000000000 as usize).unwrap();
    let mut c : &[u8] = &z;

    println!("==> Decompressing {}", pkg_path);
    decompressed.write_all(&mut c).expect("Failed to write bytes of decompressed file.");

    // Expand tarball
    let tarball = File::open(&pkg_decompressed_path).unwrap();
    let mut archive = tar::Archive::new(tarball);
    println!("==> Extracting {}", &pkg_decompressed_path);
    archive.unpack("/").expect("Failed to decompress tarball -- Bad permissions?");
    std::fs::remove_file(&pkg_decompressed_path).unwrap();

    // Process mtree
    let mtree_compressed = BufReader::new(File::open("/.MTREE").unwrap());
    let decoder = bufread::GzDecoder::new(mtree_compressed);
    let entries = MTree::from_reader(decoder);

    let mut files : Vec<String> = vec![];
    for entry in entries
    {
        let entry = entry.unwrap();
        
        if entry.file_type().unwrap() != mtree::FileType::Directory {continue;}

        let p = entry.path().as_os_str().to_str().unwrap().to_string();

        files.push(p);
    }

    /*
    These lines keep having issues with not being able to find the
    files for some reason. Shouldn't matter since the whole point 
    is to get rid of them, anyways.

    TODO: Make these ignore not being able to find their respective files.
     */
    // std::fs::remove_file("/.MTREE").unwrap();
    // std::fs::remove_file("/.INSTALL").unwrap();
    // std::fs::remove_file("/.PKGINFO").unwrap();
    // std::fs::remove_file("/.BUILDINFO").unwrap();


    return files;
}

/// Install a package & its dependencies.
pub async fn install_package(pkg: ArchDesc, reinstall_dependencies: bool)
{
    // TODO: Make this take a vector of ArchDesc parameters
    // for multiple package installs.
    let cfg = config::get_config();
    let mut pkgs_to_install : Vec<ArchDesc> = vec![];

    
    for dependency in &pkg.depends
    {
        let dependency_desc = search_db(&dependency).await.unwrap();

        if is_pkg_installed(&dependency_desc).await && !reinstall_dependencies {continue;} 

        pkgs_to_install.push(dependency_desc);
    }

    pkgs_to_install.push(pkg);
    let pkgs = &pkgs_to_install.clone();
    
    download_packages(pkgs_to_install).await;

    for pkg in pkgs
    {
        let dir = format!("{}/{}", &cfg.general.download_path, pkg.file_name);

        let tarball_processed = process_tarball(&dir);
        add_pkg(pkg, tarball_processed).await;
    }

}