use crate::commands::seller::purchase_log::PurchaseLogCommands;
use crate::context::Context;
use structopt::StructOpt;

mod purchase_log;

#[derive(Debug, StructOpt)]
pub enum SellerCommands {
    #[structopt(about = "仕入ログに関する操作を実行します")]
    PurchaseLog(PurchaseLogCommands),
}

impl SellerCommands {
    pub fn exec(self, ctx: &mut Context) {
        match self {
            SellerCommands::PurchaseLog(c) => c.exec(ctx),
        }
    }
}
