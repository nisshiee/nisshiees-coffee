extern crate structopt;
extern crate uuid;

use cqrs_es::EventStorage;
use eventstorage_file::FileEventStorage;
use nisshiees_coffee_core::canister_list;
use nisshiees_coffee_core::canister_list::Canister;
use std::str::FromStr;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
#[structopt(name = "nisshiees-coffee", about = "nisshiee's coffee運用ツールです")]
struct Opt {
    #[structopt(subcommand)]
    sub: Subcommands,
}

#[derive(Debug, StructOpt)]
enum Subcommands {
    #[structopt(about = "初期化処理を実行します")]
    Init,
    #[structopt(about = "キャニスターに関する操作を実行します")]
    Canister(CanisterCommands),
}

#[derive(Debug, StructOpt)]
enum CanisterCommands {
    #[structopt(about = "新しいキャニスターを登録します")]
    Add { color: Color, name: Name },
    #[structopt(about = "キャニスターの一覧を表示します")]
    List,
}

#[derive(Debug)]
enum Color {
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
enum Name {
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

struct Context<'a> {
    canister_list_storage: FileEventStorage<'a, canister_list::CanisterListAggregate>,
    default_canister_list_id: canister_list::CanisterListAggregateId,
}

impl<'a> Context<'a> {
    fn new() -> Context<'a> {
        let canister_list_storage = FileEventStorage::new("storage/events").unwrap();
        let default_canister_list_id =
            Uuid::parse_str("008044ba-7674-4ff3-a0ae-ef724ddd66a6").unwrap();
        let default_canister_list_id =
            canister_list::CanisterListAggregateId(default_canister_list_id);
        Context {
            canister_list_storage,
            default_canister_list_id,
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    let mut ctx = Context::new();

    match opt.sub {
        Subcommands::Init => {
            let cmd = canister_list::CanisterListCommand::Create;
            ctx.canister_list_storage
                .execute_command(ctx.default_canister_list_id, cmd)
                .unwrap();
        }
        Subcommands::Canister(CanisterCommands::Add { color, name }) => {
            let cmd = canister_list::CanisterListCommand::AddCanister(Canister {
                id: canister_list::CanisterId(Uuid::new_v4()),
                color: color.into(),
                name: name.into(),
            });
            ctx.canister_list_storage
                .execute_command(ctx.default_canister_list_id, cmd)
                .unwrap();
        }
        Subcommands::Canister(CanisterCommands::List) => {
            let agg = ctx
                .canister_list_storage
                .replay_aggregate(ctx.default_canister_list_id)
                .unwrap();
            if let canister_list::CanisterListAggregate::Created { canisters } = agg {
                canisters.into_iter().for_each(|c| println!("{:?}", c))
            }
        }
    }
}
