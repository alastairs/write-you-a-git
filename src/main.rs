mod git_objects;
mod repository;

use std::{
    fs::File,
    io::{stdout, Read, Write},
};

use clap::{Parser, Subcommand};
use git_objects::git_object::GitObjectData;

use crate::{git_objects::git_object::GitObject, repository::repository::Repository};

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
    Init {
        directory: String,
    },

    /// Provider content of repository objects
    #[command(name = "cat-file", about)]
    CatFile {
        /// Specify the type
        r#type: String,
        /// The object to display
        object: String,
    },

    /// Compute object ID and optionally creates a blob from a file
    #[command(name = "hash-object", about)]
    HashObject {
        /// Specify the type
        #[arg(short, long)]
        r#type: String,

        /// Actually write the object into the database
        #[arg(short, long)]
        write: bool,

        /// Read object from <path>
        path: String,
    },
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let result = match args.command {
        Some(GitCommands::Init { directory: path }) => Repository::repo_create(path),
        Some(GitCommands::CatFile { r#type, object }) => cat_file(r#type, &object),
        Some(GitCommands::HashObject {
            r#type,
            write,
            path,
        }) => hash_file(r#type, write, path),
        None => Ok({}),
    };

    result.expect("?");
}

fn cat_file(r#type: String, object: &String) -> Result<(), std::io::Error> {
    let repo = Repository::repo_find(String::from(object), None)
        .expect("No git directory when required")
        .unwrap();

    let sha = repo.object_find(String::from(object), Some(r#type), None);
    let data = repo.read_object(sha).unwrap();

    let object = GitObject::new(Some(repo), Some(data));
    let GitObjectData(_, data) = object.serialize();
    stdout().write(data.as_slice())?;

    return Ok(());
}

fn hash_file(r#type: String, write: bool, path: String) -> Result<(), std::io::Error> {
    let repo = match write {
        true => Some(Repository::new(&String::from("."), false)),
        false => None,
    };

    let fd = File::open(path)?;
    let sha = object_hash(fd, r#type, repo);
    stdout().write(format!("{:?}", sha).as_bytes())?;

    return Ok(());
}

fn object_hash(
    mut fd: File,
    fmt: String,
    repo: Option<Repository>,
) -> Result<String, std::io::Error> {
    let mut data = Vec::<u8>::new();
    fd.read_to_end(&mut data)?;

    let write_object =
        GitObject::write_object(GitObject::new(repo, Some(GitObjectData(fmt, data))), None);
    return Ok(write_object);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
