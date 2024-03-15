// This tool collects all possible forks from a github repository and tries to add all of them as
// remote endpoints to the current repository.

use clap::Parser;
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

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    let owner_repo = OwnerRepo::new(&args.repository).expect("Invalid repository format: gh standartformat is <owner>/<repo>");

    let client = Client::new("myAgent", to_credential(args.token)).expect("Failed to create gh client");

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
}
