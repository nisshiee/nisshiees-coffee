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
    type Command = CanisterCommand;
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

pub enum CanisterCommand {
    AddCanister(Canister),
}

impl Command<CanisterListAggregate> for CanisterCommand {
}
