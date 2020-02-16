use crate::{Command, Event};

pub trait Aggregate: Default + Clone {
    type Event: Event<Self>;
    type Command: Command<Self>;

    // `/` で区切ることで階層化することを想定
    fn type_name() -> &'static str;
}
