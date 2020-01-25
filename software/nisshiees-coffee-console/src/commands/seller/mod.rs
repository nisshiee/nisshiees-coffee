use crate::commands::seller::stock::StockCommands;
use crate::context::Context;
use structopt::StructOpt;

mod stock;

#[derive(Debug, StructOpt)]
pub enum SellerCommands {
    #[structopt(about = "在庫に関する操作を実行します")]
    Stock(StockCommands),
}

impl SellerCommands {
    pub fn exec(self, ctx: &mut Context) {
        match self {
            SellerCommands::Stock(c) => c.exec(ctx),
        }
    }
}
