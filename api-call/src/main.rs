use serde::Deserialize;
use reqwest::Error;
use reqwest::header::{USER_AGENT, ACCEPT};

#[derive(Debug, Deserialize)]
struct User {
    login: String,
    id: u32,
}

#[derive(Debug, Deserialize)]
struct GithubError {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let repo_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/stargazers",
        owner = "rust-lang-nursery",
        repo = "rust-cookbook"
    );
    println!("Fetching: {}", repo_url);

    let client = reqwest::Client::new();
    let response = client
        .get(&repo_url)
        .header(USER_AGENT, "rust web-api-client demo")
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        // Optional: If you have a Personal Access Token, uncomment the line below to authenticate
        // .header("Authorization", "Bearer YOUR_GITHUB_TOKEN") 
        .send()
        .await?;

    // Check if the server returned a 200 OK status
    if response.status().is_success() {
        let users: Vec<User> = response.json().await?;
        println!("Success! Found {} stargazers:", users.len());
        println!("{:?}", users);
    } else {
        // If it failed, decode the single map error object
        let status = response.status();
        let err_body: GithubError = response.json().await?;
        println!("GitHub API Error (Status {}): {}", status, err_body.message);
    }

    Ok(())
}
