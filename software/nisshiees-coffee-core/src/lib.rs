extern crate chrono;
extern crate failure;
extern crate uuid;
#[macro_use]
extern crate failure_derive;

pub mod canister_list;
pub mod seller;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Brand(pub String);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Roast(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Gram(pub u32);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
