use serde::Deserialize;
use std::io::{self, Write};
use std::process::{Command, Stdio};

#[derive(Deserialize)]
struct RepoContent {
    name: String,
    #[serde(rename = "download_url")]
    download_url: Option<String>,
    #[serde(rename = "type")]
    file_type: String,
}

/// GitHub CLI を使って、デフォルトのGitHubユーザー名を取得する
fn get_default_user() -> io::Result<String> {
    let output = Command::new("gh")
        .args(&["api", "user", "-q", ".login"])
        .output()?;
    let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if user.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "GitHubユーザーの取得に失敗しました",
        ))
    } else {
        Ok(user)
    }
}

/// GraphQLクエリを使って、ユーザーのリポジトリ一覧を取得する
fn get_repo_list(default_user: &str) -> io::Result<String> {
    let graphql_query = r#"
query ($owner: String!, $endCursor: String) {
  repositoryOwner(login: $owner) {
    repositories(first: 100, after: $endCursor) {
      nodes {
        nameWithOwner
      }
    }
  }
}
"#;
    let output = Command::new("gh")
        .args(&[
            "api",
            "graphql",
            "--paginate",
            "-F",
            &format!("owner={}", default_user),
            "-f",
            &format!("query={}", graphql_query),
            "-q",
            ".data.repositoryOwner.repositories.nodes[].nameWithOwner",
        ])
        .output()?;
    let repo_list = String::from_utf8_lossy(&output.stdout).to_string();
    if repo_list.trim().is_empty() {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "リポジトリ一覧の取得に失敗しました",
        ))
    } else {
        Ok(repo_list)
    }
}

/// fzf を使って、与えられたリストから1行選択する
fn select_with_fzf(input: &str) -> io::Result<String> {
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

fn main() -> io::Result<()> {
    // GitHubのデフォルトユーザーを取得
    let user = get_default_user()?;
    println!("デフォルトユーザー: {}", user);

    // リポジトリ一覧を取得し、fzfで選択
    let repo_list = get_repo_list(&user)?;
    let selected_repo = select_with_fzf(&repo_list)?;
    if selected_repo.is_empty() {
        eprintln!("リポジトリが選択されませんでした。");
        return Ok(());
    }
    println!("選択されたリポジトリ: {}", selected_repo);

    // 選択したリポジトリの内容を取得
    let contents = get_repo_contents(&selected_repo)?;
    let file_list = parse_repo_contents(&contents)?;
    if file_list.is_empty() {
        eprintln!("選択されたリポジトリにファイルが見つかりませんでした。");
        return Ok(());
    }
    let file_names: Vec<String> = file_list.iter().map(|(name, _)| name.clone()).collect();
    let file_list_str = file_names.join("\n");
    let selected_file = select_with_fzf(&file_list_str)?;
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
