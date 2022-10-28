use std::{any::Any, collections::BTreeMap, fs::File, io::Write, string::FromUtf8Error};

use crypto::{digest::Digest, sha1::Sha1};
use flate2::read::ZlibDecoder;

use crate::{
    git_objects::{git_blob::Blob, git_commit::Commit, git_tree::Tree},
    repository::repository::Repository,
};

pub(crate) struct GitObject {}

#[derive(Debug)]
pub(crate) struct GitObjectData(pub String, pub Vec<u8>);

impl GitObject {
    pub(crate) fn new(repo: Option<Repository>, data: Option<GitObjectData>) -> Box<dyn GitSerDe> {
        match data {
            Some(data) => {
                let GitObjectData(object_type, data) = data;
                let boxed: Box<dyn GitSerDe> = match object_type.as_str() {
                    "blob" => Box::new(Blob::new(repo, GitObjectData(object_type, data))),
                    "commit" => Box::new(Commit::new(repo, GitObjectData(object_type, data))),
                    "tree" => Box::new(Tree::new(repo, GitObjectData(object_type, data))),
                    _ => panic!(),
                };

                return boxed;
            }
            None => panic!(),
        }
    }

    pub(crate) fn write_object(obj: Box<dyn GitSerDe>, actually_write: Option<bool>) -> String {
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
                    .get_repo()
                    .repo_file(&["objects", &hash[0..2], &hash[2..]], Some(true));
                let f = File::create(path).unwrap();
                ZlibDecoder::new(f).write_all(&result).unwrap();
            }
        };

        return hash;
    }
}

impl GitObjectData {
    /// Parser for more complex git objects, Key-Value List with Message.
    pub(crate) fn kvlm_parse(
        &self,
        start: Option<usize>,
        dct: Option<BTreeMap<String, Vec<String>>>,
    ) -> Result<BTreeMap<String, Vec<String>>, FromUtf8Error> {
        let mut dict = match dct {
            None => BTreeMap::new(),
            Some(d) => d,
        };

        let start = match start {
            None => 0,
            Some(s) => s,
        };

        let GitObjectData(_, data_vec) = self;
        let line: Vec<u8> = data_vec.iter().skip(start).cloned().collect();
        let mut _spc = line.iter().position(|b| b == &b' ');
        let spc = *_spc.get_or_insert(start) + start;
        let mut _nl = line.iter().position(|b| b == &b'\n');
        let nl = *_nl.get_or_insert(start) + start;

        // If space appears before a newline, we have a keyword and there's
        // still more to parse.

        // Base case
        // ---------
        //
        // If newline appears first (or there's no space at all, in which case position() returns None), we
        // assume a blank line. A blank line means the remainder of the data is the message.
        let parsing_message = nl < spc;

        if parsing_message {
            assert!(nl == start);
            dict.insert(
                String::from(""),
                vec![String::from_utf8(data_vec[start + 1..].to_vec())?],
            );

            return Ok(dict);
        }

        // Recursive case
        // --------------
        //
        // We read a key-value pair and recurse for the next
        let key = String::from_utf8(data_vec[start..spc].to_vec())?;

        // Find the end of the value. Continuation lines begin with a space, so we loop until we find a "\n"
        // not followed by a space.
        let mut end = start;

        loop {
            end = data_vec
                .iter()
                .skip(end + 1)
                .position(|b| b == &b'\n')
                .unwrap()
                + end
                + 1;

            if data_vec[end + 1] != b' ' {
                break;
            }
        }

        // Grab the value
        // Also, drop the leading space on continuation lines
        let value = String::from_utf8(data_vec[spc + 1..end].to_vec())?.replace("\n ", "\n");

        // Don't overwrite existing data contents
        dict.entry(key)
            .and_modify(|v| v.extend_from_slice(&[value.clone()]))
            .or_insert(vec![value.clone()]);

        return self.kvlm_parse(Some(end + 1), Some(dict));
    }

    pub(crate) fn kvlm_serialize(kvlm: &BTreeMap<String, Vec<String>>) -> GitObjectData {
        let mut str = String::from("");

        for key in kvlm.keys() {
            // Skip the message itself
            if key == "" {
                continue;
            }

            let val = kvlm.get(key).unwrap();
            for v in val {
                str += &(key.to_owned() + &" " + &(v.replace("\n", "\n ")) + "\n")
            }
        }

        // Append message
        str += &("\n".to_owned() + kvlm.get("").unwrap()[0].as_str());

        return GitObjectData("".to_string(), str.as_bytes().to_vec());
    }
}

