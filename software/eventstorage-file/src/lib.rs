extern crate failure;
#[macro_use]
extern crate failure_derive;

use cqrs_es::*;

pub struct FileEventStorage<A: Aggregate> {
    
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
