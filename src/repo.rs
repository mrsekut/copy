use serde::Deserialize;
use std::io::{self};
use std::process::Command;

#[derive(Deserialize)]
struct TreeResponse {
    tree: Vec<TreeEntry>,
}

#[derive(Deserialize)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    node_type: String,
}

pub fn get_all_repo_files(repo: &str, branch: &str) -> io::Result<String> {
    let tree_api_path = format!("repos/{}/git/trees/{}?recursive=1", repo, branch);
    let output = Command::new("gh").args(&["api", &tree_api_path]).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Parse the Git Trees API response and generate the Raw URL from the branch and repository information
pub fn parse_repo_tree(
    contents: &str,
    repo: &str,
    branch: &str,
) -> io::Result<Vec<(String, String)>> {
    let tree_resp: TreeResponse =
        serde_json::from_str(contents).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut file_list = Vec::new();
    for entry in tree_resp.tree {
        if entry.node_type == "blob" {
            // Raw URL generation: https://raw.githubusercontent.com/{repo}/{branch}/{path}
            let raw_url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}",
                repo, branch, entry.path
            );
            file_list.push((entry.path, raw_url));
        }
    }
    Ok(file_list)
}
