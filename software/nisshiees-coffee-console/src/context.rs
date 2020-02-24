use cqrs_es::Id;
use eventstorage_file::FileEventStorage;
use nisshiees_coffee_core::canister_list::{CanisterListAggregate, CanisterListEvent};
use nisshiees_coffee_core::seller::stock::{StockAggregate, StockEvent};
use uuid::Uuid;

pub struct Context {
    pub canister_list_storage: FileEventStorage<CanisterListAggregate, CanisterListEvent>,
    pub default_canister_list_id: Id<CanisterListAggregate>,
    pub seller_stock_storage: FileEventStorage<StockAggregate, StockEvent>,
    pub default_seller_stock_id: Id<StockAggregate>,
}

const EVENT_STORAGE_ROOT_PATH: &str = "target/storage/events";

impl Context {
    pub fn new() -> Context {
        let canister_list_storage = FileEventStorage::new(EVENT_STORAGE_ROOT_PATH).unwrap();
        let seller_stock_storage = FileEventStorage::new(EVENT_STORAGE_ROOT_PATH).unwrap();
        let default_canister_list_id =
            Uuid::parse_str("008044ba-7674-4ff3-a0ae-ef724ddd66a6").unwrap();
        let default_canister_list_id = From::from(default_canister_list_id);
        let default_seller_stock_id =
            Uuid::parse_str("7b068432-c5a8-4e7e-ba79-758b902a07ba").unwrap();
        let default_seller_stock_id = From::from(default_seller_stock_id);
        Context {
            canister_list_storage,
            default_canister_list_id,
            seller_stock_storage,
            default_seller_stock_id,
        }
    }
}
