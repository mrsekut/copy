use repos::{get_default_user, get_repo_list};
use serde::Deserialize;
use std::io::{self, Write};
use std::process::{Command, Stdio};

mod repos;

#[derive(Deserialize)]
struct RepoContent {
    name: String,
    #[serde(rename = "download_url")]
    download_url: Option<String>,
    #[serde(rename = "type")]
    file_type: String,
}

/// TODO: deps
fn main() -> io::Result<()> {
    let user = get_default_user()?;
    let repo_list = get_repo_list(&user)?;
    let repo = select(&repo_list)?;

    let contents = get_repo_contents(&repo)?;
    let file_list = parse_repo_contents(&contents)?;
    if file_list.is_empty() {
        eprintln!("選択されたリポジトリにファイルが見つかりませんでした。");
        return Ok(());
    }
    let file_names: Vec<String> = file_list.iter().map(|(name, _)| name.clone()).collect();
    let file_list_str = file_names.join("\n");
    let selected_file = select(&file_list_str)?;
    if selected_file.is_empty() {
        eprintln!("ファイルが選択されませんでした。");
        return Ok(());
    }
    println!("選択されたファイル: {}", selected_file);

    // 選択されたファイルのダウンロードURLを取得し、ファイルをダウンロード
    let download_url = file_list
        .into_iter()
        .find(|(name, _)| name == &selected_file)
        .map(|(_, url)| url)
        .unwrap();
    println!(
        "{} を {} からダウンロードします...",
        selected_file, download_url
    );
    download_file(&download_url, &selected_file)?;

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

/// 選択されたリポジトリのルートディレクトリ内容を取得する
fn get_repo_contents(repo: &str) -> io::Result<String> {
    let repo_api_path = format!("repos/{}/contents", repo);
    let output = Command::new("gh").args(&["api", &repo_api_path]).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// JSONをパースし、ファイルのみ（ファイル名とダウンロードURL）の一覧を返す
fn parse_repo_contents(contents: &str) -> io::Result<Vec<(String, String)>> {
    let repo_contents: Vec<RepoContent> =
        serde_json::from_str(contents).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let file_list: Vec<(String, String)> = repo_contents
        .into_iter()
        .filter(|item| item.file_type == "file" && item.download_url.is_some())
        .map(|item| (item.name, item.download_url.unwrap()))
        .collect();
    Ok(file_list)
}

/// curl を使い、指定URLからファイルをダウンロードする
fn download_file(download_url: &str, file_name: &str) -> io::Result<()> {
    let status = Command::new("curl")
        .args(&["-s", download_url, "-o", file_name])
        .status()?;
    if status.success() {
        println!("ファイル {} のダウンロードに成功しました。", file_name);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "ファイルのダウンロードに失敗しました",
        ))
    }
}
