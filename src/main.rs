use repo::{get_all_repo_files, parse_repo_tree};
use repos::{get_default_branch, get_default_user, get_repo_list};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

mod history;
mod repo;
mod repos;

use history::History;

fn main() -> io::Result<()> {
    // 履歴の読み込み
    let mut history = History::load()?;

    // リポジトリ一覧の取得
    let user = get_default_user()?;
    let repo_list = get_repo_list(&user)?;

    // リポジトリリストと履歴を統合（--tacと--layout=reverseを使用するため、履歴を先頭に追加すると最終的に下に表示される）
    let mut combined_list = Vec::new();

    if !history.entries.is_empty() {
        let history_options = history
            .entries
            .iter()
            .map(|entry| format!("[History] {}: {}", entry.repo, entry.file_path))
            .collect::<Vec<String>>();

        combined_list.extend(history_options);
    }

    combined_list.extend(repo_list.lines().map(|s| s.to_string()));

    let combined_str = combined_list.join("\n");

    // 統合リストから選択
    let selected_item = select(&combined_str)?;

    let (repo, selected_file, download_url) = if selected_item.starts_with("[History]") {
        // 履歴が選択された場合
        let history_entry = selected_item.strip_prefix("[History] ").unwrap();

        // 選択された履歴エントリに一致するエントリを見つける
        let entry = match history
            .entries
            .iter()
            .find(|entry| format!("{}: {}", entry.repo, entry.file_path) == history_entry)
        {
            Some(entry) => entry.clone(),
            None => {
                eprintln!("Error: Selected history entry not found.");
                return Ok(());
            }
        };

        // 選択されたエントリからURLを構築
        let default_branch = get_default_branch(&entry.repo)?;
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            entry.repo, default_branch, entry.file_path
        );

        (entry.repo, entry.file_path, url)
    } else {
        // リポジトリが選択された場合
        let repo = selected_item;

        // リポジトリからファイル選択
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

        (repo, selected_file, download_url)
    };

    // ファイルのコピー
    copy_file(&download_url, &selected_file)?;

    // 履歴への追加
    history.add_entry(&repo, &selected_file)?;

    Ok(())
}

fn select(input: &str) -> io::Result<String> {
    let mut fzf = Command::new("fzf")
        // .args(&["--layout=reverse", "--tac"]) // 逆順にして一番下の項目からスタート
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
