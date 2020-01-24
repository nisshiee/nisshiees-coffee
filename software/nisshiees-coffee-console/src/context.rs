use eventstorage_file::FileEventStorage;
use nisshiees_coffee_core::canister_list;
use nisshiees_coffee_core::seller;
use uuid::Uuid;

pub struct Context<'a> {
    pub canister_list_storage: FileEventStorage<'a, canister_list::CanisterListAggregate>,
    pub purchase_log_storage: FileEventStorage<'a, seller::purchase_log::PurchaseLogAggregate>,
    pub default_canister_list_id: canister_list::CanisterListAggregateId,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        let canister_list_storage = FileEventStorage::new("storage/events").unwrap();
        let purchase_log_storage = FileEventStorage::new("storage/events").unwrap();
        let default_canister_list_id =
            Uuid::parse_str("008044ba-7674-4ff3-a0ae-ef724ddd66a6").unwrap();
        let default_canister_list_id =
            canister_list::CanisterListAggregateId(default_canister_list_id);
        Context {
            canister_list_storage,
            purchase_log_storage,
            default_canister_list_id,
        }
    }
}
