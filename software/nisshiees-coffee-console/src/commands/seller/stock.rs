use crate::context::Context;
use cqrs_es::EventStorage;
use nisshiees_coffee_core::seller::stock;
use nisshiees_coffee_core::{Brand, Roast};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum StockCommands {
    #[structopt(about = "新しい豆の購入を登録します")]
    PurchasePack {
        #[structopt(long = "--brand")]
        brand: String,
        #[structopt(long = "--roast")]
        roast: u8,
    },
    #[structopt(about = "在庫状況を表示します")]
    Show,
}

impl StockCommands {
    pub fn exec(self, ctx: &mut Context) {
        match self {
            StockCommands::PurchasePack { brand, roast } => {
                let cmd = stock::StockCommand::Purchase(stock::Pack {
                    brand: Brand(brand),
                    roast: Roast(roast),
                    remaining_amount: stock::RemainingAmount::GteFillingCanister,
                });
                ctx.seller_stock_storage
                    .execute_command(ctx.default_seller_stock_id, cmd)
                    .unwrap();
            }
            StockCommands::Show => {
                let agg = ctx
                    .seller_stock_storage
                    .replay_aggregate(ctx.default_seller_stock_id)
                    .unwrap();
                if let stock::StockAggregate::Created { packs } = agg {
                    packs.iter().for_each(|p| println!("{:?}", p));
                }
            }
        }
    }
}
