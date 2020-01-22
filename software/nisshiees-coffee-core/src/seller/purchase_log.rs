use crate::{Brand, Gram, Roast};
use chrono::NaiveDate;
use cqrs_es::{Aggregate, AggregateId, Command, Event};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum PurchaseLogAggregate {
    Uninitialized,
    Created {
        brand: Brand,
        roast: Roast,
        gram: Gram,
        date: NaiveDate,
    },
}

impl Default for PurchaseLogAggregate {
    fn default() -> Self {
        PurchaseLogAggregate::Uninitialized
    }
}

impl Aggregate for PurchaseLogAggregate {
    type Id = PurchaseLogAggregateId;
    type Event = PurchaseLogEvent;
    type Command = PurchaseLogCommand;

    fn type_name() -> &'static str {
        "seller/purchase_log"
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PurchaseLogAggregateId(pub Uuid);

impl ToString for PurchaseLogAggregateId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl AggregateId<PurchaseLogAggregate> for PurchaseLogAggregateId {}

#[derive(Debug, Clone)]
pub enum PurchaseLogEvent {
    Created {
        brand: Brand,
        roast: Roast,
        gram: Gram,
        date: NaiveDate,
    },
}

impl Event<PurchaseLogAggregate> for PurchaseLogEvent {
    fn apply_to(self, aggregate: &mut PurchaseLogAggregate) {
        match self {
            PurchaseLogEvent::Created {
                brand,
                roast,
                gram,
                date,
            } => {
                *aggregate = PurchaseLogAggregate::Created {
                    brand,
                    roast,
                    gram,
                    date,
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum PurchaseLogCommand {
    Create {
        brand: Brand,
        roast: Roast,
        gram: Gram,
        date: NaiveDate,
    },
}

#[derive(Fail, Debug)]
pub enum PurchaseLogCommandError {
    #[fail(display = "PurchaseLog already created")]
    AlreadyCreated,
}

impl Command<PurchaseLogAggregate> for PurchaseLogCommand {
    type Events = Option<PurchaseLogEvent>;
    type Error = PurchaseLogCommandError;

    fn execute_on(self, aggregate: &PurchaseLogAggregate) -> Result<Self::Events, Self::Error> {
        match self {
            PurchaseLogCommand::Create {
                brand,
                roast,
                gram,
                date,
            } => match aggregate {
                PurchaseLogAggregate::Uninitialized => Ok(Some(PurchaseLogEvent::Created {
                    brand,
                    roast,
                    gram,
                    date,
                })),
                _ => Err(PurchaseLogCommandError::AlreadyCreated),
            },
        }
    }
}
