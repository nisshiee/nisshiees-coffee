use cqrs_es::{Aggregate, Event, AggregateId, Command};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum CanisterListAggregate {
    Uninitialized,
    Created {
        canisters: Vec<Canister>,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CanisterListAggregateId(Uuid);

impl AggregateId<CanisterListAggregate> for CanisterListAggregateId {}

#[derive(Debug, Clone)]
pub struct Canister {
    id: CanisterId,
    color: Color,
    name: Name,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CanisterId(Uuid);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    Blue,
    Green,
    Red,
    Purple,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Name {
    Matsubara,
    Matsumoto,
    Manchose,
    Makabe,
}

impl Aggregate for CanisterListAggregate {
    type Id = CanisterListAggregateId;
    type Event = CanisterListEvent;
    type Command = CanisterListCommand;
}

impl Default for CanisterListAggregate {
    fn default() -> Self {
        CanisterListAggregate::Uninitialized
    }
}

pub enum CanisterListEvent {
    Created,
    CanisterAdded(Canister),
}

impl Event<CanisterListAggregate> for CanisterListEvent {
    fn apply_to(self, aggregate: &mut CanisterListAggregate) {
        match self {
            CanisterListEvent::Created => {
                if let CanisterListAggregate::Uninitialized = *aggregate {
                    *aggregate = CanisterListAggregate::Created { canisters: Vec::new() }
                }
            }
            CanisterListEvent::CanisterAdded(canister) => {
                if let CanisterListAggregate::Created { canisters } = aggregate {
                    canisters.push(canister)
                }
            }
        }
    }
}

pub enum CanisterListCommand {
    Create,
    AddCanister(Canister),
}

#[derive(Fail, Debug)]
pub enum CanisterListCommandError {
    #[fail(display = "ID duplicated")]
    IdDuplicated,
    #[fail(display = "Name duplicated")]
    NameDuplicated,
    #[fail(display = "Color duplicated")]
    ColorDuplicated,
    #[fail(display = "CanisterList already created")]
    AlreadyCreated,
    #[fail(display = "CanisterList uninitialized")]
    Uninitialized,
}

impl Command<CanisterListAggregate> for CanisterListCommand {
    type Events = Option<CanisterListEvent>;
    type Error = CanisterListCommandError;

    fn execute_on(self, aggregate: &CanisterListAggregate) -> Result<Self::Events, Self::Error> {
        match self {
            CanisterListCommand::Create => {
                match aggregate {
                    CanisterListAggregate::Uninitialized => Ok(Some(CanisterListEvent::Created)),
                    _ => Err(CanisterListCommandError::AlreadyCreated)
                }
            }

            CanisterListCommand::AddCanister(adding) => {
                match aggregate {
                    CanisterListAggregate::Created { canisters } => {
                        if canisters.into_iter().any(|c| c.id == adding.id) {
                            return Err(CanisterListCommandError::IdDuplicated)
                        }
                        if canisters.into_iter().any(|c| c.color == adding.color) {
                            return Err(CanisterListCommandError::ColorDuplicated)
                        }
                        if canisters.into_iter().any(|c| c.name == adding.name) {
                            return Err(CanisterListCommandError::NameDuplicated)
                        }
                        Ok(Some(CanisterListEvent::CanisterAdded(adding)))
                    },
                    _ => Err(CanisterListCommandError::Uninitialized),
                }
            }
        }
    }
}
