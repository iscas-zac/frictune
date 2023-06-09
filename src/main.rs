#![allow(non_snake_case)]
pub mod ui;
#[cfg(not(target_arch = "wasm32"))]
pub mod conf;

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            let settings = match crate::conf::read_config() {
                Ok(config) => config,
                Err(e) => frictune::logger::rupt(e.to_string().as_str()),
            };
            let mut conn = match frictune::db::crud::Database::sync_new(&settings.db_uri)
            {
                Ok(conn) => conn,
                Err(e) => frictune::logger::rupt(e.to_string().as_str()),
            };
            ui::cli::parse_args(&mut conn);
        }
        else {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("logger library is not enabled");
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
            let response = gloo::net::http::Request::get("/tags.gluesql1").send().await
                .unwrap();
            response.binary().await.unwrap_or_default()
        });
    match temp.value() {
        Some(response) => {
            // frictune::logger::print(&format!("{:?}", String::from_utf8(response.clone())));
            // frictune::logger::print(&format!("{:?}", response));
            // frictune::logger::print(&format!("{:?}", 
            //     bincode::serialize(
            //         &gluesql::memory_storage::MemoryStorage::default()).unwrap()));

            let mut glue = Database::deser_new(response).unwrap();
            use_shared_state_provider(cx, || glue);

            let tag1_name = use_state(cx, || "abc".to_string());
            let tag1_desc = use_state(cx, || "".to_string());
            let tag2_name = use_state(cx, || "".to_string());
            let tag2_desc = use_state(cx, || "".to_string());
            let link = use_state(cx, || 0.0);
            let words = use_state(cx, || "".to_string());

            let glue = use_shared_state::<Database>(cx).unwrap();
            // ui::graph::d3play(&mut glue.write_silent());
            let nodes = use_state(cx, || ui::graph::export_nodes_json(&mut glue.write_silent()));
            let links = use_state(cx, || ui::graph::export_links_json(&mut glue.write_silent()));
            cx.render(rsx! {
                input { value: "{tag1_name}", oninput: |e| tag1_name.set(e.value.clone()) },
                input { value: "{tag1_desc}", oninput: |e| tag1_desc.set(e.value.clone()) },
                input { value: "{tag2_name}", oninput: |e| tag2_name.set(e.value.clone()) },
                input { value: "{tag2_desc}", oninput: |e| tag2_desc.set(e.value.clone()) },
                input { value: "{link}", oninput: |e| link.set(e.value.clone().parse::<f32>().unwrap_or_default()) },
                div { "{words}" }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        frictune::Tag::new_with_desc(tag1_name.get(), Some(tag1_desc.get().into()))
                            .add_sync::<String>(&mut glue.write_silent(), &[]);
                    },
                    "ADD"
                }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        let mut glue = glue.write_silent();
                        let tags = frictune::Tag::new_with_desc(tag1_name.get(), Some(tag1_desc.get().into()))
                            .qtrd(&mut glue);
                        words.set(
                            tag1_name.get().to_string() + " " +
                            &tags.into_iter().map(|(tag, desc, weight)|
                                    format!("{} | {} | {}", tag, desc.unwrap_or_default(), weight.unwrap_or_default())
                                ).collect::<Vec<_>>()
                                .join("\n")
                        );
                    },
                    "QTR"
                }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        let mut glue = glue.write_silent();
                        frictune::Tag::new_with_desc(tag1_name.get(), Some(tag1_desc.get().into()))
                            .link_sync(&mut glue,
                                &frictune::Tag::new_with_desc(tag2_name.get(), Some(tag2_desc.get().into()))
                                , *link.get());
                        frictune::logger::warn(link.get().to_string());
                    },
                    "LNK"
                }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        let mut glue = glue.write_silent();
                        frictune::Tag::new_with_desc(tag1_name.get(), Some(tag1_desc.get().into()))
                            .rem_sync(&mut glue);
                    },
                    "REM"
                }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        let mut glue = glue.write_silent();
                        frictune::Tag::new_with_desc(tag1_name.get(), Some(tag1_desc.get().into()))
                            .mod_sync(&mut glue, tag1_desc.get());
                    },
                    "MOD"
                }
                button {
                    onclick: move |_| {
                        let glue = use_shared_state::<Database>(cx).unwrap();
                        let mut glue = glue.write_silent();
                        let tag_name = tag1_name.get();
                        let (new_nodes, new_links) = ui::graph::export_succ_json(tag_name, &mut glue);
                        nodes.set(new_nodes);
                        links.set(new_links);
                    },
                    "REDRAW"
                }
                button {
                    id: "redraw",
                    "AAAAAAAAAAAAAAAAAAAAAAAAAAA!"
                }
                ui::graph::inspect_graph {
                    nodes: nodes.get(),
                    links: links.get(),
                }
            })
        },
        None => cx.render(rsx! {
            p { "111" }
        })
    }

}