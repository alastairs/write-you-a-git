use std::{any::Any, collections::BTreeMap};

use crate::{git_objects::git_object::GitObjectData, repository::repository::Repository};

use super::git_object::GitSerDe;

pub(crate) struct Commit {
    repo: Option<Repository>,
    kvlm: BTreeMap<String, Vec<String>>,
}

impl Commit {
    pub(crate) fn has_parent(&self) -> bool {
        return self.kvlm.contains_key("parent");
    }

    pub(crate) fn parents(&self) -> &Vec<String> {
        return self.kvlm.get("parent").unwrap();
    }
}

impl GitSerDe for Commit {
    fn new(repo: Option<Repository>, data: GitObjectData) -> Commit {
        let mut commit = Commit {
            repo: repo,
            kvlm: BTreeMap::new(),
        };

        commit.deserialize(data);

        return commit;
    }

    fn serialize(&self) -> GitObjectData {
        let GitObjectData(_, data) = GitObjectData::kvlm_serialize(&self.kvlm);
        return GitObjectData("commit".to_string(), data);
    }

    fn deserialize(&mut self, data: GitObjectData) {
        self.kvlm = data
            .kvlm_parse(None, None)
            .expect("Could not parse the kvlm object.");
    }

    fn get_repo(&self) -> &Repository {
        return self.repo.as_ref().expect("No repo set");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
