use crate::repository::repository::Repository;

use super::git_object::{GitObjectData, GitSerDe};

pub(crate) struct Tree {
    items: Vec<Leaf>,
    repo: Option<Repository>,
}

impl GitSerDe for Tree {
    fn new(repo: Option<Repository>, data: GitObjectData) -> Self
    where
        Self: Sized,
    {
        let mut tree = Tree {
            repo,
            items: Vec::new(),
        };
        tree.deserialize(data);
        return tree;
    }

    fn serialize(&self) -> GitObjectData {
        return tree_serialize(self);
    }

    fn deserialize(&mut self, data: GitObjectData) {
        self.items = tree_parse(data);
    }

    fn get_repo(&self) -> &Repository {
        return self.repo.as_ref().expect("No repo set");
    }
}

/// A leaf in git's tree is a triple of:
/// (file mode, path, sha1)
#[derive(Debug)]
struct Leaf(String, String, String);

fn tree_serialize(tree: &Tree) -> GitObjectData {
    let mut serialized = String::new();
    for leaf in &tree.items {
        serialized.push_str(format!("{} {}\x00{}", leaf.0, leaf.1, leaf.2).as_str());
    }

    return GitObjectData(String::from("tree"), serialized.as_bytes().to_vec());
}

fn tree_parse(raw: GitObjectData) -> Vec<Leaf> {
    let mut pos = 0;
    let max = raw.1.len();
    let mut tree = Vec::new();

    while pos < max {
        if let Some((new_pos, leaf)) = tree_parse_one(&raw.1, Some(pos)) {
            tree.push(leaf);
            pos = new_pos;
        }
    }

    return tree;
}

fn tree_parse_one(raw: &Vec<u8>, start: Option<usize>) -> Option<(usize, Leaf)> {
    let start = match start {
        None => 0,
        Some(start) => start,
    };

    // Find the space terminating the file mode value
    let x = raw.iter().skip(start).position(|b| b == &b' ')? + start;
    assert!(x - start == 5 || x - start == 6);

    // Read the file mode
    let mode = &raw[start..x];

    // Find the NUL terminator of the path value and read the path
    let y = raw.iter().skip(x).position(|b| b == &b'\x00')? + x;
    let path = &raw[x + 1..y];

    // Read the SHA and convert it to a hex string
    let sha = &raw[y + 1..y + 21];

    return Some((
        y + 21,
        Leaf(
            String::from_utf8(mode.to_vec()).unwrap(),
            String::from_utf8(path.to_vec()).unwrap(),
            String::from_utf8(sha.to_vec()).unwrap(),
        ),
    ));
}
