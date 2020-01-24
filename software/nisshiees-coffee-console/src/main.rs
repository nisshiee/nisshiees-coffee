extern crate structopt;
extern crate uuid;

use crate::commands::Commands;
use crate::context::Context;
use structopt::StructOpt;

mod commands;
mod context;

#[derive(Debug, StructOpt)]
#[structopt(name = "nisshiees-coffee", about = "nisshiee's coffee運用ツールです")]
struct Opt {
    #[structopt(subcommand)]
    sub: Commands,
}

fn main() {
    let opt = Opt::from_args();
    let mut ctx = Context::new();
    opt.sub.exec(&mut ctx);
}
