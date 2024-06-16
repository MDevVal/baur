use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn run_process_with_output(
    command: &str,
    args: Vec<String>,
    current_dir: Option<&PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new(command)
        .args(args)
        .current_dir(current_dir.unwrap_or(&PathBuf::from(".")))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    _ = cmd.wait();

    Ok(())
}
