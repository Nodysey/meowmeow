use std::fs;
use serde_derive::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
pub struct Config
{
    pub general : General,
    pub mirrors: Mirrors,
}

#[derive(Debug, Deserialize)]
pub struct General
{
    pub arch: String,
    pub db_path: String,
    pub download_path: String,
    pub no_cache: bool,
    pub enabled_repos: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct Mirrors
{
    pub mirrorlist : String,
    pub max_ping : i32
}

pub fn get_config() -> Config
{
    let path = "/etc/meow.conf";
    let contents = fs::read_to_string(path).expect("Failed to read contents of /etc/meow.conf.");
    let mut config : Config = toml::from_str(&contents).unwrap();
    // dbg!(&config.general.db_path);

    if config.general.arch == "any"
    {
        config.general.arch = get_cpu_arch();
    }

    return config;
}

/// Pulls mirrors from the mirrorlist in meow.conf
pub fn get_mirrors() -> Vec<String>
{
    let mirrorlist_path = get_config().mirrors.mirrorlist;
    let mirrorlist = fs::read_to_string(&mirrorlist_path).unwrap();
    let mut mirrors: Vec<String> = Vec::new();
 
    for mirror in mirrorlist.split("\n")
    {
        let mirror_string = mirror.to_string();
    
        if mirror_string.is_empty()
        {
            continue;
        }
        
        if !validate_mirror(&mirror)
        {
            continue;
        }

        mirrors.push(mirror_string);
    }

    return mirrors;
}

/// Validates mirrors to ensure they're structured correctly
pub fn validate_mirror(mirror: &str) -> bool
{
    if mirror.contains("$arch") && mirror.contains("$repo")
    {
        return true
    }

    return false;
}

/// Checks the architecture of the CPU we're running the program on.
pub fn get_cpu_arch() -> String
{  
    // TODO: This needs to actually get the CPU architecture
    return "x86_64".to_string();
}