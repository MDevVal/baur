use std::error::Error;

mod args;
mod process;
mod sync;

use crate::process::run_process_with_output;
use args::Args;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().collect::<Vec<String>>();
    let mut parsed_args = Args {
        operation: None,
        operation_flags: Vec::new(),
        target: None,
        additional_options: Vec::new(),
    };

    args.remove(0);

    for arg in args.iter() {
        if arg.starts_with('-') & !arg.starts_with("--") {
            if parsed_args.operation.is_none() {
                parsed_args.operation = Some(arg.chars().nth(1).unwrap());
                for flag in arg.chars().skip(2) {
                    parsed_args.operation_flags.push(flag);
                }
            } else {
                panic!("error: Multiple operations provided");
            }
        } else if arg.starts_with("--") {
            parsed_args.additional_options.push(arg.clone());
        } else if parsed_args.target.is_none() {
            parsed_args.target = Some(arg.clone());
        } else {
            panic!("Error: Multiple targets provided");
        }
    }

    match parsed_args.operation {
        Some('S') => {
            sync::cmd(parsed_args)?;
        }
        _ => {
            run_process_with_output("man", vec!["pacman".to_owned()], None)?;
        }
    }

    Ok(())
}
