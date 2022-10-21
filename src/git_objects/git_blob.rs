use crate::git_objects::git_object::{GitObjectData, GitSerDe};

struct Blob {
    data: GitObjectData,
}

impl GitSerDe for Blob {
    fn serialize(&self) -> &GitObjectData {
        let data = &self.data;
        return data;
    }

    fn deserialize(&mut self, data: GitObjectData) {
        self.data = data;
    }
}