pub(crate) trait GitSerDe {
    /// Create a new GitObject for serde
    fn new(repo: Option<Repository>, data: GitObjectData) -> Self
    where
        Self: Sized;

    /// Serialise the object
    fn serialize(&self) -> GitObjectData;

    /// Deserialise the object
    fn deserialize(&mut self, data: GitObjectData);

    /// Obtain the wrapped Repository object
    fn get_repo(&self) -> &Repository;

    fn get_data(&self) -> GitObjectData {
        return self.serialize();
    }

    fn as_any(&self) -> &dyn Any;
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT_EXAMPLE: &str = "tree 4f0ae14e719a707c91ad1885e38bd18e7d214629
parent 5f350c20eb579d51d0a3b1fafc8fce3f26d9d61a
author Alastair Smith <github@alastairsmith.me.uk> 1666471801 +0100
committer Alastair Smith <github@alastairsmith.me.uk> 1666471801 +0100
gpgsig -----BEGIN PGP SIGNATURE-----
 
 iQJPBAABCgA5FiEEcAUvar0ofFepLhfUJfqdqODN8YIFAmNUV/YbHGdpdGh1YkBh
 bGFzdGFpcnNtaXRoLm1lLnVrAAoJECX6najgzfGC/xYP/Rw37i0K2168upfp2OLO
 NCKyqXgcRbF8YgzajH4YAK/JIRko8/I3haINQofIRRop0gw4Qih3RvEQ5bBXh2xf
 Y1w9WyaDqwBz09zMKE5GMsZQ4xZ7HjqLq/iOXhwx7vEwAg77WrYJgqgzuC8qYHCr
 CIm0Ni7rJSL5Rn2chuxzOs3RdwMNSSnmMgbvYkASHuBAAMSiALT2XOZM8zK1smx4
 RpI2xc7cS/2E+2OdBgwF4fg64b/vWtRcNZpfF31L3VVi1KlObtgyAlR/gMLxA/G9
 PNn1G47rgb09lAYjz3akwl3sZyeawDZ6sJPONYKl+hIq+qFKDYvh4wL98zDZBkrc
 LjvXCZAjionPN/f54sh+71Ec5r6j0y0I3nXXPHCGyl+tUlj7Rkm+vTPaa1FpfKJA
 pQQt2cnItDi5rixteGqGlk2Kmpi9LkQUSOFu9a5VV0I7zcbaT6sNXDIJZNwCYSSc
 Q4dEpZSV1EYePUl9GWYWXCvf0inqFaGGOI2CTaHXvRE0c9llhgbFNTkIpCm1G5pL
 7H7T4FIkxLPUHOGTsfUaZ0hT0UVf48eO0HUxDr6jjKDdPSMZMMcQVy1eaAmRehZO
 73ezq3oPt/gPCssAk+vbriGjp9hFY7QwrmiZnivtM62yTqVxPHIixgjwOd87Q8h4
 Dsu+R6iJ+fYrbCCZSELWGA4Z
 =SBeJ
 -----END PGP SIGNATURE-----

Implement Commit object type

Well, this forced me to resolve the polymorphism of the GitObject type,
which I think has now been handled via traits and Box. Let's see if wyag
commit works...
";

    #[test]
    fn test_can_parse_tree_key_value() -> Result<(), FromUtf8Error> {
        let commit = COMMIT_EXAMPLE;
        let commit_object = GitObjectData(
            "commit".to_string(),
            String::from(commit).as_bytes().to_vec(),
        );

        let parsed = commit_object.kvlm_parse(None, None)?;
        assert_eq!(
            "4f0ae14e719a707c91ad1885e38bd18e7d214629",
            parsed.get("tree").expect("Tree is empty")[0],
            "{:?}",
            parsed.get("tree"),
        );

        return Ok(());
    }

    #[test]
    fn test_can_parse_parent_key_value() -> Result<(), FromUtf8Error> {
        let commit = COMMIT_EXAMPLE;
        let commit_object = GitObjectData(
            "commit".to_string(),
            String::from(commit).as_bytes().to_vec(),
        );

        let parsed = commit_object.kvlm_parse(None, None)?;
        assert_eq!(
            "5f350c20eb579d51d0a3b1fafc8fce3f26d9d61a",
            parsed.get("parent").expect("Parents is empty")[0]
        );

        return Ok(());
    }

