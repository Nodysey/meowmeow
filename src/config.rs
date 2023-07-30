use std::fs;

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
