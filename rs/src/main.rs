use deepl::{DeepLApi, Lang};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

// get bot working
// hookup postgres
// get deepl translate_text working
// validate args for /q /t
// validate chat and/or user id

struct Latin {
    // Represents a row in the database
    #[allow(unused)]
    id: i32,
    en: String,
    la: String,
    defn: String,
    fr: String,
    es: String,
    it: String,
}

// Type for building an in-memory dictionary mapping char in [a-z] to vector of english words
// e.g. {'a': ['absent',], ..., 'z': []}
// wrapped in Arc to be easily cloned
type Dictionary = Arc<HashMap<char, Vec<String>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting tg bot...");

    let bot = Bot::from_env();
    let dl = DeepLApi::with(
        &std::env::var("DEEPL_API_KEY").expect("failed to read api key from env")
    )
    .new();

    let db_path = std::env::var("DATABASE_URL").expect("failed to read db path from env");
    let db: PgPool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_path)
        .await.expect("failed to connect postgres");
    
    // Load dictionary into memory
    let mut map: HashMap<char, Vec<String>> = HashMap::new();
    let rows = sqlx::query_as!(Latin, "select * from latin")
        .fetch_all(&db)
        .await.expect("failed to read db");
    let mut count = 0_usize;
    for row in rows {
        let en = row.en;
        let first = (&en).chars().next().unwrap();
        map.entry(first).or_insert(vec![]).push(en);
        count += 1;
    }
    log::info!("Loaded {count} dictionary entries");

    let dict: Dictionary = Arc::new(map);

    Command::repl(
        bot, 
        move |bot, msg, cmd| respond(
            bot, msg, cmd, dl.clone(), db.clone(), dict.clone()
        )
    ).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Show help")]
    H,
    #[command(description = "Query a word for its latin origin")]
    Q,
    #[command(description = "Translate a word or phrase to a given target language")]
    T,
    #[command(description = "Get character usage for the current period")]
    U,
}

async fn respond(
    bot: Bot, 
    msg: Message, 
    cmd: Command, 
    dl: DeepLApi, 
    db: PgPool,
    dict: Dictionary,
) -> ResponseResult<()> 
{
    match cmd {
        Command::H => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::Q => {
            // Accept a list of words and iterate until we find a match
            // e.g. "/q foo absent"
            let text = msg.text().unwrap();

            let mut iter = text.split_whitespace();
            let _cmd = iter.next();

            // Match queries against dictionary words
            // note: we only return a result for the first match, as we can only return 1 message (Command match arms)
            let mut q = String::new();
            
            while let Some(str) = iter.next() {
                let s = str.to_owned();
                let first = str.chars().next().unwrap();
                let vec = dict.get(&first).unwrap();
                // TODO: validate first is in [a-z]
                if vec.contains(&s) {
                    q.push_str(&s);
                    break;
                }
            }
            
            let reply = if q.len() == 0 { String::from("None") } else {
                // Query database
                let row = sqlx::query_as!(
                    Latin,
                    "select * from latin where en = ($1)", q
                )
                .fetch_one(&db)
                .await.expect("failed to read db");
            
                // Build reply
                let la = row.la;
                let defn = row.defn;
                let fr = row.fr;
                let es = row.es;
                let it = row.it;
                format!("Here's what I found for {q},\nfrom the latin: {la}, {defn}\nmodern equivalents:\nfr {fr}\nes {es}\nit {it}")
            };
            
            bot.send_message(msg.chat.id, &reply).await?
        },
        Command::T => {
            // Call `translate_text` from deepl api
            let raw = msg.text().unwrap();
            let s = raw.strip_prefix("/t ").unwrap(); // handle case that user sends no args

            let reply = match dl.translate_text(s, Lang::IT).await {
                Ok(r) => {
                    let trans = r.translations;
                    if trans.len() > 0 {
                        String::from(&trans[0].text)
                    } else {
                        String::from("None")
                    }
                },
                Err(e) => {
                    log::debug!("API translate returned an error: {e}");
                    format!("bad request. refer to logs")
                }
            };
            
            bot.send_message(msg.chat.id, &reply).await?            
        },
        Command::U => {
            // Get usage stats
            let reply = match dl.get_usage().await {
                Ok(resp) => {
                    let count = resp.character_count;
                    let limit = resp.character_limit;
                    format!("{count} / {limit}")
                },
                Err(e) => {
                    log::debug!("API get_usage returned an error: {e}");
                    format!("bad request. refer to logs")
                }
            };
            
            bot.send_message(msg.chat.id, &reply).await?
        }
    };

    Ok(())
}

