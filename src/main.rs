// This tool collects all possible forks from a github repository and tries to add all of them as
// remote endpoints to the current repository.

use chrono::{
    Local,
    LocalResult,
    TimeZone,
};
use clap::Parser;
use git2;
use octorust::{
    types::ReposListForksSort,
    Client,
    StatusCode,
};
use std::process::exit;


#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Add all forks of a github repository as remotes to the current repository")]
struct Args {
    // Options

    /// Do everything except actually add the remotes
    #[clap(short, long)]
    dry_run: bool,


    /// Add the forks to current repository as remotes
    #[clap(short, long, default_value = "false")]
    add: bool,

    /// Only list the forks, but do not add them as remotes. Sort order is newest first
    ///
    /// Just a list of forks and their own fork count is printed. Example:
    ///
    ///     $ rgf google/battery-historian --list
    ///     ntc-stefan/battery-historian | 0
    ///     ippocratis/battery-historian | 0
    ///     314937885/battery-historian | 0
    ///     pingping-jiang6141/battery-historian | 0
    ///     gilbertalgordo/battery-historian | 0
    ///     goldjunge91/battery-historian | 0
    ///     mengzhiya/battery-historian | 0
    #[clap(short, long, default_value = "false", verbatim_doc_comment)]
    list: bool,

    /// Number of remotes to be added or listed
    #[clap(long, default_value = "10")]
    per_page: u16,

    /// At which point the listing or adding of remotes is started
    #[clap(long, default_value = "1")]
    page: u16,

    /// View current rate limit status
    ///
    /// Output of this option is the current rate limit status of the github api.
    /// Example:
    /// rate-limit:1/5000 available:4999 reset-at:Fri, 15 Mar 2024 13:33:52 +0100
    #[clap(long, default_value = "false", verbatim_doc_comment)]
    rate_limit: bool,

    /// Github token for authentication
    #[clap(short, long, env="GITHUB_TOKEN")]
    token: Option<String>,

    // Arguments

    /// The repository from which the forks are to be fetched
    repository: String,
}

#[derive(Debug)]
pub struct OwnerRepo {
    pub owner: String,
    pub repo: String,
}

impl OwnerRepo {
    pub fn new(orinput: &String) -> Result<OwnerRepo, String> {
        let parts: Vec<&str> = orinput.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid repository format".to_string());
        }
        Ok(OwnerRepo {
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
        })
    }
}


fn to_credential(tok: Option<String>) -> Option<octorust::auth::Credentials> {
    match tok {
        Some(token) => Some(octorust::auth::Credentials::Token(token.clone())),
        None => None,
    }
}

fn unify_remote_name(name: &String) -> String {
    let mut out: String = name.clone();
    out.insert_str(0, "rgf__");
    out.replace("/", "_")
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    let owner_repo = OwnerRepo::new(&args.repository).expect("Invalid repository format: gh standartformat is <owner>/<repo>");

    let client = Client::new("myAgent", to_credential(args.token)).expect("Failed to create gh client");

    if args.rate_limit {
        let rate_limit = match client.rate_limit().get().await {
            Ok(response) => {
                if response.status == StatusCode::OK {
                    response.body
                } else {
                    panic!("Response Status not okay: {}", response.status);
                }
            },
            Err(e) => {
                println!("Error: {}", e);
                exit(1);
            }
        };
        // let x = Local.timestamp_opt(rate_limit.rate.reset, 0);
        let dt = match Local.timestamp_opt(rate_limit.rate.reset, 0) {
            // Some problems, just give the number back as string
            LocalResult::None => rate_limit.rate.reset.to_string(),
            LocalResult::Ambiguous(_, _) => rate_limit.rate.reset.to_string(),
            // Clearly identifiable time. Format as rfc2822
            LocalResult::Single(dt) => dt.to_rfc2822(),
        };
        println!("rate-limit:{}/{} available:{} reset-at:{}",
            rate_limit.rate.used,
            rate_limit.rate.limit,
            rate_limit.rate.remaining,
            dt);
    }

    let forks = match client.repos().list_forks(&owner_repo.owner, &owner_repo.repo, ReposListForksSort::Newest, args.per_page as i64, args.page as i64 ).await {
        Ok(response) => {
            if response.status == StatusCode::OK {
                response.body
            } else {
                panic!("Response Status not okay: {}", response.status);
            }
        },
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };

    if args.list {
        for fork in &forks {
            println!("{} | {}", fork.full_name, fork.forks_count);
        }
    }

    if args.add {
        let repo = match git2::Repository::discover(".") {
            Ok(repo) => repo,
            Err(e) => panic!("Failed to open repository: {}", e),
        };

        let current_remotes = match repo.remotes() {
            Ok(remotes) => remotes,
            Err(e) => panic!("Failed to get remotes: {}", e),
        };

        for fork in forks {
            let remote_name = unify_remote_name(&fork.full_name);

            if current_remotes.iter().any(|r| r.unwrap() == remote_name) {
                println!("= {}", remote_name);
                continue;
            }

            if args.dry_run {
                println!("(+) {}", remote_name);
                continue;
            } else {
                match repo.remote(&remote_name, &fork.clone_url) {
                    Ok(_) => println!("Remote {} added", remote_name),
                    Err(e) => println!("Failed to add remote {}: {}", remote_name, e),
                }
            }
        }
    }

}
