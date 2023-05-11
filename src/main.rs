#![allow(non_snake_case)]
#[cfg(not(target_arch = "wasm32"))]
pub mod ui;
#[cfg(not(target_arch = "wasm32"))]
pub mod conf;
#[cfg(not(target_arch = "wasm32"))]
use frictune::db;

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            let settings = match crate::conf::read_config() {
                Ok(config) => config,
                Err(e) => frictune::logger::rupt(e.to_string().as_str()),
            };
            let mut conn = match db::crud::Database::sync_new(&settings.db_uri)
            {
                Ok(conn) => conn,
                Err(e) => frictune::logger::rupt(e.to_string().as_str()),
            };
            ui::cli::parse_args(&mut conn);
        }
        else {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("无法初始化日志库");

            dioxus_web::launch(App);
        }
    }
}

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        a {
            href: "https://www.dioxus.cn/",
            "Dioxus 中文网"
        }
        div {
            button {}
        }
        div {
            button {}
        }
    })
}