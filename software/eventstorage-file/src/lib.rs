extern crate failure;
#[macro_use]
extern crate failure_derive;

use cqrs_es::*;
use std::path::{Path, PathBuf};
use std::io;
use std::fs;
use failure::_core::marker::PhantomData;
use std::io::Write;

pub struct FileEventStorage<A: Aggregate> {
    dir: PathBuf,
    phantom: PhantomData<A>,
}

impl <A: Aggregate> FileEventStorage<A> {
    pub fn new(root_path: &Path) -> Result<Self, io::Error> {
        let mut aggregate_dir = root_path.to_owned();
        A::type_name().split('/').for_each(|e| aggregate_dir.push(e));

        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(aggregate_dir.as_path())?;

        Ok(FileEventStorage {
            dir: aggregate_dir,
            phantom: PhantomData,
        })
    }
}

#[derive(Fail, Debug)]
pub enum FileEventStorageError {
    #[fail(display = "IO error: {}", _0)]
    Io(io::Error),
}

impl<A: Aggregate> EventStorage<A> for FileEventStorage<A> {
    type Events = Option<A::Event>;
    type Error = FileEventStorageError;

    fn insert(&mut self, id: A::Id, event: A::Event) -> Result<(), Self::Error> {
        let mut file_path = self.dir.clone();
        file_path.push(id.to_string());
        let mut file = fs::OpenOptions::new().create(true).append(true).open(file_path)?;
        file.write_all(hoge)?;
        Ok(())
    }

    fn read(&self, id: <A as Aggregate>::Id) -> Result<Self::Events, Self::Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