    #[test]
    fn test_can_parse_author_key_value() -> Result<(), FromUtf8Error> {
        let commit = COMMIT_EXAMPLE;
        let commit_object = GitObjectData(
            "commit".to_string(),
            String::from(commit).as_bytes().to_vec(),
        );

        let parsed = commit_object.kvlm_parse(None, None)?;
        assert_eq!(
            "Alastair Smith <github@alastairsmith.me.uk> 1666471801 +0100",
            parsed.get("author").expect("Author is empty")[0]
        );

        return Ok(());
    }

    #[test]
    fn test_can_parse_committer_key_value() -> Result<(), FromUtf8Error> {
        let commit = COMMIT_EXAMPLE;
        let commit_object = GitObjectData(
            "commit".to_string(),
            String::from(commit).as_bytes().to_vec(),
        );

        let parsed = commit_object.kvlm_parse(None, None)?;
        assert_eq!(
            "Alastair Smith <github@alastairsmith.me.uk> 1666471801 +0100",
            parsed.get("committer").expect("Committer is empty")[0]
        );

        return Ok(());
    }

    #[test]
    fn test_can_parse_commit_message() -> Result<(), FromUtf8Error> {
        let commit = COMMIT_EXAMPLE;
        let commit_object = GitObjectData(
            "commit".to_string(),
            String::from(commit).as_bytes().to_vec(),
        );

        let parsed = commit_object.kvlm_parse(None, None)?;
        assert_eq!(
            "Implement Commit object type

Well, this forced me to resolve the polymorphism of the GitObject type,
which I think has now been handled via traits and Box. Let's see if wyag
commit works...
",
            parsed.get("").expect("Message is empty")[0]
        );

        return Ok(());
    }

    #[test]
    fn test_can_parse_gpgsig_key_value() -> Result<(), FromUtf8Error> {
        let commit = COMMIT_EXAMPLE;
        let commit_object = GitObjectData(
            "commit".to_string(),
            String::from(commit).as_bytes().to_vec(),
        );

        let parsed = commit_object.kvlm_parse(None, None)?;

        assert_eq!(
            "-----BEGIN PGP SIGNATURE-----

iQJPBAABCgA5FiEEcAUvar0ofFepLhfUJfqdqODN8YIFAmNUV/YbHGdpdGh1YkBh
bGFzdGFpcnNtaXRoLm1lLnVrAAoJECX6najgzfGC/xYP/Rw37i0K2168upfp2OLO
NCKyqXgcRbF8YgzajH4YAK/JIRko8/I3haINQofIRRop0gw4Qih3RvEQ5bBXh2xf
Y1w9WyaDqwBz09zMKE5GMsZQ4xZ7HjqLq/iOXhwx7vEwAg77WrYJgqgzuC8qYHCr
CIm0Ni7rJSL5Rn2chuxzOs3RdwMNSSnmMgbvYkASHuBAAMSiALT2XOZM8zK1smx4
RpI2xc7cS/2E+2OdBgwF4fg64b/vWtRcNZpfF31L3VVi1KlObtgyAlR/gMLxA/G9
PNn1G47rgb09lAYjz3akwl3sZyeawDZ6sJPONYKl+hIq+qFKDYvh4wL98zDZBkrc
LjvXCZAjionPN/f54sh+71Ec5r6j0y0I3nXXPHCGyl+tUlj7Rkm+vTPaa1FpfKJA
pQQt2cnItDi5rixteGqGlk2Kmpi9LkQUSOFu9a5VV0I7zcbaT6sNXDIJZNwCYSSc
Q4dEpZSV1EYePUl9GWYWXCvf0inqFaGGOI2CTaHXvRE0c9llhgbFNTkIpCm1G5pL
7H7T4FIkxLPUHOGTsfUaZ0hT0UVf48eO0HUxDr6jjKDdPSMZMMcQVy1eaAmRehZO
73ezq3oPt/gPCssAk+vbriGjp9hFY7QwrmiZnivtM62yTqVxPHIixgjwOd87Q8h4
Dsu+R6iJ+fYrbCCZSELWGA4Z
=SBeJ
-----END PGP SIGNATURE-----",
            parsed.get("gpgsig").expect("gpgsig is empty")[0]
        );

        return Ok(());
    }

    #[test]
    fn test_can_round_trip_serialized_objects() -> Result<(), FromUtf8Error> {
        let commit = COMMIT_EXAMPLE;
        let commit_object = GitObjectData(
            "commit".to_string(),
            String::from(commit).as_bytes().to_vec(),
        );

        let parsed_src = commit_object.kvlm_parse(None, None)?;
        let serialized = GitObjectData::kvlm_serialize(&parsed_src);
        let parsed_serialized = serialized.kvlm_parse(None, None)?;

        assert_eq!(parsed_src, parsed_serialized);

        return Ok(());
    }
}
