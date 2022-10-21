mod git_objects;
mod repository;

use clap::{Parser, Subcommand};

/// The stupid content tracker
#[derive(Parser, Debug)]
#[command(name = "wyag", version, about, long_about = None)]
struct Args {
    /// The command to run: add, checkout, commit, ...
    #[command(subcommand)]
    command: Option<GitCommands>,
}

#[derive(Subcommand, Debug)]
enum GitCommands {
    Init { directory: String },
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let result = match args.command {
        Some(GitCommands::Init { directory: path }) => {
            repository::repository::Repository::repo_create(path)
        }
        None => Ok({}),
    };

    result.expect("?");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
