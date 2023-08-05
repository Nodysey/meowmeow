use toml;
use serde_derive::{Serialize, Deserialize};
use std::fs::{File, create_dir};
use std::io::Write;
use std::path::Path;
use reqwest;
use colored::Colorize;

use crate::config;
use crate::api;

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledPackage
{
    pub desc : PackageDesc,
    pub files : Vec<String>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PackageDesc
{
    pub pkgname : String,
    pub pkgbase : String,
    pub pkgver : String,
    pub pkgdesc : String,
    pub url : String,
    pub build_date : String,
    pub packager: String,
    pub size : i64,
    pub arch : String,
    pub licenses : Vec<String>,
    pub dependencies : Vec<String>,
    pub dependencies_optional : Vec<String>
}

/// Specifically for handling arch desc files.
/// Primarily for use with the arch .db files.
pub struct ArchDesc
{
    pub file_name : String,
    pub name : String,
    pub base : String,
    pub version : String,
    pub desc : String,
    pub csize : i32,    // Compressed size
    pub size : i32,     // Installed size
    pub md5s : String,
    pub sha256 : String,
    pub pgpsig : String,
    pub url : String,
    pub license : String,
    pub arch : String,
    pub build_date: i64,
    pub packager : String,
    pub depends : Vec<String>,
    pub opt_depends : Vec<String>
}

pub async fn add_pkg(pkg: &api::PackageDetails)
{
    let config = config::get_config();
    let file_list = api::get_package_files(&pkg).await;
    let dir_path : String = format!("{}{}-{}-{}", config.general.db_path, &pkg.pkgname, &pkg.pkgver, &pkg.pkgrel);

    if Path::exists(&Path::new(&dir_path))
    {
        // TODO:
        return;
    }

    let pkgdesc = PackageDesc {
        pkgname: pkg.pkgname.to_owned(),
        pkgbase: pkg.pkgbase.to_owned(),
        pkgver: format!("{}-{}", pkg.pkgver, pkg.pkgrel).into(),
        pkgdesc: pkg.pkgdesc.to_owned(),
        url: pkg.url.to_owned(),
        build_date: pkg.build_date.to_owned(),
        packager: pkg.packager.to_owned(),
        size: pkg.installed_size,
        arch: pkg.arch.to_owned(),
        licenses: pkg.licenses.to_owned(),
        dependencies: pkg.depends.to_owned(),
        dependencies_optional: pkg.optdepends.to_owned()
   };

   let installed_pkg = InstalledPackage {desc: pkgdesc, files: file_list};
   let toml = toml::to_string(&installed_pkg).unwrap();

   create_dir(&dir_path).unwrap();
   let mut file = File::create(format!("{}/{}", &dir_path, "PKGDESC")).expect("Failed to create PKGDESC file\nBad permissions?");
   file.write_all(&toml.as_bytes()).expect("Failed to write to database\nBad permissions?");
}

pub async fn remove_pkg(pkg: &str)
{
    let db_path = config::get_config().general.db_path;
    let dirs = std::fs::read_dir(&db_path).unwrap();

    for dir in dirs
    {
        let path = &dir.unwrap().path().into_os_string().into_string().unwrap();

        if !path.contains(&pkg) {continue;}

        std::fs::remove_dir_all(&path).expect("Failed to remove package from database\nBad permissions?");
    }
}

pub async fn is_pkg_installed(pkg: &api::PackageDetails) -> bool
{
    let config = config::get_config();
    let path = format!("{}{}-{}-{}", &config.general.db_path, &pkg.pkgname, &pkg.pkgver, &pkg.pkgrel);

    if !Path::exists(&Path::new(&path))
    {
        return false;
    }

    return true;
} 

pub fn get_installed_packages() -> Vec<PackageDesc>
{
    let db_path = config::get_config().general.db_path;
    let path = std::fs::read_dir(&db_path).unwrap();
    let mut packages : Vec<PackageDesc> = Vec::new();

    for x in path
    {
        let pkg_path = x.unwrap().path().into_os_string().into_string().unwrap();
        let pkg = format!("{}/{}", pkg_path, "PKGDESC");
        let pkgdesc_contents = std::fs::read_to_string(&pkg).expect("Failed to read PKGDESC!\nBad permissions?");
        let installed_pkg : InstalledPackage = toml::from_str(&pkgdesc_contents).unwrap();

        packages.push(installed_pkg.desc);
    }

    return packages;
}

/// Syncs the databases for all enabled repositories.
/// Needs to be ran as root or as a user with the rights to the database path.
pub async fn sync_mirrors()
{
    let config : config::Config = config::get_config();
    let mirror = config::get_mirrors()[0].to_owned();

    for repo in config.general.enabled_repos
    {
        let dl_url = format!("{}/{}.db", &mirror.replace("$repo", &repo).replace("$arch", &config.general.arch), &repo);
        let dl_path = format!("{}/{}.db", &config.general.db_path, &repo);   
        // TODO: Check to see if the bytes between the new database in the current database are the same 
        if Path::exists(&Path::new(&dl_path))
        {
            std::fs::remove_file(&dl_path).unwrap();
        }

        println!("{} Syncing repository {}", "::".green().bold(), &repo);

        let dl = reqwest::get(&dl_url).await.expect("WHOOPS!");
        let data = dl.bytes().await.unwrap();
        let mut out = File::create(&dl_path).expect("Failed to create file -- Bad permissions?");
        out.write_all(&data).expect("Failed to write data to file.");
    }
}
{
    let db_path = config::get_config().general.db_path;
    let path = std::fs::read_dir(&db_path).unwrap();
    

    for dir in path
    {
        let pkg_path = dir.unwrap().path().into_os_string().into_string().unwrap();
        if !&pkg_path.contains(&pkg_name) {continue;}

        let pkg = format!("{}/{}", &pkg_path, "PKGDESC");
        let contents = std::fs::read_to_string(&pkg).expect("Failed to read PKGDESC!");
        let installed_pkg : InstalledPackage = toml::from_str(&contents).unwrap();

/// Parses an arch linux package desc file.
/* 
TODO: 
There is MOST DEFINITELY a better way to do this. I need to find it, because this looping
might get fucking exhausting for the software, who knows though! It could work completely fine!
this could be the best way to do this and I don't even know about it! Who cares?
*/
fn parse_desc(desc: &str) -> ArchDesc
{
    let split : Vec<&str> = desc.split("\n").collect();

    let mut fname : String = String::new();
    let mut name : String = String::new();
    let mut base : String = String::new();
    let mut ver : String = String::new();
    let mut desc : String = String::new();
    let mut csize : i32 = 0;
    let mut size : i32 = 0;
    let mut md5 : String = String::new();
    let mut sha : String = String::new();
    let mut pgp : String = String::new();
    let mut url : String = String::new();
    let mut license : String = String::new();
    let mut arch : String = String::new();
    let mut build_date : i64 = 0;
    let mut packager : String = String::new();
    let mut depends : Vec<String> = Vec::new();
    let mut opt_depends : Vec<String> = Vec::new();

    for i in 0..split.len()
    {
        match split[i]
        {
            "%FILENAME%" => fname = split[i + 1].into(),
            "%NAME%" => name = split[i + 1].into(),
            "%BASE%" => base = split[i + 1].into(),
            "%VERSION%" => ver = split[i + 1].into(),
            "%DESC%" => desc = split[i + 1].into(), // This could be multi-line sometimes? Not sure.
            "%CSIZE%" => csize = split[i + 1].parse().unwrap(),
            "%ISIZE%" => size = split [i + 1 ].parse().unwrap(),
            "%MD5SUM%" => md5 = split[i + 1].into(),
            "%SHA256SUM%" => sha = split[i + 1].into(),
            "%PGPSIG%" => pgp = split[i + 1].into(),
            "%URL%" => url = split[i + 1].into(),
            "%LICENSE%" => license = split[i + 1].into(),
            "%ARCH%" => arch = split[i + 1].into(),
            "%BUILDDATE%" => build_date = split[i + 1].parse().unwrap(),
            "%PACKAGER%" => packager = split[i + 1].into(),
            "%DEPENDS%" => {
                let mut x = i + 1;
                while split[x] != ""
                {
                    depends.push(split[x].into());
                    x += 1;
                }
            },
            "%OPTDEPENDS" => {
                let mut x = i + 1;
                while split[x] != ""
                {
                    opt_depends.push(split[x].into());
                    x += 1;
                }
            },


            _=>()
        }
    }

    let archdesc : ArchDesc = ArchDesc {
        file_name : fname,
        name : name,
        base : base,
        version : ver,
        desc : desc,
        csize : csize,
        size : size,
        md5s : md5,
        sha256 : sha,
        pgpsig : pgp,
        url : url,
        license : license,
        arch : arch,
        build_date : build_date,
        packager : packager,
        depends : depends,
        opt_depends : opt_depends
    };


    dbg!(&archdesc);
    return archdesc; 
}