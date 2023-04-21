pub mod ui;
pub mod conf;

use frictune::db;

fn main() {
    let settings = match crate::conf::read_config() {
        Ok(config) => config,
        Err(e) => frictune::logger::naive::rupt(e.to_string().as_str()),
    };
    let mut conn = match db::crud::Db::sync_new(&settings.db_uri)
    {
        Ok(conn) => conn,
        Err(e) => frictune::logger::naive::rupt(e.to_string().as_str()),
    };
    ui::cli::parse_args(&mut conn);

}
