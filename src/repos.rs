use indoc::formatdoc;
use std::io::{self};
use std::process::Command;

pub fn get_default_user() -> io::Result<String> {
    let output = Command::new("gh")
        .args(&["api", "user", "-q", ".login"])
        .output()?;
    let user = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if user.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to get GitHub user",
        ))
    } else {
        Ok(user)
    }
}

pub fn get_repo_list(default_user: &str) -> io::Result<String> {
    let graphql_query = formatdoc! {r#"
        query ($owner: String!, $endCursor: String) {{
            repositoryOwner(login: $owner) {{
                repositories(
                    first: 30
                    after: $endCursor
                ) {{
                    pageInfo {{
                        hasNextPage
                        endCursor
                    }}
                    nodes {{
                        nameWithOwner
                    }}
                }}
            }}
        }}
    "#};

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
            "Failed to get repository list",
        ))
    } else {
        Ok(repo_list)
    }
}
