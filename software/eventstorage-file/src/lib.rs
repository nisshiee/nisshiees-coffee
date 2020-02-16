extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate serde;
extern crate serde_json;

use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use cqrs_es::store::*;
use cqrs_es::version::*;
use cqrs_es::*;
use failure::_core::marker::PhantomData;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

pub struct FileEventStorage<A, E>
where
    A: Aggregate<Event = E> + Serialize,
    E: Event<A> + Serialize,
{
    dir: PathBuf,
    phantom: PhantomData<A>,
    //    projectors: Vec<&'a mut dyn Projector<A>>,
}

impl<A, E> FileEventStorage<A, E>
where
    A: Aggregate<Event = E> + Serialize,
    E: Event<A> + Serialize,
{
    pub fn new<P>(root_path: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let mut aggregate_dir = root_path.as_ref().to_owned();
        A::type_name()
            .split('/')
            .for_each(|e| aggregate_dir.push(e));

        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(aggregate_dir.as_path())?;

        Ok(FileEventStorage {
            dir: aggregate_dir,
            phantom: PhantomData,
            //            projectors: Vec::new(),
        })
    }

    fn file_path(&self, id: Id<A>) -> PathBuf {
        let mut file_path = self.dir.clone();
        file_path.push(id.to_string());
        file_path
    }
}

//impl<'a, A: Aggregate> FileEventStorage<'a, A> {
//    pub fn add_projector<P: Projector<A>>(&mut self, projector: &'a mut P) {
//        self.projectors.push(projector)
//    }
//}

#[derive(Fail, Debug)]
pub enum FileEventStorageError {
    #[fail(display = "IO error: {}", _0)]
    Io(#[fail(cause)] io::Error),
    #[fail(display = "JSON error: {}", _0)]
    Json(#[fail(cause)] serde_json::Error),
}

impl EventStorageError for FileEventStorageError {}

impl From<io::Error> for FileEventStorageError {
    fn from(e: io::Error) -> Self {
        FileEventStorageError::Io(e)
    }
}

impl From<serde_json::Error> for FileEventStorageError {
    fn from(e: serde_json::Error) -> Self {
        use serde_json::error::Category;
        match e.classify() {
            Category::Io => FileEventStorageError::Io(e.into()),
            _ => FileEventStorageError::Json(e),
        }
    }
}

impl<A, E> EventStorage<A> for FileEventStorage<A, E>
where
    A: Aggregate<Event = E> + Serialize + DeserializeOwned,
    E: Event<A> + Serialize + DeserializeOwned,
{
    type Events = Vec<VersionedEvent<A>>;
    type Error = FileEventStorageError;

    fn insert(&mut self, id: Id<A>, event: VersionedEvent<A>) -> Result<(), Self::Error> {
        let file_path = self.file_path(id);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        let json = serde_json::to_string(&event)?;
        file.write_all(json.as_bytes())?;
        file.write_all(&[0x0A])?;

        //        self.projectors
        //            .iter_mut()
        //            .for_each(|p| p.project(id, &event));

        Ok(())
    }

    fn read(&self, id: Id<A>) -> Result<Self::Events, Self::Error> {
        let file_path = self.file_path(id);

        if let Ok(metadata) = fs::metadata(&file_path) {
            if !metadata.is_file() {
                return Ok(Vec::new());
            }
            if metadata.len() == 0 {
                return Ok(Vec::new());
            }
        } else {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&file_path)?;
        let file = io::BufReader::new(file);
        Deserializer::from_reader(file)
            .into_iter()
            .try_fold(Vec::new(), |mut a, e| {
                let event = e?;
                a.push(event);
                Ok(a)
            })
    }

    //    fn insert(&mut self, id: A::Id, event: A::Event) -> Result<(), Self::Error> {
    //        let file_path = self.file_path(id);
    //        let mut file = fs::OpenOptions::new()
    //            .create(true)
    //            .append(true)
    //            .open(file_path)?;
    //        let json = serde_json::to_string(&event)?;
    //        file.write_all(json.as_bytes())?;
    //        file.write_all(&[0x0A])?;
    //
    //        self.projectors
    //            .iter_mut()
    //            .for_each(|p| p.project(id, &event));
    //
    //        Ok(())
    //    }
    //
    //    fn read(&self, id: <A as Aggregate>::Id) -> Result<Self::Events, Self::Error> {
    //        let file_path = self.file_path(id);
    //
    //        if let Ok(metadata) = fs::metadata(&file_path) {
    //            if !metadata.is_file() {
    //                return Ok(Vec::new());
    //            }
    //            if metadata.len() == 0 {
    //                return Ok(Vec::new());
    //            }
    //        } else {
    //            return Ok(Vec::new());
    //        }
    //
    //        let file = fs::File::open(&file_path)?;
    //        let file = io::BufReader::new(file);
    //        file.lines().try_fold(Vec::new(), |mut a, e| {
    //            let line = e?;
    //            let event = serde_json::from_str(&line)?;
    //            a.push(event);
    //            Ok(a)
    //        })
    //    }
}

#[cfg(test)]
mod tests {
    mod test_aggregate;

    use crate::*;
    use cqrs_es::*;
    use test_aggregate::*;

    //    #[derive(Debug)]
    //    struct TestProjector();
    //
    //    impl Projector<TestAggregate> for TestProjector {
    //        fn project(&mut self, _id: TestAggregateId, event: &TestEvent) {
    //            println!("{:?}", event);
    //        }
    //    }

    #[test]
    fn it_works() {
        let mut storage = FileEventStorage::<TestAggregate, TestEvent>::new("test").unwrap();
        //        let mut projector = TestProjector();
        //        storage.add_projector(&mut projector);
        let id = Id::<TestAggregate>::new();
        storage
            .insert(
                id,
                VersionedEvent {
                    version: Version(1),
                    event: TestEvent::Increased,
                },
            )
            .unwrap();
        storage
            .insert(
                id,
                VersionedEvent {
                    version: Version(2),
                    event: TestEvent::Increased,
                },
            )
            .unwrap();

        let events = storage.read(id).unwrap();
        println!("{:?}", events);
        assert_eq!(2 + 2, 4);
    }
}
