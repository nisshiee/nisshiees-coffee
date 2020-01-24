use crate::commands::canister::CanisterCommands;
use crate::commands::seller::SellerCommands;
use crate::Context;
use cqrs_es::EventStorage;
use nisshiees_coffee_core::canister_list;
use structopt::StructOpt;

mod canister;
mod seller;

#[derive(Debug, StructOpt)]
pub enum Commands {
    #[structopt(about = "初期化処理を実行します")]
    Init,
    #[structopt(about = "キャニスターに関する操作を実行します")]
    Canister(CanisterCommands),
    #[structopt(about = "運用者に関する操作を実行します")]
    Seller(SellerCommands),
}

impl Commands {
    pub fn exec(self, ctx: &mut Context) {
        match self {
            Commands::Init => {
                let cmd = canister_list::CanisterListCommand::Create;
                ctx.canister_list_storage
                    .execute_command(ctx.default_canister_list_id, cmd)
                    .unwrap();
            }
            Commands::Canister(c) => c.exec(ctx),
            Commands::Seller(c) => c.exec(ctx),
        }
    }
}
