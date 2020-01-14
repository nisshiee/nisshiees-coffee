use crate::{Brand, Gram, Roast};
use chrono::NaiveDate;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PurchaseLogAggregate {
    Uninitialized,
    Created {
        brand: Brand,
        roast: Roast,
        gram: Gram,
        date: NaiveDate,
    },
}
