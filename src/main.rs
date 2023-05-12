#![allow(non_snake_case)]
#[cfg(not(target_arch = "wasm32"))]
pub mod ui;
#[cfg(not(target_arch = "wasm32"))]
pub mod conf;

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
    use frictune::db::crud::Database;

    let temp =
        use_future(&cx, (), |_| async move {
            let response = gloo::net::http::Request::get("/gluesql").send().await
                .unwrap();
            response.binary().await.unwrap_or_default()
        });
    match temp.value() {
        Some(response) => {
            let glue = Database::deser_new(response).unwrap();
            use_shared_state_provider(cx, || glue);

            let tag1_name = use_state(cx, || "abc".to_string());
            let tag1_desc = use_state(cx, || "".to_string());
            let tag2_name = use_state(cx, || "".to_string());
            let tag2_desc = use_state(cx, || "".to_string());
            let link = use_state(cx, || 0.0);

            cx.render(rsx! {
                
                input { value: "{tag1_name}", },
                input { value: "{tag1_desc}", },
                input { value: "{tag2_name}", },
                input { value: "{tag2_desc}", },
                input { value: "{link}", },
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        frictune::Tag::new_with_desc(tag1_name.get(), Some(tag1_desc.get().into()))
                            .add_sync::<String>(&mut glue.write_silent(), &[]);
                        web_sys::console::log_1(&"111".into());
                    },
                    "C"
                }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx);
                    },
                    "R"
                }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        let mut glue = glue.write_silent();
                        frictune::Tag::new_with_desc(tag1_name.get(), Some(tag1_desc.get().into()))
                            .link_sync(&mut glue,
                                &frictune::Tag::new_with_desc(tag2_name.get(), Some(tag2_desc.get().into()))
                                , *link.get())
                    },
                    "U"
                }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        let mut glue = glue.write_silent();
                        frictune::Tag::new_with_desc(tag1_name.get(), Some(tag1_desc.get().into()))
                            .rem_sync(&mut glue);
                    },
                    "D"
                }
            })
        },
        None => cx.render(rsx! {
            p { "111" }
        })
    }

}