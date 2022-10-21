pub mod repository {
    use std::{
        fs::{create_dir_all, File},
        io::{self, Read, Write},
        path::{Path, PathBuf, MAIN_SEPARATOR},
    };

    use configparser::ini::{Ini, IniDefault};
    use flate2::read::ZlibDecoder;

    use crate::git_objects::git_object::GitObjectData;

    /// A git repository
    pub struct Repository {
        worktree: PathBuf,
        git_dir: PathBuf,
        config: Ini,
    }

    impl Repository {
        /// Creates a new [`Repository`].
        ///
        /// # Panics
        ///
        /// Panics if .
        pub(crate) fn new(path: &String, force: bool) -> Repository {
            let worktree = Path::new(&path).to_path_buf();
            let git_dir = worktree.join(".git");
            let config = Ini::new();

            let mut me = Repository {
                worktree,
                git_dir,
                config,
            };

            if !(force || me.git_dir.is_dir()) {
                panic!("Not a Git repository {}", path)
            }

            // Read the config
            let repo_config = me.repo_file(&["config"], None);
            if repo_config.exists() {
                me.config.load(repo_config).unwrap();
            } else if !force {
                panic!("Configuration file missing")
            }

            if !force {
                let version = me.config.get("core", "repositoryformatversion");
                if version != Some(String::from("0")) {
                    panic!("Unsupported repositoryformatversion {:?}", version);
                }
            }

            return me;
        }

        /// Create a new repository at path
        pub fn repo_create(path: String) -> Result<(), io::Error> {
            let mut repo = Repository::new(&path, true);

            // Make sure the path either doesn't exist, or is empty
            if repo.worktree.exists() {
                if !repo.worktree.is_dir() {
                    panic!("{} is not a directory!", path);
                }

                let is_empty = repo.worktree.read_dir()?.next().is_none();
                if !is_empty {
                    panic!("{} is not empty!", path);
                }
            } else {
                create_dir_all(&repo.worktree).unwrap();
            }

            repo.repo_dir(&["branches"], Some(true)).unwrap();
            repo.repo_dir(&["objects"], Some(true)).unwrap();
            repo.repo_dir(&["refs", "tags"], Some(true)).unwrap();
            repo.repo_dir(&["refs", "heads"], Some(true)).unwrap();

            // .git/description
            let mut f = File::create(repo.repo_file(&["description"], None)).unwrap();
            writeln!(
                f,
                "Unnamed repository: edit this file 'description' to name the repository."
            )
            .unwrap();

            // .git/HEAD
            f = File::create(repo.repo_file(&["HEAD"], None)).unwrap();
            writeln!(f, "ref: refs/heads/master").unwrap();

            repo.config = Repository::repo_default_config();
            repo.config
                .write(repo.repo_file(&["config"], None))
                .unwrap();

            return Ok(());
        }

        fn repo_default_config() -> Ini {
            let mut default = IniDefault::default();
            default.comment_symbols = vec!['#'];
            default.delimiters = vec!['='];
            default.case_sensitive = true;
            default.multiline = false;

            let mut config = Ini::new_from_defaults(default.clone());
            config.setstr("core", "repositoryformatversion", Some("0"));
            config.setstr("core", "filemode", Some("false"));
            config.setstr("core", "bare", Some("false"));

            return config;
        }

        /// Find a repository directory
        ///
        /// Recurse up the directory tree, all the way to /, until a .git directory is found
        pub fn repo_find(
            path: String,
            required: Option<bool>,
        ) -> Result<Option<Repository>, io::Error> {
            let my_path = Path::new(&path).canonicalize()?;

            let is_dir = my_path.join(".git").is_dir();
            if is_dir {
                return Ok(Some(Repository::new(&path, false)));
            }

            // If we haven't returned, recurse in parent
            let parent = my_path.parent();

            return match parent {
                // Bottom case, the root directory, is represented by None
                None => {
                    let required = match required {
                        Some(v) => v,
                        None => true,
                    };

                    if required {
                        panic!("No git directory.")
                    } else {
                        return Ok(None);
                    }
                }
                Some(parent) => {
                    Repository::repo_find(String::from(parent.to_str().unwrap()), required)
                }
            };
        }

        /// Computes a path under the Repository's gitdir
        fn repo_path(&self, path_segments: &[&str]) -> PathBuf {
            return self
                .git_dir
                .join(&path_segments.join(String::from(MAIN_SEPARATOR).as_str()));
        }

        ///Same as repo_path, but create dirname(path_segments) if absent.  For
        ///example, r.repo_file(&["refs", "remotes", "origin", "HEAD"]) will create
        ///.git/refs/remotes/origin.
        pub fn repo_file(&self, path_segments: &[&str], mkdir: Option<bool>) -> PathBuf {
            return match self.repo_dir(&path_segments[0..path_segments.len() - 1], mkdir) {
                Ok(_) => self.repo_path(&path_segments),
                Err(e) => panic!("{}", e),
            };
        }

        /// Creates the directory and all parents if absent
        fn repo_dir(
            &self,
            path_segments: &[&str],
            mkdir: Option<bool>,
        ) -> Result<PathBuf, io::Error> {
            let repo_path = self.repo_path(&path_segments);

            if repo_path.exists() {
                if repo_path.is_dir() {
                    return Ok(repo_path);
                } else {
                    panic!("Not a directory {}", repo_path.display())
                }
            }

            let result = match mkdir {
                Some(_) => create_dir_all(&repo_path),
                None => Ok(()),
            };

            return match result {
                Ok(()) => Ok(repo_path),
                Err(e) => Err(e),
            };
        }

        /// Read object object_id from Git repository repo.  Return a
        /// GitObject.
        pub(crate) fn read_object(&self, sha: String) -> Result<GitObjectData, std::io::Error> {
            let path = self.repo_file(&["objects", &sha[0..2], &sha[2..]], None);
            let f = File::open(path)?;

            let mut raw = Vec::new();
            ZlibDecoder::new(f).read_to_end(&mut raw)?;

            let x = raw.iter().position(|b| b == &b' ').unwrap();
            let object_type = String::from_utf8(raw[0..x].to_vec()).unwrap();

            let y = raw.iter().position(|b| b == &b'\x00').unwrap();
            let size = String::from_utf8(raw[x..y].to_vec())
                .unwrap()
                .parse::<usize>()
                .unwrap();

            if size != raw.len() - y - 1 {
                panic!("Malformed object {}: bad length", sha)
            }

            return Ok(GitObjectData(object_type, raw));
        }

        pub(crate) fn object_find(
            &self,
            name: String,
            _fmt: Option<String>,
            _follow: Option<bool>,
        ) -> String {
            return name;
        }
    }
}
