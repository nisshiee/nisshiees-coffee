use crate::context::Context;
use chrono::NaiveDate;
use cqrs_es::EventStorage;
use nisshiees_coffee_core::seller::purchase_log;
use nisshiees_coffee_core::{Brand, Gram, Roast};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
pub enum PurchaseLogCommands {
    #[structopt(about = "新しい仕入ログを登録します")]
    Create {
        #[structopt(long = "--brand")]
        brand: String,
        #[structopt(long = "--roast")]
        roast: u8,
        #[structopt(long = "--gram")]
        gram: u32,
        #[structopt(long = "--date")]
        date: Option<NaiveDate>,
    },
}

impl PurchaseLogCommands {
    pub fn exec(self, ctx: &mut Context) {
        match self {
            PurchaseLogCommands::Create {
                brand,
                roast,
                gram,
                date,
            } => {
                let cmd = purchase_log::PurchaseLogCommand::Create {
                    brand: Brand(brand),
                    roast: Roast(roast),
                    gram: Gram(gram),
                    date: date.unwrap_or(chrono::Local::today().naive_local()),
                };
                ctx.purchase_log_storage
                    .execute_command(purchase_log::PurchaseLogAggregateId(Uuid::new_v4()), cmd)
                    .unwrap()
            }
        }
    }
}
