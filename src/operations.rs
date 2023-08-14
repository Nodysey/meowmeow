use crate::database::{ArchDesc, search_db, sync};
use crate::install;

pub async fn install(args: Vec<String>)
{
    let mut reinstall_dependencies = false;
    let mut auto_confirm = false;
    let mut pkgs : Vec<ArchDesc> = vec![];

    if nix::unistd::geteuid() != 0.into() 
    {
        println!("Install needs to be ran as root!");
        return;
    }

    sync().await;

    for i in 2..args.len()
    {
        if args[i] == "-y" || args[i] == "--auto-confirm"
        {
            auto_confirm = true;
            continue;
        }

        if args[i] == "-rd" || args[i] == "--reinstall-deps"
        {
            reinstall_dependencies = true;
            continue;
        }

        let pkg = search_db(&args[i]).await.unwrap();
        
        pkgs.push(pkg);
    }

    println!("::: The following packages will be installed:");

    for pkg in &pkgs
    {
        println!(":: {}", pkg.name);
    }

    if auto_confirm
    {
        for pkg in pkgs
        {
            install::install_package(pkg, reinstall_dependencies).await;
        }

        return;
    }

    println!("\nDo you want to continue with the installation? [Y/N] ");

    let mut confirmation = String::new();
    std::io::stdin().read_line(&mut confirmation).unwrap();

    if confirmation.trim().to_lowercase() != "y" && confirmation.trim() != ""
    {
        return;
    }

    for pkg in pkgs
    {
        install::install_package(pkg, reinstall_dependencies).await;
    }
}