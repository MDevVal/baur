use std::io::Write;
use std::{env, io::stdout};
use std::process::Command;
use reqwest::blocking::get;
use serde_json::Value;
use std::io::BufReader;
use std::io::BufRead;
use std::process::{Stdio};

// baur -<operation><options> for example -Syu where S is the operation and y and u are options
fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                            sync(&pkg);
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

fn sync(package: &str) {
    let baur_directory = std::env::var("HOME").expect("Failed to get home directory") + "/.cache/baur";
    if !std::path::Path::new(&baur_directory).exists() {
        std::fs::create_dir(&baur_directory).expect("Failed to create baur cache directory");
    }

    let pkg_url = format!("https://aur.archlinux.org/rpc/v5/info/{}", package);
    let body = get(pkg_url).expect("Failed to fetch package").text().expect("Failed to parse package");
    let json: Value = serde_json::from_str(&body).expect("Failed to parse JSON");

    if json["resultcount"].as_u64().expect("Failed to parse resultcount") > 1 {
        println!("error: Multiple packages found");
    } else if json["resultcount"] == 0 {
        println!("error: No packages found");
    } else {
        let pkg = &json["results"][0];
        let pkg_name = pkg["Name"].as_str().unwrap();
        let pkg_version = pkg["Version"].as_str().unwrap();
        let pkg_description = pkg["Description"].as_str().unwrap();
        println!("Name: {}", pkg_name);
        println!("Version: {}", pkg_version);
        println!("Description: {}", pkg_description);
        print!("\nDo you want to install this package? [Y/n] ");

        let _ = stdout().flush();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read input");
        match input.trim() {
            "Y" | "y" => {
                let pkg_directory = format!("{}/{}", &baur_directory, pkg_name);
                let repo = format!("https://aur.archlinux.org/{}.git", pkg_name);
                Command::new("git").arg("clone").arg(repo).arg(&pkg_directory).current_dir(&baur_directory).output().expect("Failed to clone repository");
                makepkg(&pkg_directory).expect("Failed to build package");
            }
            _ => {
                println!("Aborted");
            }
        }
    }
}

fn makepkg(directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    let process = Command::new("makepkg") 
        .arg("-si")
        .current_dir(directory)
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdout) = process.stdout {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }
    }

    Ok(())
}
