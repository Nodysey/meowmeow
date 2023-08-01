use std::fs;
use serde_derive::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
pub struct Config
{
    pub general : General,
    pub mirrors: Mirrors
}

#[derive(Debug, Deserialize)]
pub struct General
{
    pub arch: String,
    pub db_path: String,
    pub download_path: String,
    pub no_cache: bool
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
    let config : Config = toml::from_str(&contents).unwrap();

    return config;
}

/// Pulls mirrors from /etc/meow.d/mirrorlist
pub fn get_mirrors() -> Vec<String>
{
    let mirrorlist = fs::read_to_string("/etc/meow.d/mirrorlist").unwrap();
    let mut mirrors: Vec<String> = Vec::new();
 
    for mirror in mirrorlist.split("\n")
    {
        let mirror_string = mirror.to_string();
        
        if mirror_string.is_empty()
        {
            continue;
        }

        mirrors.push(mirror_string);
    }

    return mirrors;
}

/// Validates mirrors to ensure they're structured correctly
fn validate_mirror(mirror: &str) -> Result<(), String>
{
    if mirror.contains("$arch") && mirror.contains("$repo")
    {
        return Ok(());
    }

    return Err("Failed to validate mirror!".into());
}

/// Checks the architecture of the CPU we're running the program on.
pub fn get_cpu_arch() -> String
{
    let config = get_config();
    if config.general.arch != "auto"
    {
        return config.general.arch;
    }    

    return "x86_64".to_string();
}