extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate serde;
extern crate serde_json;

use cqrs_es::*;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub struct FileEventStorage<'a, A: Aggregate> {
    dir: PathBuf,
    projectors: Vec<&'a mut dyn Projector<A>>,
}

impl<A: Aggregate> FileEventStorage<'_, A> {
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
            projectors: Vec::new(),
        })
    }

    fn file_path(&self, id: A::Id) -> PathBuf {
        let mut file_path = self.dir.clone();
        file_path.push(id.to_string());
        file_path
    }
}

impl<'a, A: Aggregate> FileEventStorage<'a, A> {
    pub fn add_projector<P: Projector<A>>(&mut self, projector: &'a mut P) {
        self.projectors.push(projector)
    }
}

#[derive(Fail, Debug)]
pub enum FileEventStorageError {
    #[fail(display = "IO error: {}", _0)]
    Io(#[fail(cause)] io::Error),
    #[fail(display = "JSON error: {}", _0)]
    Json(#[fail(cause)] serde_json::Error),
}

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

impl<A: Aggregate> EventStorage<A> for FileEventStorage<'_, A> {
    type Events = Vec<A::Event>;
    type Error = FileEventStorageError;

    fn insert(&mut self, id: A::Id, event: A::Event) -> Result<(), Self::Error> {
        let file_path = self.file_path(id);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        let json = serde_json::to_string(&event)?;
        file.write_all(json.as_bytes())?;
        file.write_all(&[0x0A])?;

        self.projectors
            .iter_mut()
            .for_each(|p| p.project(id, &event));

        Ok(())
    }

    fn read(&self, id: <A as Aggregate>::Id) -> Result<Self::Events, Self::Error> {
        let file_path = self.file_path(id);

        if let Ok(metadata) = fs::metadata(&file_path) {
            if !metadata.is_file() {
                return Ok(Vec::new());
            }
            if metadata.len() == 0 {
                return Ok(Vec::new());
            }
        } else {
            return Ok(Vec::new())
        }

        let file = fs::File::open(&file_path)?;
        let file = io::BufReader::new(file);
        file.lines().try_fold(Vec::new(), |mut a, e| {
            let line = e?;
            let event = serde_json::from_str(&line)?;
            a.push(event);
            Ok(a)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::FileEventStorage;
    use cqrs_es::{Aggregate, AggregateId, Command, Event, EventStorage, Projector};
    use serde::{Deserialize, Serialize};

    struct TestAggregate(u64);

    impl Default for TestAggregate {
        fn default() -> Self {
            TestAggregate(0)
        }
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    struct TestAggregateId(pub u64);

    impl ToString for TestAggregateId {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }

    impl AggregateId<TestAggregate> for TestAggregateId {}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum TestEvent {
        E1,
        E2(u64),
        E3 { x: u64, y: String, z: bool },
    }

    impl Event<TestAggregate> for TestEvent {
        fn apply_to(self, _aggregate: &mut TestAggregate) {
            unimplemented!()
        }
    }

    #[derive(Debug)]
    enum TestCommand {}

    #[derive(Fail, Debug)]
    #[fail(display = "test")]
    struct TestCommandError();

    impl Command<TestAggregate> for TestCommand {
        type Events = Option<TestEvent>;
        type Error = TestCommandError;

        fn execute_on(self, _aggregate: &TestAggregate) -> Result<Self::Events, Self::Error> {
            unimplemented!()
        }
    }

    impl Aggregate for TestAggregate {
        type Id = TestAggregateId;
        type Event = TestEvent;
        type Command = TestCommand;

        fn type_name() -> &'static str {
            "test"
        }
    }

    #[derive(Debug)]
    struct TestProjector();

    impl Projector<TestAggregate> for TestProjector {
        fn project(&mut self, _id: TestAggregateId, event: &TestEvent) {
            println!("{:?}", event);
        }
    }

    #[test]
    fn it_works() {
        let mut storage = FileEventStorage::<TestAggregate>::new("test").unwrap();
        let mut projector = TestProjector();
        storage.add_projector(&mut projector);
        let id = TestAggregateId(1);
        storage.insert(id, TestEvent::E1).unwrap();
        storage.insert(id, TestEvent::E2(123)).unwrap();
        storage
            .insert(
                id,
                TestEvent::E3 {
                    x: 456,
                    y: "hoge".to_string(),
                    z: true,
                },
            )
            .unwrap();
        let events = storage.read(id).unwrap();
        println!("{:?}", events);
        assert_eq!(2 + 2, 4);
    }
}
