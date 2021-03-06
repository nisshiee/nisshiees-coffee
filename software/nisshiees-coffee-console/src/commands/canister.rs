use cqrs_es::store::EventStorage;
use nisshiees_coffee_core::canister_list;
use std::str::FromStr;
use structopt::StructOpt;
use uuid::Uuid;

use crate::context::Context;

#[derive(Debug, StructOpt)]
pub enum CanisterCommands {
    #[structopt(about = "新しいキャニスターを登録します")]
    Add { color: Color, name: Name },
    #[structopt(about = "キャニスターの一覧を表示します")]
    List,
}

#[derive(Debug)]
pub enum Color {
    Blue,
    Green,
    Red,
    Purple,
}

impl FromStr for Color {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blue" => Ok(Color::Blue),
            "green" => Ok(Color::Green),
            "red" => Ok(Color::Red),
            "purple" => Ok(Color::Purple),
            _ => Err("blue, green, red, purpleのいずれかを指定してください"),
        }
    }
}

impl Into<canister_list::Color> for Color {
    fn into(self) -> canister_list::Color {
        match self {
            Color::Blue => canister_list::Color::Blue,
            Color::Green => canister_list::Color::Green,
            Color::Red => canister_list::Color::Red,
            Color::Purple => canister_list::Color::Purple,
        }
    }
}

#[derive(Debug)]
pub enum Name {
    Matsubara,
    Matsumoto,
    Manchose,
    Makabe,
}

impl FromStr for Name {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "matsubara" => Ok(Name::Matsubara),
            "matsumoto" => Ok(Name::Matsumoto),
            "manchose" => Ok(Name::Manchose),
            "makabe" => Ok(Name::Makabe),
            _ => Err("matsubara, matsumoto, manchose, makabeのいずれかを指定してください"),
        }
    }
}

impl Into<canister_list::Name> for Name {
    fn into(self) -> canister_list::Name {
        match self {
            Name::Matsubara => canister_list::Name::Matsubara,
            Name::Matsumoto => canister_list::Name::Matsumoto,
            Name::Manchose => canister_list::Name::Manchose,
            Name::Makabe => canister_list::Name::Makabe,
        }
    }
}

impl CanisterCommands {
    pub fn exec(self, ctx: &mut Context) {
        match self {
            CanisterCommands::Add { color, name } => {
                let cmd =
                    canister_list::CanisterListCommand::AddCanister(canister_list::Canister {
                        id: canister_list::CanisterId(Uuid::new_v4()),
                        color: color.into(),
                        name: name.into(),
                    });
                ctx.canister_list_storage
                    .execute_command(ctx.default_canister_list_id, cmd)
                    .unwrap();
            }
            CanisterCommands::List => {
                let agg = ctx
                    .canister_list_storage
                    .replay_aggregate(ctx.default_canister_list_id)
                    .unwrap();
                let agg = agg.aggregate;
                if let canister_list::CanisterListAggregate::Created { canisters } = agg {
                    canisters.into_iter().for_each(|c| println!("{:?}", c))
                }
            }
        }
    }
}
