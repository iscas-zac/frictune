# FricTune, the tag linker

The idea is very simple. I collect hyperlinks and write articles, and store them somewhere. But how can I find a helpful one in the archive?

1. Link everything with a tag (tune).

2. Link tags.

3. Find through the link.

4. All with a weight (friction).

## made with rust / sqlx

I'm trying to learn rust, and this is an exercise.

## the current APIs

The `main()` function is for tests only for now. Most APIs lie in the lib.rs file, and cover the data operations.

`pub async fn add_tag<'a>(&self, db: &mut db::crud::Db, weights: HashMap<String, f32>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>`: add a tag to the database, with an option to initialize the links with the `weight` parameter.

`pub async fn update_all_links(db: &mut db::crud::Db)`: update the links with a formula. Some link weights are designated by the user, the others will be inferred. (effectiveness in progress)

The other functions should speak for themselves.

## TODOs

The project is under <span style="background: red;">lazy</span> development. A few features are to be added:

- [ ] testcases

- [ ] better error handling style

- [ ] better rust code style (maybe not)
    - [ ] parameter style, (reference instead of move, [] instead of vec)

- [x] complete CRUD APIs

- [ ] UIs

- [ ] automated workflows (with file folders / websites)

- [ ] async / multi-user features (less probable)

- [ ] docs

- [ ] a good formula to infer the weights between any two tags