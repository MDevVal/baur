use std::env;
use std::process::Command;
use reqwest::blocking::get;
use serde_json::Value;
use std::io::BufReader;
use std::io::BufRead;
use std::process::Stdio;
use std::error::Error;
use std::path::PathBuf;
use dialoguer::Confirm;
use dirs;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let _additional_options: Vec<String> = Vec::new();
    let mut operation: Option<char> = None;
    let mut target: Option<String> = None;

    for (i, arg) in args.iter().enumerate() {
        if arg.starts_with("-D") || arg.starts_with("-Q") || arg.starts_with("-R") || arg.starts_with("-S") || arg.starts_with("-T") || arg.starts_with("-U") || arg.starts_with("-F") || arg.starts_with("-V") || arg.starts_with("-h") {
            if operation.is_none() { 
                operation = Some(arg.chars().nth(1).unwrap());
                target = args.get(i + 1).cloned();
            } else {
                panic!("error: Multiple operations provided");
            }
        }
    }

    match operation {
        Some(op) => {
            match op {
                'D' => {
                    println!("Downloading packages");
                }
                'Q' => {
                    println!("Querying packages");
                }
                'R' => {
                    println!("Removing packages");
                }
                'S' => {
                    println!("Synchronizing packages");
                    match target {
                        Some(pkg) => {
                            sync(&pkg)?;
                        }
                        None => {
                            println!("error: No package provided");
                        }
                    }
                }
                'T' => {
                    println!("Testing packages");
                }
                'U' => {
                    println!("Upgrading packages");
                }
                'F' => {
                    println!("Force operation");
                }
                'V' => {
                    println!("Display version");
                }
                'h' => {
                    println!("Display help");
                }
                _ => {
                    println!("Invalid operation");
                }
            }
        }
        None => {
            println!("error: No operation provided");
        }
    }

    Ok(())
}

fn sync(package: &str) -> Result<(), Box<dyn Error>> {
    let baur_directory = get_baur_directory()?;
    let pkg_info = fetch_package_info(package)?;

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

    std::process::Command::new("git")
        .arg("clone")
        .arg(repo)
        .arg(&pkg_directory)
        .current_dir(baur_directory)
        .output()?;

    Ok(pkg_directory)
}

fn build_package(pkg_directory: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("makepkg")
        .arg("-si")
        .current_dir(pkg_directory)
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.as_mut().ok_or("Failed to get stdout")?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(line) => println!("{}", line),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    let status = child.wait()?;

    if !status.success() {
        return Err("Failed to build package".into());
    }

    Ok(())
}
