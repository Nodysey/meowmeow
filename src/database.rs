use flate2::read::GzDecoder;
use toml;
use serde_derive::{Serialize, Deserialize};
use std::fs::{File, create_dir};
use std::io::{Write, Error, Read};
use std::path::Path;
use reqwest;
use tar;
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
    pub build_date : i64,
    pub packager: String,
    pub size : i32,
    pub arch : String,
    pub license : String,
    pub dependencies : Vec<String>,
    pub dependencies_optional : Vec<String>
}

/// Specifically for handling arch desc files.
/// Primarily for use with the arch .db files.
#[derive(Debug)]
pub struct ArchDesc
{
    pub file_name : String,
    pub name : String,
    pub base : String,
    pub repo : String,
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

// TODO: This needs to pull from the "files" file included with arch .tar.zst archives
pub async fn add_pkg(pkg: &ArchDesc, files: Vec<String>)
{
    let config = config::get_config();
    let dir_path : String = format!("{}/{}-{}", config.general.db_path, &pkg.name, &pkg.version);

    if Path::exists(&Path::new(&dir_path))
    {
        // TODO:
        return;
    }

    let pkgdesc = PackageDesc {
        pkgname: pkg.name.to_owned(),
        pkgbase: pkg.base.to_owned(),
        pkgver: pkg.version.to_owned(),
        pkgdesc: pkg.desc.to_owned(),
        url: pkg.url.to_owned(),
        build_date: pkg.build_date.to_owned(),
        packager: pkg.packager.to_owned(),
        size: pkg.size,
        arch: pkg.arch.to_owned(),
        license: pkg.license.to_owned(),
        dependencies: pkg.depends.to_owned(),
        dependencies_optional: pkg.opt_depends.to_owned()
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

pub async fn is_pkg_installed(pkg: &ArchDesc) -> bool
{
    let config = config::get_config();
    let path = format!("{}/{}-{}", &config.general.db_path, &pkg.name, &pkg.version);

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
/// Needs to be ran as root.
pub async fn sync()
{
    let config : config::Config = config::get_config();
    let mirror = config::get_mirrors()[0].to_owned();

    for repo in config.general.enabled_repos
    {
        let dl_url = format!("{}/{}.db", &mirror.replace("$repo", &repo).replace("$arch", &config.general.arch), &repo);
        let dl_path = format!("{}/{}.db", &config.general.db_path, &repo);   


        println!("{} Syncing repository {}", "::".green().bold(), &repo);

        let dl = reqwest::get(&dl_url).await.expect("WHOOPS!");
        let data = dl.bytes().await.unwrap();

        if Path::exists(&Path::new(&dl_path))
        {
            let existing_db = File::open(&dl_path).unwrap();
            let mut reader = std::io::BufReader::new(existing_db);
            let mut buffer: Vec<u8> = Vec::new();
            
            reader.read_to_end(&mut buffer).unwrap();

            if &buffer.as_slice() == &data {continue;} 
            
            std::fs::remove_file(&dl_path).unwrap();
        }

        let mut out = File::create(&dl_path).expect("Failed to create file -- Bad permissions?");
        out.write_all(&data).expect("Failed to write data to file.");
    }
}

/// Searches the synced databases for the best match to a package's EXACT name.
/// Assumes that mirrors are synchronized before running.
pub async fn search_db(pkgname : &str) -> Result<ArchDesc, Error>
{
    let config : config::Config = config::get_config();
    
    for repo in config.general.enabled_repos
    {
        let db_path = format!("{}/{}.db", &config.general.db_path, repo);
        dbg!(&db_path);

        // Arch's .db files are really .tar.gz files in disguise!
        let tar_gz = File::open(&db_path).expect("Failed to open database file. Corrupted or bad permissions?");
        let tar = GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);

        /* 
        This code is temporary, there's a way to iterate through 
        the archive's files without having to expand it first,
        but at the moment, I'm having trouble trying to find
        out how exactly to do that without having 50,000,000
        problems with borrowing and moved values.
         */
        
        let tmp_path : String = format!("{}/{}", &config.general.download_path, &repo);
        if Path::exists(Path::new(&tmp_path))
        {
            std::fs::remove_dir_all(&tmp_path).expect("Failed to remove old database tmp file. Bad permissions?");
        }

        archive.unpack(&tmp_path).unwrap();

        for file in std::fs::read_dir(&tmp_path).unwrap()
        {
            let filename = file.unwrap().file_name().into_string().unwrap();
            let desc_path : String = format!("{}/{}/desc", &tmp_path, &filename);

            if !filename.contains(pkgname) {continue;}
            
            let desc = parse_desc(&std::fs::read_to_string(&desc_path).unwrap(), &repo);

            if desc.name != pkgname {continue;}

            std::fs::remove_dir_all(&tmp_path).expect("Failed to remove db temporary dir. Bad perms or file is still being used?");
            return Ok(desc);
        }

        std::fs::remove_dir_all(&tmp_path).expect("Failed to remove db temporary dir. Bad perms or file is still being used?");
    }

    return Err(Error::new(std::io::ErrorKind::NotFound, format!("Failed to find the package {}", pkgname)));
}

/// Parses an arch linux package desc file.
/* 
TODO: 
There is MOST DEFINITELY a better way to do this. I need to find it, because this looping
might get fucking exhausting for the software, who knows though! It could work completely fine!
this could be the best way to do this and I don't even know about it! Who cares?
*/
fn parse_desc(desc: &str, repo: &str) -> ArchDesc
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
        repo : repo.into(),
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

    return archdesc; 
}