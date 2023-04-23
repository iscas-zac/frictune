use std::{io::{BufReader, Read}, fs::File, collections::HashMap};

use handlebars::{Handlebars, handlebars_helper};
use serde_json::json;

fn main() { // TODO: a dirty implementation. later for refactorization
    let mut reg = Handlebars::new();
    if let Err(e) = reg.register_template_file("page", "./template.hbs")
    { println!("{}", e); panic!() }

    let content = match read_content("./temp.txt") {
        Ok(s) => {s},
        Err(e) => { println!("{}", e); panic!() }
    };

    handlebars_helper!(query: |tag: String, link: String, *args| {
        let mut conn = frictune::db::crud::Db::sync_new("./tags.db").unwrap();
        let src = frictune::Tag { name: format!("'{}'", tag), desc: Some(format!("'{}'", link)) };
        src.add_sync(&mut conn, HashMap::new());
        for index in (2..args.len()).step_by(3) {
            let dst = frictune::Tag { name: format!("'{}'", args[index].as_str().unwrap()), desc: Some(format!("'{}'", args[index + 1])) };
            println!("{:?}", &args[index..index + 3]);
            dst.add_sync(&mut conn, HashMap::new());
            src.link_sync(&mut conn, args[index].as_str().unwrap(), args[index + 2].as_f64().unwrap() as f32);
        }
        let bubbles = frictune::Tag::new(&tag).qtr_sync(&mut conn)
        .iter()
        .map(|s|
            {// TODO: check the desc is a link below
                match frictune::Tag::new(s).qd_sync(&mut conn) {
                    Some(desc) => format!(
                        "
                        <div class=\"bubble\">
                            <a href={}>
                                {}
                            </a>
                        </div>", desc, s),
                    None => format!(
                        "
                        <div class=\"bubble\">
                                {}
                        </div>", s)
                }
                
            }).collect::<Vec<_>>().join("\n");
        format!("<div id=\"tag\">
            <a href={}>
                {}
            </a>{}
        </div>",
            link,
            tag,
            bubbles,
        )
    });
    reg.register_escape_fn(|a| { a.to_owned() });
    reg.register_helper("query", Box::new(query));
    
    match reg.register_partial("content", content.lines()
        .map(|s| format!("<p>{}</p>", s))
        .collect::<Vec<_>>()
        .join("\n"))
    {
        Ok(s) => s,
        Err(e) => { println!("{}", e); panic!() }
    };
    
    match reg.render("page", &json!({
        "date": &chrono::Local::now()
            .date_naive()
            .format("%Y/%m/%d")
            .to_string(),
        "name": "new post",
    }))
    {
        Ok(s) => println!("{}", s),
        Err(e) => { println!("{}", e); panic!() }
    };
    reg.unregister_escape_fn();
    
}

fn read_content(name: &str) -> Result<String, std::io::Error> {
    let mut reader = BufReader::new(File::open(name)?);

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    Ok(buf)
}