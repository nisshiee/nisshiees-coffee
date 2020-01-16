extern crate uuid;

use uuid::Uuid;

use nisshiees_coffee_core::canister_list;
use cqrs_es::*;
use eventstore_onmemory::OnMemoryEventStorage;

pub struct Context {
    canister_list_storage: OnMemoryEventStorage<canister_list::CanisterListAggregate>,
    default_canister_list_id: canister_list::CanisterListAggregateId,
}

pub use canister_list::Color::*;
pub use canister_list::Name::*;
use nisshiees_coffee_core::canister_list::Canister;

impl Context {
    fn new() -> Context {
        Context {
            canister_list_storage: OnMemoryEventStorage::new(),
            default_canister_list_id: canister_list::CanisterListAggregateId(Uuid::new_v4()),
        }
    }
}

pub fn init() -> Context {
    let mut ctx = Context::new();
    let canister_list_id = ctx.default_canister_list_id;
    let canister_list_aggregate = ctx.canister_list_storage.replay_aggregate(canister_list_id).unwrap();
    let events = canister_list::CanisterListCommand::Create.execute_on(&canister_list_aggregate).unwrap();
    events.into_iter().for_each(|e| ctx.canister_list_storage.insert(canister_list_id, e).unwrap());
    ctx
}

pub fn show(ctx: &Context) {
    let canister_list_id = ctx.default_canister_list_id;
    let canister_list_aggregate = ctx.canister_list_storage.replay_aggregate(canister_list_id).unwrap();
    println!("{:?}", canister_list_aggregate)
}

pub fn add(ctx: &mut Context, color: canister_list::Color, name: canister_list::Name) {
    let canister_list_id = ctx.default_canister_list_id;
    let canister_list_aggregate = ctx.canister_list_storage.replay_aggregate(canister_list_id).unwrap();
    let command = canister_list::CanisterListCommand::AddCanister(Canister {
        id: canister_list::CanisterId(Uuid::new_v4()),
        color,
        name
    });
    let events = command.execute_on(&canister_list_aggregate).unwrap();
    // TODO: ↑command.execute_onのErrはハンドリングしたい
    events.into_iter().for_each(|e| ctx.canister_list_storage.insert(canister_list_id, e).unwrap());
    show(ctx);
}
