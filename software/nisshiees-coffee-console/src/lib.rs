extern crate chrono;
extern crate uuid;

pub use chrono::NaiveDate;
use uuid::Uuid;

use cqrs_es::*;
use eventstore_onmemory::OnMemoryEventStorage;
use nisshiees_coffee_core::{canister_list, seller};

pub use canister_list::Color::*;
pub use canister_list::Name::*;
use nisshiees_coffee_core::canister_list::Canister;
use nisshiees_coffee_core::seller::purchase_log::{
    PurchaseLogAggregate, PurchaseLogAggregateId, PurchaseLogEvent,
};
pub use nisshiees_coffee_core::{Brand, Gram, Roast};

#[derive(Debug)]
pub struct Context {
    canister_list_storage: OnMemoryEventStorage<canister_list::CanisterListAggregate>,
    default_canister_list_id: canister_list::CanisterListAggregateId,
    seller: SellerContext,
}

#[derive(Debug)]
struct SellerContext {
    purchase_log_storage: OnMemoryEventStorage<seller::purchase_log::PurchaseLogAggregate>,
}

#[derive(Debug)]
struct PurchaseLogProjector {
    sum_quantity: Gram,
}

impl Projector<PurchaseLogAggregate> for PurchaseLogProjector {
    fn project(&mut self, _id: PurchaseLogAggregateId, event: &PurchaseLogEvent) {
        let PurchaseLogEvent::Created { gram: Gram(g), .. } = *event;
        self.sum_quantity = Gram(self.sum_quantity.0 + g);
    }
}

impl Context {
    fn new() -> Context {
        Context {
            canister_list_storage: OnMemoryEventStorage::new(),
            default_canister_list_id: canister_list::CanisterListAggregateId(Uuid::new_v4()),
            seller: SellerContext::new(),
        }
    }
}

impl SellerContext {
    fn new() -> SellerContext {
        let mut purchase_log_storage = OnMemoryEventStorage::new();
        purchase_log_storage.add_projector(PurchaseLogProjector {
            sum_quantity: Gram(0),
        });
        SellerContext {
            purchase_log_storage,
        }
    }
}

pub fn init() -> Context {
    let mut ctx = Context::new();
    let canister_list_id = ctx.default_canister_list_id;
    let canister_list_aggregate = ctx
        .canister_list_storage
        .replay_aggregate(canister_list_id)
        .unwrap();
    let events = canister_list::CanisterListCommand::Create
        .execute_on(&canister_list_aggregate)
        .unwrap();
    events.into_iter().for_each(|e| {
        ctx.canister_list_storage
            .insert(canister_list_id, e)
            .unwrap()
    });
    ctx
}

pub fn show_canister_list(ctx: &Context) {
    let canister_list_id = ctx.default_canister_list_id;
    let canister_list_aggregate = ctx
        .canister_list_storage
        .replay_aggregate(canister_list_id)
        .unwrap();
    println!("{:?}", canister_list_aggregate)
}

pub fn add_canister(ctx: &mut Context, color: canister_list::Color, name: canister_list::Name) {
    let canister_list_id = ctx.default_canister_list_id;
    let canister_list_aggregate = ctx
        .canister_list_storage
        .replay_aggregate(canister_list_id)
        .unwrap();
    let command = canister_list::CanisterListCommand::AddCanister(Canister {
        id: canister_list::CanisterId(Uuid::new_v4()),
        color,
        name,
    });
    let events = command.execute_on(&canister_list_aggregate).unwrap();
    // TODO: ↑command.execute_onのErrはハンドリングしたい
    events.into_iter().for_each(|e| {
        ctx.canister_list_storage
            .insert(canister_list_id, e)
            .unwrap()
    });
    show_canister_list(ctx);
}

pub fn add_purchase_log(
    ctx: &mut Context,
    brand: Brand,
    roast: Roast,
    gram: Gram,
    date: NaiveDate,
) {
    let id = seller::purchase_log::PurchaseLogAggregateId(Uuid::new_v4());
    let mut aggregate = ctx
        .seller
        .purchase_log_storage
        .replay_aggregate(id)
        .unwrap();
    let command = seller::purchase_log::PurchaseLogCommand::Create {
        brand,
        roast,
        gram,
        date,
    };
    let events = command.execute_on(&mut aggregate).unwrap();
    events
        .into_iter()
        .for_each(|e| ctx.seller.purchase_log_storage.insert(id, e).unwrap());
}
