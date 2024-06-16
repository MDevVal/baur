use crate::args::Args;
use crate::process::run_process_with_output;
use dialoguer::Confirm;
use reqwest::blocking::{get, Response};
use serde_json::Value;
use std::error::Error;
use std::path::PathBuf;

pub fn cmd(args: Args) -> Result<(), Box<dyn Error>> {
    for flag in &args.operation_flags {
        match flag {
            's' => {
                return sync_search(args);
            }
            'y' => {
                break;
            }
            _ => {
                println!("Warning: Ignoring unknown flag '{}'", flag);
            }
        }
    }

    sync(args)
}

fn sync_search(args: Args) -> Result<(), Box<dyn Error>> {
    if args.target.is_none() {
        return Ok(());
    }

    let target = args.target.unwrap();

    let url = format!(
        "https://aur.archlinux.org/rpc/v5/search/{}?by=name-desc",
        target
    );

    let req: Response = get(url)?;

    let text = req.text()?;

    let value: Value = serde_json::from_str(&text)?;

    let results = &value["results"];

    for result in results.as_array().expect("Error: unable to parse results") {
        let name = result["Name"].as_str().unwrap_or("Unknown");
        let version = result["Version"].as_str().unwrap_or("Unknown");
        let description = result["Description"].as_str().unwrap_or("Unknown");
        println!("aur/{} {}", name, version);
        println!("  {}", description);
    }

    Ok(())
}

fn sync(args: Args) -> Result<(), Box<dyn Error>> {
    let package = args
        .target
        .ok_or("error: no targets specified (use -h for help)")?;
    let baur_directory = get_baur_directory()?;
    let pkg_info = fetch_package_info(&package)?;

    if pkg_info.len() > 1 {
        println!("error: Multiple packages found");
    } else if pkg_info.is_empty() {
        println!("error: No packages found");
    } else {
        let pkg = &pkg_info[0];
        display_package_info(pkg);

        if confirm_install()? {
            let pkg_directory = install_package(pkg, &baur_directory)?;
            build_package(&pkg_directory)?;
        } else {
            println!("Aborted");
        }
    }

    Ok(())
}

fn get_baur_directory() -> Result<PathBuf, Box<dyn Error>> {
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let baur_directory = home_dir.join(".cache/baur");

    if !baur_directory.exists() {
        std::fs::create_dir_all(&baur_directory)?;
    }

    Ok(baur_directory)
}

fn fetch_package_info(package: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let pkg_url = format!("https://aur.archlinux.org/rpc/v5/info/{}", package);
    let body = get(pkg_url)?.text()?;
    let json: Value = serde_json::from_str(&body)?;

    Ok(json["results"].as_array().cloned().unwrap_or_default())
}

fn display_package_info(pkg: &Value) {
    let pkg_name = pkg["Name"].as_str().unwrap_or("Unknown");
    let pkg_version = pkg["Version"].as_str().unwrap_or("Unknown");
    let pkg_description = pkg["Description"].as_str().unwrap_or("No description");

    println!("Name: {}", pkg_name);
    println!("Version: {}", pkg_version);
    println!("Description: {}", pkg_description);
}

fn confirm_install() -> Result<bool, Box<dyn Error>> {
    let confirm = Confirm::new()
        .with_prompt("Do you want to install this package?")
        .interact()?;

    Ok(confirm)
}

fn install_package(pkg: &Value, baur_directory: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let pkg_name = pkg["Name"].as_str().ok_or("Failed to get package name")?;
    let pkg_directory = baur_directory.join(pkg_name);
    let repo = format!("https://aur.archlinux.org/{}.git", pkg_name);

    if pkg_directory.exists() {
        println!("Package already exists in cache");
        return Ok(pkg_directory);
    }

    run_process_with_output(
        "git",
        vec!["clone".to_owned(), repo, pkg_name.to_owned()],
        Some(baur_directory),
    )?;

    Ok(pkg_directory)
}

fn build_package(pkg_directory: &PathBuf) -> Result<(), Box<dyn Error>> {
    run_process_with_output("makepkg", vec!["-si".to_owned()], Some(pkg_directory))
}
