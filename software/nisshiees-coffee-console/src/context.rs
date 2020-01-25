use eventstorage_file::FileEventStorage;
use nisshiees_coffee_core::canister_list;
use nisshiees_coffee_core::seller;
use uuid::Uuid;

pub struct Context<'a> {
    pub canister_list_storage: FileEventStorage<'a, canister_list::CanisterListAggregate>,
    pub default_canister_list_id: canister_list::CanisterListAggregateId,
    pub seller_stock_storage: FileEventStorage<'a, seller::stock::StockAggregate>,
    pub default_seller_stock_id: seller::stock::StockAggregateId,
}

const EVENT_STORAGE_ROOT_PATH: &str = "target/storage/events";

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        let canister_list_storage = FileEventStorage::new(EVENT_STORAGE_ROOT_PATH).unwrap();
        let seller_stock_storage = FileEventStorage::new(EVENT_STORAGE_ROOT_PATH).unwrap();
        let default_canister_list_id =
            Uuid::parse_str("008044ba-7674-4ff3-a0ae-ef724ddd66a6").unwrap();
        let default_canister_list_id =
            canister_list::CanisterListAggregateId(default_canister_list_id);
        let default_seller_stock_id =
            Uuid::parse_str("7b068432-c5a8-4e7e-ba79-758b902a07ba").unwrap();
        let default_seller_stock_id = seller::stock::StockAggregateId(default_seller_stock_id);
        Context {
            canister_list_storage,
            default_canister_list_id,
            seller_stock_storage,
            default_seller_stock_id,
        }
    }
}
