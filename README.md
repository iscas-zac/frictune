# FricTune, the tag linker

 I collect hyperlinks and write articles, and store them somewhere. But how can I find a helpful one in the archive? The idea is very simple.

1. Link everything with a tag (tune).

2. Link tags.

3. Find through the link.

4. All with a weight (friction).

## examples

### command-line interaction

Right now, the program’s main function compiles to a commandline tool,
which keeps a database named tags.db (whose name is supposedly specified in a configuration file config.toml, but it does not seem to work for now).

The program can add a tag (a string) with an optional description (an optional string) into the database (`frictune add abc`), link two tags with a weight (a decimal) (`frictune link abc def —weight 0.5`),and delete a tag.

### HTML construction

There is a second binary built with `cargo run --bin tune_html`.It will be used more as the main functionalities are not complete, which is from another main function in the `/bin` folder, `tune_html.rs`.  This function compiles to another commandline tool, which does some more non-trivial work. It accepts a [handlebars](https://github.com/sunng87/handlebars-rust) template (default to `template.hbs`), a content file (default to `temp.txt`), a SQLite database (default to `tags.db`), and output a filled HTML file (default to `b.html`). The names are only for temporary use.

The program parses the content file, puts the tags into the database, and output HTML with the hyperlinks followed by related tags. The parsing rules are explained in the `tune_html.rs` file. For an example, see [this post in Chinese](https://peb.pages.dev/writing/experiment230505) and hover your cursor on the links.

### WebAssembly database interaction

There is another file to compile to, the `wasm32-unknown-unknown` target, which I think is the most portable and least dependent. This part is work in progress (WIP).

For now, the web interaction involves the [Dioxus](https://dioxuslabs.com/) framework (for its integration with WASM and multi-platform support), and the database part [GlueSQL](https://github.com/gluesql/gluesql)'s `MemoryStorage` (which seems the only database choice to support `wasm32-unknown-unknown` target).

As `MemoryStorage` is "non-persistent" by design, the file interaction is limited, and I will only read a file `gluesql` without any writing for now.

Currently, I implement the main APIs on the webpage with no further wrappers. Run `dioxus build`, change the root folder and start a web server to see the results.

## Architecture (WIP)

There are roughly three layers in the codebase, the database layer,
the tag operation layer and the user interface layer, essentially
the frontend/backend partition.

The tag operation layer binds the database and the user interface layer,
and provides the APIs to the user interface layer. For now it sits
in `lib.rs` and consists simply of the `frictune::Tag` functions.

The database can switch between GlueSQL and SQLite bindings (the
`crud/gluesql::Database` struct), as a result of multi-target
support. They are switched by condition macros, which also results
in some problems.

The user interface layer also support two targets for now, the CLI
application and the webpage. The CLI application `frictune` can add,
delete, query information about and modify a tag. It comes with two
companions (in `/bin` for now), `database_mig` can read a SQLite
database and construct a GlueSQL database accordingly, and `tune_html`
can read a Handlebars-like format to give a webpage demo.

The webpage also implements the simple data manipulations (but can not
write back to local files), and can draw graphs according to the data.
The graph is currently drawn by [D3.js](https://d3js.org/) and [d3-force](https://github.com/d3/d3-force).

TL; DR: the architecture is like this,


| CLI(frictune (basic)/tune_html (reader)/database_mig (compat)) | webpage |

| frictune::Tag |

| GlueSQL | SQLite |

## current APIs (WIP)

The main APIs are the tag functions in `lib.rs`. They communicate
with underlying database wrappers and give query results.

For now, some of the functions have sync and async versions, of
which the sync ones have 'sync' in their names.

The input values of them are usually a mutable reference of the
database and a tag, and the return values vary. I have not decided
whether to return some `Result`s through the APIs.

As to usages, `frictune` can add, delete, query about a tag, and
add new link between tags for now.

> **Warning**

The APIs are far from stable for now.

## TODOs

### Next step

Web UI: better presentation
Documentation improvement
Testcases

### Others

The project is under **lazy** development. A few features are to be added:

- [ ] testcases
    - [ ] assertions

- [ ] better error handling style

- [ ] better rust code style (maybe not)
    - [x] parameter style, (reference instead of move, [] instead of vec)

- [ ] configurations
    - [x] command line read
    - [ ] web read
    - [ ] file

- [ ] database layers
    - [x] complete CRUD APIs
    - [ ] unified data interaction
    - [ ] persistency of `MemoryStorage` interaction

- [ ] UIs
    - [x] Command Line Interface
        - [ ] REPL
    - [x] render to static HTML
    - [ ] web UI
        - [x] WASM data APIs

- [ ] automated workflows (with file folders / websites)

- [ ] async / multi-user features (less probable)

- [ ] docs

- [ ] a good formula to infer the weights between any two tags

## made with rust / sqlx

I'm trying to learn rust, and this is an exercise.