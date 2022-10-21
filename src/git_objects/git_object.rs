use std::{fs::File, io::Write};

use crypto::{digest::Digest, sha1::Sha1};
use flate2::read::ZlibDecoder;

use crate::repository::repository::Repository;

pub(crate) struct GitObject {
    repo: Option<Repository>,
}

pub(crate) struct GitObjectData(pub String, pub Vec<u8>);

impl GitObject {
    pub(crate) fn new(repo: Option<Repository>, data: Option<GitObjectData>) -> GitObject {
        let mut object = GitObject { repo };

        match data {
            Some(data) => object.deserialize(data),
            None => {}
        }

        return object;
    }

    pub(crate) fn write_object(obj: GitObject, actually_write: Option<bool>) -> String {
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
}

pub(crate) trait GitSerDe {
    fn serialize(&self) -> &GitObjectData;
    fn deserialize(&mut self, data: GitObjectData);
}

impl GitSerDe for GitObject {
    fn serialize(&self) -> &GitObjectData {
        todo!()
    }

    fn deserialize(&mut self, _: GitObjectData) {
        todo!()
    }
}
