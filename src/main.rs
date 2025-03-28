use repo::{get_all_repo_files, parse_repo_tree};
use repos::{get_default_branch, get_default_user, get_repo_list};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

mod repo;
mod repos;

/// TODO: dependencies
fn main() -> io::Result<()> {
    let user = get_default_user()?;
    let repo_list = get_repo_list(&user)?;
    let repo = select(&repo_list)?;

    let default_branch = get_default_branch(&repo)?;
    let contents = get_all_repo_files(&repo, &default_branch)?;
    let file_list = parse_repo_tree(&contents, &repo, &default_branch)?;
    if file_list.is_empty() {
        eprintln!("No files found in the selected repository.");
        return Ok(());
    }

    let file_names: Vec<String> = file_list.iter().map(|(name, _)| name.clone()).collect();
    let file_list_str = file_names.join("\n");
    let selected_file = select(&file_list_str)?;

    let download_url = file_list
        .into_iter()
        .find(|(name, _)| name == &selected_file)
        .map(|(_, url)| url)
        .unwrap();
    copy_file(&download_url, &selected_file)?;

    Ok(())
}

fn select(input: &str) -> io::Result<String> {
    let mut fzf = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let stdin = fzf.stdin.as_mut().unwrap();
        stdin.write_all(input.as_bytes())?;
    }
    let output = fzf.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn copy_file(download_url: &str, file_name: &str) -> io::Result<()> {
    if let Some(parent) = Path::new(file_name).parent() {
        fs::create_dir_all(parent)?;
    }

    let status = Command::new("curl")
        .args(&["-s", download_url, "-o", file_name])
        .status()?;

    if status.success() {
        println!("File {} copied successfully.", file_name);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to download file",
        ))
    }
}
