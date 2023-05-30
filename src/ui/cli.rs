use frictune::Tag;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// add a new tag and
    /// optionally link to existent tags with weights
    Add {
        name: String,
        #[arg(long, short)]
        tags: Vec<String>,
        #[arg(long, short)]
        weights: Vec<f32>,
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
    Mod {
        name: String,
        desc: String,
    },
    Repl,
}

pub fn parse_args(db_conn: &mut frictune::db::crud::Database) {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Add { name, tags, weights }) => {
            if tags.len() == weights.len() {
                Tag::new(name).add_sync(db_conn,
                    &tags.iter().zip(weights)
                        .map(|(tag, weight)| (tag.to_owned(), weight.to_owned()))
                        .collect::<Vec<(String, f32)>>()
                );
            }

            else {
                frictune::logger::warn("links should be <name, weight> pairs.".to_owned());
                Tag::new(name).add_sync::<String>(db_conn, &[]);
            }
            
        },
        Some(Commands::Del { name }) => {
            Tag::new(name).rem_sync(db_conn);
        },
        Some(Commands::Eval { src, tgt }) => {
            match Tag::query_sync(db_conn, src, tgt) {
                Some(weight) => { frictune::logger::print(
                    &format!("The weight between {} and {} is {}",
                        src, tgt, weight
                    )
                )},
                None => {
                    frictune::logger::print("No such link")
                }
            };
            let desc = Tag::new(src).qd_sync(db_conn).unwrap_or_default();
            frictune::logger::print(
                &format!("The tag {src} is linked with description {desc}.")
            )
        },
        Some(Commands::Link { src, tgt, weight }) => {
            Tag::new(src).link_sync(db_conn, tgt, *weight);
        },
        Some(Commands::Mod { name, desc }) => {
            let concerned = Tag::new(name);
            let old_desc = concerned.qd_sync(db_conn).unwrap_or_default();
            Tag::new(name).mod_sync(db_conn, desc);
            let new_desc = concerned.qd_sync(db_conn).unwrap_or_default();
            frictune::logger::print(
                &format!("Tag {name} is updated with description {new_desc} from {old_desc}.")
            );
        },
        Some(Commands::Repl) => { frictune::logger::rupt("not implemented"); },
        None => { frictune::logger::rupt("not implemented"); },
    }
}