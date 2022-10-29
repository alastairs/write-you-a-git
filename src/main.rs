mod git_objects;
mod repository;

use std::{
    collections::HashSet,
    fs::{create_dir_all, File},
    io::{stdout, Read, Write},
    path::Path,
};

use clap::{Parser, Subcommand};
use git_objects::{
    git_commit::Commit,
    git_object::GitObjectData,
    git_tree::{Leaf, Tree},
};
use repository::repository::ReadObjectErrorType;

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

    /// Provider content of repository objects.
    #[command(name = "cat-file", about)]
    CatFile {
        /// Specify the type
        r#type: String,
        /// The object to display
        object: String,
    },

    /// Compute object ID and optionally creates a blob from a file.
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

    /// Display history of a given commit.
    Log {
        /// Commit to start at
        commit: Option<String>,
    },

    /// Pretty-print a tree object.
    #[command(name = "ls-tree", about)]
    LsTree {
        /// The object to show.
        object: String,
    },

    /// Checkout a commit inside of a directory.
    #[command(about)]
    Checkout {
        /// The commit or tree to checkout.
        commit: String,

        /// The EMPTY directory to checkout on.
        path: String,
    },
}

fn main() -> Result<(), ReadObjectErrorType> {
    env_logger::init();
    let args = Args::parse();

    return match args.command {
        Some(GitCommands::Init { directory: path }) => {
            Repository::repo_create(Path::new(&path)).map_err(ReadObjectErrorType::IO)
        }
        Some(GitCommands::CatFile { r#type, object }) => {
            cat_file(r#type, &object).map_err(ReadObjectErrorType::IO)
        }
        Some(GitCommands::HashObject {
            r#type,
            write,
            path,
        }) => hash_file(r#type, write, path).map_err(ReadObjectErrorType::IO),
        Some(GitCommands::Log { commit }) => match commit {
            Some(commit) => print_log(commit),
            None => print_log("HEAD".to_string()),
        },
        Some(GitCommands::LsTree { object }) => ls_tree(&object),
        Some(GitCommands::Checkout { commit, path }) => checkout(commit, path),
        None => Ok({}),
    };
}

fn cat_file(r#type: String, object: &String) -> Result<(), std::io::Error> {
    let repo = Repository::repo_find(String::from(object), None)
        .expect("No git directory when required")
        .unwrap();

    let sha = repo.object_find(String::from(object), Some(r#type), None);
    let object = repo.read_object(sha).unwrap();

    let GitObjectData(_, data) = object.serialize();
    stdout().write(data.as_slice())?;

    return Ok(());
}

fn hash_file(r#type: String, write: bool, path: String) -> Result<(), std::io::Error> {
    let repo = match write {
        true => Some(Repository::new(&Path::new("."), false)),
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

fn print_log(commit: String) -> Result<(), ReadObjectErrorType> {
    let repo = Repository::repo_find(".".to_string(), None)
        .expect("No git directory when required")
        .unwrap();

    print!("digraph wyaglog{{");
    let sha = repo.object_find(commit.clone(), None, None);
    repo.log_graphviz(sha, &mut HashSet::new())?;
    print!("}}");

    return Ok(());
}

fn ls_tree(object: &String) -> Result<(), ReadObjectErrorType> {
    let repo = Repository::repo_find(".".to_string(), None)
        .expect("No git directory when required")
        .unwrap();
    let sha = repo.object_find(object.clone(), Some("tree".to_owned()), None);
    let object = repo.read_object(sha)?;
    let object = object
        .as_any()
        .downcast_ref::<Tree>()
        .expect("Not a Tree object.");

    for item in &object.items {
        let Leaf(mode, path, sha) = item;
        let GitObjectData(fmt, _) = (*repo.read_object(sha.to_string())?).get_data();
        println!(
            "{} {} {}\t{}",
            "0".repeat(6 - mode.len()) + mode.as_str(),
            fmt,
            &sha,
            path
        )
    }
    return Ok(());
}

fn checkout(commit: String, path: String) -> Result<(), ReadObjectErrorType> {
    let repo = Repository::repo_find(".".to_string(), None)
        .expect("No git directory when required")
        .unwrap();

    let object = repo.read_object(repo.object_find(commit, None, None))?;

    // If the object is a commit, grab its tree
    let commit = object.as_any().downcast_ref::<Commit>();

    let tree_candidate = match commit {
        Some(commit) => repo.read_object(
            commit
                .kvlm
                .get("tree")
                .ok_or(ReadObjectErrorType::TreeNotFoundError)?[0]
                .clone(),
        )?,
        None => object,
    };

    let tree = match tree_candidate.as_any().downcast_ref::<Tree>() {
        Some(tree) => tree,
        None => panic!("Not a tree object!"),
    };

    let path = Path::new(&path);
    if path.exists() {
        if !path.is_dir() {
            return Err(ReadObjectErrorType::InvalidPathError);
        }

        let is_empty = path
            .read_dir()
            .map_err(ReadObjectErrorType::IO)?
            .next()
            .is_none();
        if !is_empty {
            return Err(ReadObjectErrorType::InvalidPathError);
        }
    } else {
        create_dir_all(&path).map_err(ReadObjectErrorType::IO)?;
    }

    repo.tree_checkout(tree, &path.canonicalize().map_err(ReadObjectErrorType::IO)?)?;

    return Ok(());
}
