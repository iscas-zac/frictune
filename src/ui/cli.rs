use std::collections::HashMap;

use frictune::Tag;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// 
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// add a new tag and
    /// optionally link to existent tags with weights
    Add {
        name: String,
        vararg: Option<String>,
    },
    Del {
        name: String,
    },
    Link {
        src: String,
        tgt: String,
        weight: f32,
    },
    Eval {
        src: String,
        tgt: String,
    },
    Repl,
}

pub fn parse_args(db_conn: &mut frictune::db::crud::Db) {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Add { name, vararg }) => {
            Tag::new(name).add_sync(db_conn, HashMap::new());
        },
        Some(Commands::Del { name }) => {
            Tag::new(name).rem_sync(db_conn);
        },
        Some(Commands::Eval { src, tgt }) => {
            match Tag::query_sync(db_conn, src, tgt) {
                Some(weight) => { frictune::logger::naive::print(
                    &format!("The weight between {} and {} is {}",
                        src, tgt, weight
                    )
                )},
                None => {
                    frictune::logger::naive::print("No such link")
                }
            }
        },
        Some(Commands::Link { src, tgt, weight }) => {
            Tag::new(src).link_sync(db_conn, tgt, *weight);
        },
        Some(Commands::Repl) => { frictune::logger::naive::rupt("not implemented"); },
        None => { frictune::logger::naive::rupt("not implemented"); },
    }
}