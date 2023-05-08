use std::{io::{BufReader, Read}, fs::File, collections::HashMap};
use handlebars::Handlebars;

extern crate pest;
#[macro_use]
extern crate pest_derive;
use pest::Parser;

#[derive(Parser)]
#[grammar = "./settings/tag_seg.pest"]
pub struct TagParser;

/// As something like `{{"Hacker News"}}` will generate a
/// `{{#each Hacker News}}` in the Handlebars file, I replace
/// the space with this symbol
const HANDLEBARS_BLANK_ESCAPE_TO: &str = "ß";

/// The program read something in a `temp.txt` file like
/// ```
/// **any text** {{foo "https://example.com" (bar 0.5)}} **other text**
/// ```
/// and transform it into an .html file.
/// The rules of the format are as follows.
/// 0. the first line will be set as the title.
/// 1. every paragraph separated by two newlines are
///     embraced with `<p></p>`;
/// 2. every paragraph has zero or more `{{TEXT}}` and other plain
///     html things.
/// 3. the TEXT has a leading word, which can be double quoted
///     and have blanks in it, like `"Hacker News"`, or simply
///     a word without blanks.
/// 4. the TEXT has an optional second word, which usually is
///     a `http` website link.
/// 5. the TEXT has zero or more `(BRACED_TEXT)` parts.
/// 6. the BRACED_TEXT has a leading word, an optional second
///     word, and an optional weight. If the weight doesn't exist,
///     it will be set as 1.0 in the database.
/// The leading words are recorded in the database with
/// the braced leading words with the given weight, and then
/// in the final html, the top weighted things will be following
/// the leading words (which is in a `<span id="tag">`)
/// in a `<span class="bubble">` element. If the optional second
/// words are set and are http links, they will be hyperlinked.
/// 
/// The whole stuff will be fit into a template in the file
/// `./template.hbs`.
fn main() {
    match handle_all("./template.hbs",
        "./temp.txt",
        "./tags.db",
        "b.html") {
        Ok(_) => {},
        Err(e) => { println!("{}", e); }
    }
    
}

/// *for later refactor.*
/// This function now uses `pest` to parse the original file
/// and submit the contents to the database, and use `handlebars`
/// to read from the database and replace the tag 
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

    // replace the self-defined tag with a `handlebars` tag
    for main_tag in tags.iter() {
        let name = main_tag.name.trim_matches('\'');
        let desc_opt = main_tag.desc.clone().unwrap_or_default();
        let desc = desc_opt.trim_matches('\'');
        let re = regex::Regex::new(
            &format!("\\{{\\{{[\\t\\n\\v\\f\\r ]*\"?{}\"?(?s:.)*?\\}}\\}}",
                regex::escape(name))
            ).unwrap();
        //println!("{}", re);
        let hyperlink = if desc.contains("http")
            { format!("<a href=\"{}\">{}</a>", desc, name) } else { name.into() };
        content = re.replace(&content, &format!("<span id=\"tag\">{}\
        {{{{#each {}}}}}{{{{#with this}}}}\
            {{{{#if desc}}}}
                <span class=\"bubble\"><a href={{{{desc}}}}>{{{{name}}}}</a></span>\
            {{{{else}}}}
                <span class=\"bubble\">{{{{name}}}}</span>\
            {{{{/if}}}}\
        {{{{/with}}}}{{{{/each}}}}</span>",
            hyperlink,
            name.replace(" ", HANDLEBARS_BLANK_ESCAPE_TO)
        )).into();
    }
    println!("{}", content);
    reg.register_template_string("content", content)?;

    // construct the json from database
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

/// this function has a strong relation with the `pest` parser.
/// If the format is changed, both need to be changed.
fn extract_tags(content: &String, db_conn: &str) -> Vec<frictune::Tag> {
    //TODO: let this init only once
    // let mut conn = match frictune::db::crud::Database::sync_new(db_conn)
    // {
    //     Ok(conn) => std::rc::Rc::new(conn),
    //     Err(e) => frictune::logger::naive::rupt(e.to_string().as_str()),
    // };

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

            main_tag.add_sync(&mut frictune::db::crud::Database::sync_new(db_conn).unwrap(), &trailers
                .into_iter()
                .map(|(tag, weight)|
                    (tag, weight)
                ).collect::<Vec<(_, f32)>>());
            main_tag
        })
    }).collect();
    tags
}

fn construct_json_from_database(json: &mut serde_json::Value, tags: Vec<frictune::Tag>, db_conn: &str) {
    for main_tag in tags.iter() {
        let mut conn = frictune::db::crud::Database::sync_new(db_conn).unwrap();

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
