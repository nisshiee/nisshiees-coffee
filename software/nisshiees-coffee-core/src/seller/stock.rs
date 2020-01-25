use crate::{Brand, Roast};
use cqrs_es::{Aggregate, AggregateId, Command, CommandError, Event};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum StockAggregate {
    Uninitialized,
    Created { packs: Vec<Pack> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub brand: Brand,
    pub roast: Roast,
    pub remaining_amount: RemainingAmount,
}

impl Pack {
    pub fn is_same_bean(&self, other: &Pack) -> bool {
        self.brand == other.brand && self.roast == other.roast
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RemainingAmount {
    GteFillingCanister,
    LtFillingCanister,
}

impl Default for StockAggregate {
    fn default() -> Self {
        StockAggregate::Uninitialized
    }
}

impl Aggregate for StockAggregate {
    type Id = StockAggregateId;
    type Event = StockEvent;
    type Command = StockCommand;

    fn type_name() -> &'static str {
        "seller/stock"
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StockAggregateId(pub Uuid);

impl ToString for StockAggregateId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl AggregateId<StockAggregate> for StockAggregateId {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StockEvent {
    Created,
    Purchased(Pack),
}

impl Event<StockAggregate> for StockEvent {
    fn apply_to(self, aggregate: &mut StockAggregate) {
        match self {
            StockEvent::Created => *aggregate = StockAggregate::Created { packs: Vec::new() },
            StockEvent::Purchased(new_pack) => {
                if let StockAggregate::Created { packs } = aggregate {
                    let same_bean_pack = packs.iter_mut().find(|p| p.is_same_bean(&new_pack));
                    match same_bean_pack {
                        None => packs.push(new_pack),
                        Some(same_bean_pack) => {
                            same_bean_pack.remaining_amount = RemainingAmount::GteFillingCanister
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum StockCommand {
    Create,
    Purchase(Pack),
}

#[derive(Fail, Debug)]
pub enum StockCommandError {
    #[fail(display = "Stock already created")]
    AlreadyCreated,
    #[fail(display = "Stock uninitialized")]
    Uninitialized,
}

impl CommandError for StockCommandError {}

impl Command<StockAggregate> for StockCommand {
    type Events = Option<StockEvent>;
    type Error = StockCommandError;

    fn execute_on(self, aggregate: &StockAggregate) -> Result<Self::Events, Self::Error> {
        match self {
            StockCommand::Create => match aggregate {
                StockAggregate::Uninitialized => Ok(Some(StockEvent::Created)),
                _ => Err(StockCommandError::AlreadyCreated),
            },
            StockCommand::Purchase(new_pack) => match aggregate {
                StockAggregate::Created { .. } => Ok(Some(StockEvent::Purchased(new_pack))),
                StockAggregate::Uninitialized => Err(StockCommandError::Uninitialized),
            },
        }
    }
}
