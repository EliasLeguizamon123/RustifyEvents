use reqwest;
use tokio;
use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use polychrome::ColorPrintExt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.username.is_empty() {
        println!("Please provide a username");
        return Ok(());
    }

    if args.username.contains("/") {
        println!("Please provide a username, not a repository");
        return Ok(());
    }

    if args.username.contains(" ") || args.username.contains("\t") || args.username.contains(",") {
        println!("Please provide only one username");
        return Ok(());
    }

    let url = format!("https://api.github.com/users/{}/events", args.username);

    let client = reqwest::Client::new();

    let response = client
        .get(&url)
        .header("User-Agent", "gh-rust-activity")
        .send()
        .await?;


    
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        eprintln!("This username does not exists: `{}`.", args.username);
        return Ok(());
    }

    if !response.status().is_success() {
        eprintln!("An error occurred: `{}`.", response.status());
        return Ok(());
    }
    
    let json: Value = response.json().await?;

    let mut events_by_repo: HashMap<String, Vec<String>> = HashMap::new();

    for event in json.as_array().unwrap() {
        let event_type = event["type"].as_str().unwrap();
        let repo = event["repo"]["name"].as_str().unwrap();
        let created_at = event["created_at"].as_str().unwrap();

        let event_description = match event_type {
            "PushEvent" => format!("Commit {} at {}", event["payload"]["commits"][0]["message"], created_at),
            "PullRequestEvent" => format!("Pull Request {} at {}", event["payload"]["pull_request"]["title"], created_at),
            "DeleteEvent" => format!("Deleted {} at {}", event["payload"]["ref"], created_at),
            "CreateEvent" => format!("Created {} at {}", event["payload"]["ref"], created_at),
            "ForkEvent" => format!("Forked {} at {}", repo, created_at),
            "WatchEvent" => format!("Starred {} at {}", repo, created_at),
            _=> format!("{} at {}", event_type, created_at),
        };

        events_by_repo.entry(repo.to_string()).or_insert(Vec::new()).push(event_description);
    }

    for (repo, events) in events_by_repo.iter() {
        println!("{}", format!("- Repository: `{}`", repo).as_str().color(114, 135, 253));

        for event in events {
            println!("  - {}", event);
        }

        println!("\n");
    }

    Ok(())
}
