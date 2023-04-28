use std::{io::{BufReader, Read}, fs::File, collections::HashMap};
use handlebars::Handlebars;

extern crate pest;
#[macro_use]
extern crate pest_derive;
use pest::Parser;

#[derive(Parser)]
#[grammar = "./settings/tag_seg.pest"]
pub struct TagParser;

const HANDLEBARS_BLANK_ESCAPE_TO: &str = "ß";

fn main() {
    match handle_all("./template.hbs",
        "./temp.txt",
        "./tags.db",
        "b.html") {
        Ok(_) => {},
        Err(e) => { println!("{}", e); }
    }
    
}

fn handle_all(global_template: &str,
        lines: &str,
        db_conn: &str,
        out_file: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
    let mut reg = Handlebars::new();
    reg.register_template_file("page", global_template)?;
    let content = read_content(lines)?;
    
    let mut content = content.split("\r\n\r\n");
    let title = content.next().unwrap_or_default();
    let mut content = content
        .map(|s| format!("<p>{}</p>", s.trim()))
        .collect::<Vec<_>>()
        .join("\n");

    let tags = extract_tags(&content, db_conn);

    for main_tag in tags.iter() {
        let name = main_tag.name.trim_matches('\'');
        let desc_opt = main_tag.desc.clone().unwrap_or_default();
        let desc = desc_opt.trim_matches('\'');
        let re = regex::Regex::new(
            &format!("\\{{\\{{[\\t\\n\\v\\f\\r ]*\"?{}\"?(?s:.)*\\}}\\}}",
                regex::escape(name))
            ).unwrap();
        println!("{}", re);
        let hyperlink = if desc.contains("http")
            { format!("<a href=\"{}\">{}</a>", desc, name) } else { name.into() };
        content = re.replace(&content, &format!("<div id=\"tag\">{}\
        {{{{#each {}}}}}{{{{#with this}}}}\
            {{{{#if desc}}}}
                <div class=\"bubble\"><a href={{{{desc}}}}>{{{{name}}}}</a></div>\
            {{{{else}}}}
                <div class=\"bubble\">{{{{name}}}}</div>\
            {{{{/if}}}}\
        {{{{/with}}}}{{{{/each}}}}</div>",
            hyperlink,
            name.replace(" ", HANDLEBARS_BLANK_ESCAPE_TO)
        )).into();
    }
    println!("{}", content);
    reg.register_template_string("content", content)?;

    let mut env_json = serde_json::json!({
        "date": chrono::Local::now()
            .date_naive()
            .format("%m/%d")
            .to_string(),
        "name": title,
    });
    construct_json_from_database(&mut env_json, tags, db_conn);
    println!("{}", env_json);
    reg.render_to_write("page", &env_json, File::create(out_file)?)?;
    //println!("{}", reg.render("page", &env_json)?);

    Ok(())
}

fn extract_tags(content: &String, db_conn: &str) -> Vec<frictune::Tag> {
    let pairs = TagParser::parse(Rule::final_seg, content)
    .unwrap_or_else(|e| panic!("{}", e));
        
    let tags: Vec<_> = pairs.flat_map(|pair| {
    pair.into_inner()
        .filter(|ip| matches!(ip.as_rule(), Rule::single_tag))
        .map(|pair| {
            let mut main_tag = frictune::Tag { name: "''".into(), desc: None };
            let mut trailers = vec![];
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::leading_word => main_tag.name = format!("'{}'", inner.as_str().trim_matches('\"')),
                    Rule::desc_leading_word => {
                        let desc = inner.as_str().trim_matches('\"');
                        if !desc.is_empty() {
                            main_tag.desc = Some(format!("'{}'", desc));
                        }
                    },
                    Rule::brace => {
                        let mut trailing_tag = frictune::Tag { name: "''".into(), desc: None };
                        let mut num = 1.0;
                        for brace_inner in inner.into_inner() {
                            match brace_inner.as_rule() {
                                Rule::inner_word => trailing_tag.name = format!("'{}'", brace_inner.as_str().trim_matches('\"')),
                                Rule::desc_inner_word => {
                                    let desc = brace_inner.as_str().trim_matches('\"');
                                    if !desc.is_empty() {
                                        trailing_tag.desc = Some(format!("'{}'", desc));
                                    }
                                },
                                Rule::number => num = brace_inner.as_str().parse::<f64>().unwrap(),
                                _ => { frictune::logger::naive::rupt(&format!("brace_inner is {}", brace_inner.as_str())); }
                            }
                        }
                        if !trailing_tag.name.contains("''") {
                            trailers.push((trailing_tag, num as f32));
                        }
                    },
                    _ => { frictune::logger::naive::rupt(&format!("inner is {}", inner.as_str())); }
                }
            }
            update_database(db_conn, &main_tag, &trailers);
            main_tag
        })
        }).collect();
    tags
}

fn construct_json_from_database(json: &mut serde_json::Value, tags: Vec<frictune::Tag>, db_conn: &str) {
    for main_tag in tags.iter() {
        let mut conn = frictune::db::crud::Db::sync_new(db_conn).unwrap();

        json[main_tag.name.trim_matches('\'').replace(" ", HANDLEBARS_BLANK_ESCAPE_TO)] = main_tag.qtr_sync(&mut conn)
            .iter()
            .map(|s| {
                println!("{}", s);
                let desc = frictune::Tag::new(s).qd_sync(&mut conn).unwrap_or("".into());
                serde_json::json!({
                    "name": s,
                    "desc": desc,
                })
            })
            .collect();
    }
}

fn read_content(name: &str) -> Result<String, std::io::Error> {
    let mut reader = BufReader::new(File::open(name)?);

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    Ok(buf)
}

fn update_database(db_uri: &str, leader: &frictune::Tag, trailers: &[(frictune::Tag, f32)]) {
    let mut conn = frictune::db::crud::Db::sync_new(db_uri).unwrap();
    
    for (tag, _) in trailers.iter() {
        tag.add_sync(&mut conn, HashMap::new());
    }

    leader.add_sync(&mut conn, trailers
        .into_iter()
        .map(|(tag, weight)| (tag.name.trim_matches('\'').to_owned(), weight.to_owned()))
        .collect()
    );
}
