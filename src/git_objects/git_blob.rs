use std::any::Any;

use crate::{git_objects::git_object::GitObjectData, repository::repository::Repository};

use super::git_object::GitSerDe;

pub(crate) struct Blob {
    repo: Option<Repository>,
    data: GitObjectData,
}

impl GitSerDe for Blob {
    fn serialize(&self) -> GitObjectData {
        let GitObjectData(object_type, data) = &self.data;
        return GitObjectData(object_type.clone(), data.clone());
    }

    fn deserialize(&mut self, data: GitObjectData) {
        self.data = data;
    }

    fn get_repo(&self) -> &Repository {
        return self.repo.as_ref().expect("No repo set");
    }

    fn new(repo: Option<Repository>, data: GitObjectData) -> Blob {
        let mut blob = Blob {
            repo,
            data: GitObjectData("blob".to_string(), vec![]),
        };

        blob.deserialize(data);
        return blob;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
