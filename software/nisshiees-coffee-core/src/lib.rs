extern crate chrono;
extern crate failure;
extern crate serde;
extern crate uuid;
#[macro_use]
extern crate failure_derive;

pub mod canister_list;
pub mod seller;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Brand(pub String);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Roast(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gram(pub u32);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
