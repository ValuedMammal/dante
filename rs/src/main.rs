// #![allow(unused)]
use crate::util::{from_dictionary_option, is_query_candidate, parse_translation_candidate, query_greedy};
use deepl::DeepLApi;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

mod util;

pub struct Latin {
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

/// Type for building an in-memory dictionary mapping ascii char to a vector of english words
// e.g. {'a': ['absent',], ..., 'z': []}
pub struct Dictionary { map: HashMap<char, Vec<String>> }

impl Dictionary {
    /// Creates a new empty dictionary containing a map of words keyed by a single char in [a-z]
    fn new() -> Self {
        let mut map = HashMap::new();

        // initialize all keys/values
        for i in 0_u8..26 {
            let key = ('a' as u8 + i) as char;
            let val: Vec<String> = vec![];
            map.insert(key, val);
        }
        Dictionary { map }
    }

    /// Returns all the dictionary keys whose value is the empty vec
    #[allow(dead_code)]
    fn get_empty(&self) -> Vec<char> {
        let map = &self.map;

        map.iter()
        .filter(|(_, ref v)| v.len() == 0)
        .map(|(&k, _)| k)
        .collect()
    }
}

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
    let mut d = Dictionary::new();
    let rows = sqlx::query_as!(Latin, "select * from latin")
        .fetch_all(&db)
        .await.expect("failed to read db");
    let mut count = 0_usize;
    for row in rows {
        let en = row.en;
        let key = (&en).chars().next().unwrap();
        d.map.get_mut(&key).unwrap().push(en);
        // Ok to unwrap, due to call to dict `new`
        // assumes first char of english words are in ascii [a-z]
        count += 1;
    }
    log::info!("Loaded {count} dictionary entries");

    let dict = Arc::new(d);

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
    #[command(description = "More info")]
    Info,
    #[command(description = "Query: '/q <word>'")]
    Q,
    #[command(description = "Translate: '/t <source lang> <target lang> <text>'")]
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
    dict: Arc<Dictionary>,
) -> ResponseResult<()> 
{
    fn is_authorized(msg: &Message) -> bool {
        let chat_id = msg.chat.id.0;
        let allow = vec![
            2027093603_i64, // tyler PM
            -961117056, // lengua group
        ];
        allow.contains(&chat_id)
    }
    if !is_authorized(&msg) { 
        //bot.send_message(msg.chat.id, "unauthorized").await?;
        return Ok(())
    }
    
    match cmd {
        Command::H => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::Info => {
            let help = "I am Dante, the romantic. I'll tell you whether an English word has roots in the Latin language. /q Where applicable, I include modern analogs for the word of interest (currently \"FR\", \"ES\", & \"IT\"). I can also translate words to and from various languages. /t\n\nSee the commands list for usage and syntax. /h \n\ntips: A query result contains a grammatical part (noun, adj, verb) that refers to the latin root, and not necessarily the english word. However, I will do my best to ensure the 'descendants' correspond grammatically to the english term.\n\nKeep in mind, the 'descendants' aim to capture lexical forms that most closely resemble their latin origin, but since the meaning of words drifts over time, they may no longer track semantically or are seldom used in modern parlance. For more dynamic translations, the translate command should come in handy.\n\nBy the way, we're adding words to the dictionary all the time - let us know if you believe a common English/Latin pair is missing.\n\nOk enough preamble,\nCarpe Diem!";
            
            //let chat_id = msg.chat.id.0;
            //bot.send_message(msg.chat.id, format!("{chat_id}")).await?
            bot.send_message(msg.chat.id, help).await?
        },            
        Command::Q => {
            // Iterate over words in text and attempt to pull a row from the database
            // e.g. "/q foo bar absent"
            let text = msg.text().unwrap();
            let words: Vec<String> = if is_query_candidate(text) {
                let text = text.strip_prefix("/q ").unwrap().to_string();
                text.split_whitespace().map(|s| s.to_lowercase()).collect()
            } else {
                vec![]
            };
            dbg!(words.len());

            // See if we have an exact match by searching in-memory dictionary
            let exact = from_dictionary_option(&words, dict.clone());

            let reply = if words.len() == 0 {
                // no query candidates to use
                format!("None")
            } else if let Some(en) = exact {
                // search exact
                let row = sqlx::query_as!(Latin, "SELECT * FROM latin WHERE en = ($1)", en)
                    .fetch_one(&db)
                    .await.expect("dict entry should return a db row");
                
                let Latin { id: _, en, la, defn, fr, es, it } = row;
                format!("Here's what I've got for {en},\nfrom the latin: {la}, {defn}\ndescendants:\nfr {fr}\nes {es}\nit {it}")
            } else {
                // greedily search for a similar row
                match query_greedy(words, db.clone()).await {
                    Some(latin) => {
                        let Latin { id: _, en, la, defn, fr, es, it } = latin;
                        format!("❔ Meno male, I found something similar: {en},\nfrom the latin: {la}, {defn}\ndescendants:\nfr {fr}\nes {es}\nit {it}")
                    },
                    None => {
                        format!("None")
                    }
                }
            };
            
            bot.send_message(msg.chat.id, &reply).await?
        },
        Command::T => {
            // Call `translate_text` from deepl api
            let text = msg.text().unwrap();
            
            let reply = match parse_translation_candidate(text) {
                Err(-1) => format!("Usage: /t <source lang> <target lang> <text>"),
                Err(-2) => format!("❗️ unknown source lang"),
                Err(-3) => format!("❗️ unknown target lang"),
                Err(_) => format!("❗️ unknown error occurred"), // unreachable assuming we've covered each err code
                Ok((src, trg, s)) => {
                    log::info!("Requesting translate for {s}");
                    match dl.translate_text(s, trg).source_lang(src).await {
                        Ok(r) => {
                            let trans = r.translations;
                            String::from(&trans[0].text)
                        },
                        Err(e) => {
                            log::debug!("DeepL translate returned an error: {e}");
                            format!("❗️ bad request. refer to logs")
                        }
                    }
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
                    format!("❗️ bad request. refer to logs")
                }
            };
            bot.send_message(msg.chat.id, &reply).await?
        }
    };

    Ok(())
}


#[cfg(test)]

    mod test;

