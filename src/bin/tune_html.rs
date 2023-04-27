use std::{io::{BufReader, Read}, fs::File, collections::HashMap};
use handlebars::{Handlebars, Path, handlebars_helper};


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
    reg.register_partial("content", &content)?;
    
    let tags: Vec<_> = reg.get_template("content").unwrap()
        .elements
        .iter()
        .filter_map(|e| {
            match e {
                handlebars::template::TemplateElement::Expression(ht) => {
                    let main_tag = expand_expr_parse(db_conn, *ht.to_owned());
                    Some(main_tag)
                },
                _ => None,
            }
        }).collect();
        
    for main_tag in tags.iter() {
        let name = main_tag.name.trim_matches('\'');
        let desc_opt = main_tag.desc.clone().unwrap_or_default();
        let desc = desc_opt.trim_matches('\'');
        let re = regex::Regex::new(
            &format!("\\{{\\{{[\\t\\n\\v\\f\\r ]*{}.*\\}}\\}}",
                regex::escape(name))
            ).unwrap();
        let hyperlink = if desc.contains("https")
            { format!("<a href=\"{}\">{}</a>", desc, name) } else { name.into() };
        content = re.replace(&content, &format!("<div id=\"tag\">{}\
        {{{{#each {}}}}}{{{{#with this}}}}\
            {{{{#if desc}}}}\
                <div class=\"bubble\"><a href={{{{desc}}}}>{{{{name}}}}</a></div>\
            {{{{else}}}}\
                <div class=\"bubble\">{{{{name}}}}</div>\
            {{{{/if}}}}
{{{{/with}}}}{{{{/each}}}}</div>",
            hyperlink,
            name
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

fn construct_json_from_database(json: &mut serde_json::Value, tags: Vec<frictune::Tag>, db_conn: &str) {
    for main_tag in tags.iter() {
        let mut conn = frictune::db::crud::Db::sync_new(db_conn).unwrap();

        json[main_tag.name.trim_matches('\'')] = main_tag.qtr_sync(&mut conn)
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s,
                    "desc": frictune::Tag::new(s).qd_sync(&mut conn),
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

fn expand_expr_parse(db_uri: &str, tple: handlebars::template::HelperTemplate) -> frictune::Tag {
    let main_tag = frictune::Tag {
        name: format!("'{}'", tple.name.as_name().unwrap_or_default()),
        desc: match &tple.params[..] {
            [handlebars::template::Parameter::Literal(serde_json::value::Value::String(s)), ..] => Some(format!("'{}'", s)),
            _ => None,
        }
    };
    let vparam: Vec<(_, f32)> = tple.params
        .iter()
        .filter_map(|p| match p {
            handlebars::template::Parameter::Subexpression(elem) => {
                let title = elem.name();
                let empty_vec: Vec<handlebars::template::Parameter> = vec![];
                let params = elem.params().unwrap_or(&empty_vec);
                match &params[..] {
                    [
                        handlebars::template::Parameter::Literal(serde_json::value::Value::String(s)),
                        handlebars::template::Parameter::Literal(serde_json::value::Value::Number(n)),
                    ] if n.is_f64() => Some((frictune::Tag { name: format!("'{}'", title), desc: Some(format!("'{}'", s)) }, n.as_f64().unwrap() as f32)),
                    [
                        handlebars::template::Parameter::Literal(serde_json::value::Value::Number(n)),
                    ] if n.is_f64() => Some((frictune::Tag { name: format!("'{}'", title), desc: None }, n.as_f64().unwrap() as f32)),
                    [] => Some((frictune::Tag { name: format!("'{}'", title), desc: None }, 1.0)),
                    _ => { frictune::logger::lib_logger::de!("{:?} does not fit parse rule", params); panic!() },
                }
            },
            _ => { None },
        }).collect();
    update_database(db_uri, &main_tag, &vparam[..]);
    main_tag
}

fn update_database(db_uri: &str, leader: &frictune::Tag, trailers: &[(frictune::Tag, f32)]) {
    let mut conn = frictune::db::crud::Db::sync_new(db_uri).unwrap();
    
    for (tag, _) in trailers.iter() {
        tag.add_sync(&mut conn, HashMap::new());
    }

    leader.add_sync(&mut conn, trailers
        .into_iter()
        .map(|(tag, weight)| (tag.name.to_owned(), weight.to_owned()))
        .collect()
    );
}

fn get_bubble(name: &str, conn: &mut frictune::db::crud::Db) -> String {
    match frictune::Tag::new(name).qd_sync(conn) {
        Some(desc) if desc.contains("https") => format!(
            "<div class=\"bubble\">
                <a href={}>{}</a>
            </div>", desc, name),
        Some(_) | None => format!(
            "<div class=\"bubble\">{}</div>", name)
    }
}

fn fill_tag(main_tag: &frictune::Tag, conn: &mut frictune::db::crud::Db) -> String {
    let bubbles = main_tag.clone().qtr_sync(conn)
        .iter()
        .map(|s| get_bubble(s, conn))
        .collect::<Vec<_>>().join("\n");
    format!("<div id=\"tag\">
        <a href=\"{}\">{}</a>{}
    </div>",
        main_tag.desc.clone().unwrap_or_default(),
        main_tag.name,
        bubbles,
    )
}