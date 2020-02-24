use cqrs_es::store::EventStorage;
use nisshiees_coffee_core::seller::stock;
use nisshiees_coffee_core::{Brand, Roast};
use structopt::StructOpt;

use crate::context::Context;

#[derive(Debug, StructOpt)]
pub enum StockCommands {
    #[structopt(about = "新しい豆の購入を登録します")]
    PurchasePack {
        #[structopt(long = "--brand")]
        brand: String,
        #[structopt(long = "--roast")]
        roast: u8,
    },
    #[structopt(about = "在庫から豆をキャニスターに移します")]
    Use {
        #[structopt(long = "--brand")]
        brand: String,
        #[structopt(long = "--roast")]
        roast: u8,
        #[structopt(long = "--all")]
        all: bool,
    },
    #[structopt(about = "在庫状況を表示します")]
    Show,
}

impl StockCommands {
    pub fn exec(self, ctx: &mut Context) {
        match self {
            StockCommands::PurchasePack { brand, roast } => {
                let cmd = stock::StockCommand::Purchase {
                    brand: Brand(brand),
                    roast: Roast(roast),
                };
                ctx.seller_stock_storage
                    .execute_command(ctx.default_seller_stock_id, cmd)
                    .unwrap();
            }
            StockCommands::Use { brand, roast, all } => {
                let cmd = stock::StockCommand::Use {
                    brand: Brand(brand),
                    roast: Roast(roast),
                    all,
                };
                ctx.seller_stock_storage
                    .execute_command(ctx.default_seller_stock_id, cmd)
                    .unwrap();
            }
            StockCommands::Show => {
                let agg = ctx
                    .seller_stock_storage
                    .replay_aggregate(ctx.default_seller_stock_id)
                    .unwrap();
                let agg = agg.aggregate;
                if let stock::StockAggregate::Created { packs } = agg {
                    packs.iter().for_each(|p| println!("{:?}", p));
                }
            }
        }
    }
}
