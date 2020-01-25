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
    pub fn is_same_bean(&self, brand: &Brand, roast: &Roast) -> bool {
        self.brand == *brand && self.roast == *roast
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
    Purchased { brand: Brand, roast: Roast },
    Decreased { brand: Brand, roast: Roast },
    Removed { brand: Brand, roast: Roast },
}

impl Event<StockAggregate> for StockEvent {
    fn apply_to(self, aggregate: &mut StockAggregate) {
        match self {
            StockEvent::Created => *aggregate = StockAggregate::Created { packs: Vec::new() },
            StockEvent::Purchased { brand, roast } => {
                if let StockAggregate::Created { packs } = aggregate {
                    let same_bean_pack = packs.iter_mut().find(|p| p.is_same_bean(&brand, &roast));
                    match same_bean_pack {
                        None => packs.push(Pack {
                            brand,
                            roast,
                            remaining_amount: RemainingAmount::GteFillingCanister,
                        }),
                        Some(same_bean_pack) => {
                            same_bean_pack.remaining_amount = RemainingAmount::GteFillingCanister
                        }
                    }
                }
            }
            StockEvent::Decreased { brand, roast } => {
                if let StockAggregate::Created { packs } = aggregate {
                    let same_bean_pack = packs.iter_mut().find(|p| p.is_same_bean(&brand, &roast));
                    if let Some(same_bean_pack) = same_bean_pack {
                        same_bean_pack.remaining_amount = RemainingAmount::LtFillingCanister;
                    }
                }
            }
            StockEvent::Removed { brand, roast } => {
                if let StockAggregate::Created { packs } = aggregate {
                    packs.retain(|p| !p.is_same_bean(&brand, &roast))
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum StockCommand {
    Create,
    Purchase {
        brand: Brand,
        roast: Roast,
    },
    Use {
        brand: Brand,
        roast: Roast,
        all: bool,
    },
}

#[derive(Fail, Debug)]
pub enum StockCommandError {
    #[fail(
        display = "brand: {:?}, roast: {:?} not contained in Stock",
        brand, roast
    )]
    NotInStock { brand: Brand, roast: Roast },
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
            StockCommand::Purchase { brand, roast } => match aggregate {
                StockAggregate::Created { .. } => Ok(Some(StockEvent::Purchased { brand, roast })),
                StockAggregate::Uninitialized => Err(StockCommandError::Uninitialized),
            },
            StockCommand::Use { brand, roast, all } => match aggregate {
                StockAggregate::Created { packs } => {
                    if packs.iter().any(|p| p.is_same_bean(&brand, &roast)) {
                        if all {
                            Ok(Some(StockEvent::Removed { brand, roast }))
                        } else {
                            Ok(Some(StockEvent::Decreased { brand, roast }))
                        }
                    } else {
                        Err(StockCommandError::NotInStock { brand, roast })
                    }
                }
                StockAggregate::Uninitialized => Err(StockCommandError::Uninitialized),
            },
        }
    }
}
