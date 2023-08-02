use crate::config;

use std::fs::OpenOptions;
use std::io::Write;

pub fn add_mirror(mirror: &str)
{
    let mirrorlist_path = config::get_config().mirrors.mirrorlist;

    let mut file = OpenOptions::new()
        .append(true)
        .open(&mirrorlist_path)
        .unwrap();

    writeln!(file, "{}", &mirror).unwrap();

    println!("Added {} to {}", &mirror, &mirrorlist_path);
}