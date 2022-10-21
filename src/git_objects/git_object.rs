use std::{
    fs::File,
    io::{Read, Write},
};

use crypto::{digest::Digest, sha1::Sha1};
use flate2::read::ZlibDecoder;

use crate::repository::repository::Repository;

struct GitObject {
    repo: Option<Repository>,
}

pub(crate) struct GitObjectData(String, Vec<u8>);

impl GitObject {
    pub fn new(repo: Repository, data: Option<GitObjectData>) -> GitObject {
        let mut object = GitObject { repo: Some(repo) };
        match data {
            Some(data) => object.deserialize(data),
            None => {}
        }

        return object;
    }

    /// Read object object_id from Git repository repo.  Return a
    /// GitObject.
    fn read_object(repo: Repository, sha: String) -> Result<GitObjectData, std::io::Error> {
        let path = repo.repo_file(&["objects", &sha[0..2], &sha[2..]], None);
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

    fn write_object(obj: GitObject, actually_write: Option<bool>) -> String {
        let GitObjectData(fmt, data_vec) = obj.serialize();

        let data_string = String::from_utf8(data_vec.to_vec()).unwrap();
        let data_length = data_string.len();

        let result = [
            fmt.as_bytes(),
            &[b' '],
            data_length.to_string().as_bytes(),
            &[b'\x00'],
            data_string.as_bytes(),
        ]
        .concat();

        let mut sha = Sha1::new();
        sha.input(&result);
        let hash = sha.result_str();

        match actually_write {
            Some(false) => {}
            _ => {
                let path = obj
                    .repo
                    .expect("")
                    .repo_file(&["objects", &hash[0..2], &hash[2..]], Some(true));
                let f = File::create(path).unwrap();
                ZlibDecoder::new(f).write_all(&result).unwrap();
            }
        };

        return hash;
    }

    fn object_find(
        repo: Repository,
        name: String,
        fmt: Option<String>,
        follow: Option<bool>,
    ) -> String {
        return name;
    }
}

pub(crate) trait GitSerDe {
    fn serialize(&self) -> &GitObjectData;
    fn deserialize(&mut self, data: GitObjectData);
}

impl GitSerDe for GitObject {
    fn serialize(&self) -> &GitObjectData {
        todo!()
    }

    fn deserialize(&mut self, data: GitObjectData) {
        todo!()
    }
}
