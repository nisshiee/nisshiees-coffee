use uuid::Uuid;
use cqrs_es::*;
use nisshiees_coffee_core::canister_list::*;

let mut canister_list = CanisterListAggregate::Uninitialized;
CanisterListEvent::Created.apply_to(&mut canister_list);

let command = CanisterListCommand::AddCanister(Canister { id: CanisterId(Uuid::new_v4()), color: Color::Blue, name: Name::Matsubara });
