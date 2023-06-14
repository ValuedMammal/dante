use deepl::{DeepLApi, Lang};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use crate::util::is_word;

mod util;

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
        Command::H => {
            let help = "I am Alejandro, the romantic. I'll tell you whether an English word has roots in the Latin language./q Where applicable, I include modern translations for the word of interest (currently \"FR\", \"ES\", & \"IT\"). I can also translate words to and from various languages./t\n\nSee the commands list for usage and syntax.\n\ntips: A query result contains a grammatical part (noun, adj, verb) that refers to the latin root, and not necessarily the english word. However, I will do my best to ensure the 'modern equivalents' correspond grammatically to the english term.\n\nKeep in mind, the 'modern equivalents' aim to capture lexical forms that most closely resemble their latin origin, but since the meaning of words has drifted over time, they may no longer track semantically or are only rarely used. For more dynamic translations, the translate command should come in handy.\n\nBy the way, we're adding words to the dictionary all the time - let us know if you believe a common English/Latin pair is missing.\n\nOk enough preamble,\nCarpe Diem!";
            
            //bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
            bot.send_message(msg.chat.id, help).await?
        },
        Command::Q => {
            // 
            // e.g. "/q foo absent"
            let text = msg.text().unwrap();
            let words: Vec<String> = text.split_whitespace()
                .filter(|s| is_word(s))
                .map(|s| s.to_lowercase())
                .collect();
            
            let mut q = String::new(); // the query param

            // Check for match against dictionary entries
            let mut is_entry = false;
            for str in &words {
                let s = str.to_owned();
                let ch = (&s).chars().next().unwrap();
                if dict.get(&ch).unwrap().contains(&s) {
                    q.push_str(&s);
                    is_entry = true;
                    break;
                }
            }

            let reply = if words.len() == 0 {
                // user passed no args
                format!("None")
            } else if is_entry {
                // search exact
                let row = sqlx::query_as!(Latin, "SELECT * FROM latin WHERE en = ($1)", q)
                    .fetch_one(&db)
                    .await.expect("dict entry should return a db row");
                
                let Latin { id: _, en, la, defn, fr, es, it } = row;
                format!("Here's what I've got for {en},\nfrom the latin: {la}, {defn}\nmodern equivalents:\nfr {fr}\nes {es}\nit {it}")
            } else {
                // search similar, using first char of first word
                // shouldn't fail assuming all dictionary buckets are populated
                let elem = &words[0];
                let ch = elem.chars().next().unwrap();
                q.push_str(&format!("{ch}%"));

                let row = sqlx::query_as!(Latin, "SELECT * FROM latin WHERE en LIKE ($1)", q)
                    .fetch_optional(&db)
                    .await.expect("failed to read db");
                
                match row {
                    Some(row) => {
                        let Latin { id: _, en, la, defn, fr, es, it } = row;
                        format!("I found something similar: {en},\nfrom the latin: {la}, {defn}\nmodern equivalents:\nfr {fr}\nes {es}\nit {it}")
                    },
                    None => format!("None")
                }
            };
            
            bot.send_message(msg.chat.id, &reply).await?
        },
        Command::T => {
            // Call `translate_text` from deepl api
            let text = msg.text().unwrap();
            let mut iter = text.split_whitespace();
            let _cmd = iter.next();
            let trg = iter.next();
            
            let mut s = String::new();
            while let Some(str) = iter.next() {
                // recombine, inserting single space
                // TODO: write a regex for this ??
                if !is_word(str) { continue }
                s.push_str(&format!("{str} "));
            }
            
            let reply = if trg == None || s.len() == 0 { 
                format!("None")
            } else {
                let trg = trg.unwrap().to_uppercase();
                if let Ok(lang) = Lang::try_from(&trg) {
                    match dl.translate_text(s, lang).await {
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
                    }
                } else { 
                    format!("unknown target lang")
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
