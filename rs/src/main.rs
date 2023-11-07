//#![allow(unused)]
use crate::util::{
    is_authorized, is_valid_query, parse_translatable, query_greedy, try_from_dictionary,
};
use deepl::DeepLApi;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

mod config;
mod util;

/// The global app formality preference
const DEFAULT_FORMALITY: deepl::Formality = deepl::Formality::PreferLess;

/// Represents a row in the database
pub struct Latin {
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
#[derive(Debug, Clone)]
pub struct Dictionary {
    inner: Arc<HashMap<char, Vec<String>>>,
}

/// My errors
#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    Language(String),
    Usage,
}

impl Dictionary {
    /// Creates a `Dictionary` from the given `map`
    fn new(map: HashMap<char, Vec<String>>) -> Self {
        Self {
            inner: Arc::new(map),
        }
    }

    /// Returns a reference to the value corresponding to the given `key`
    /// if it exists, else `None`
    fn get(&self, key: &char) -> Option<&Vec<String>> {
        self.inner.get(key)
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Database(e) => e.to_string(),
            Self::Usage => "Usage: /t <source lang> <target lang> <text>".to_string(),
            Self::Language(e) => format!("❔ Unrecognized language: {e}"),
        };
        f.write_str(&s)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    log::info!("Starting tg bot...");

    let dl_auth = env!("DEEPL_API_KEY");
    let dl = DeepLApi::with(dl_auth).new();

    let db_url = env!("DATABASE_URL");
    let db: PgPool = PgPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await?;

    // Load dictionary into memory
    let mut map = HashMap::<char, Vec<String>>::new();
    let rows = sqlx::query_as!(Latin, "select * from latin")
        .fetch_all(&db)
        .await?;

    let mut count = 0_usize;
    for row in rows {
        let en = row.en;
        let key = en.chars().next().expect("char is some");
        map.entry(key).and_modify(|v| v.push(en)).or_insert(vec![]);
        count += 1;
    }
    log::info!("Loaded {count} dictionary entries");

    let dict = Dictionary::new(map);

    Command::repl(bot, move |bot, msg, cmd| {
        respond(bot, msg, cmd, dl.clone(), db.clone(), dict.clone())
    })
    .await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
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
    dict: Dictionary,
) -> ResponseResult<()> {
    if !is_authorized(&msg) {
        return Ok(());
    }

    match cmd {
        // See command descriptions
        Command::H => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        // Extra help info
        Command::Info => {
            let help = "I am Dante, the romantic. I'll tell you whether an English word has roots in the Latin language. /q Where applicable, I include modern analogs for the word of interest (currently \"FR\", \"ES\", & \"IT\"). I can also translate words to and from various languages. /t\n\nSee the commands list for usage and syntax. /h \n\ntips: A query result contains a grammatical part (noun, adj, verb) that refers to the latin root, and not necessarily the english word. However, I will do my best to ensure the 'descendants' correspond grammatically to the english term.\n\nKeep in mind, the 'descendants' aim to capture lexical forms that most closely resemble their latin origin, but since the meaning of words drifts over time, they may no longer track semantically or are seldom used in modern parlance. For more dynamic translations, the translate command should come in handy.\n\nBy the way, we're adding words to the dictionary all the time - let us know if you believe a common English/Latin pair is missing.\n\nOk enough preamble,\nCarpe Diem!";

            //let chat_id = msg.chat.id.0;
            //bot.send_message(msg.chat.id, format!("{chat_id}")).await?
            bot.send_message(msg.chat.id, help).await?
        }
        // Iterate over words in text and attempt to pull a row from the database
        Command::Q => {
            let text = msg.text().expect("msg text");
            let words: Vec<String> = if is_valid_query(text) {
                let text = text.strip_prefix("/q ").expect("strip prefix").to_string();
                text.split_whitespace().map(|s| s.to_lowercase()).collect()
            } else {
                vec![]
            };

            // Search dictionary for exact match
            let exact = try_from_dictionary(&words, dict.clone());

            #[rustfmt::skip]
            let reply = if words.is_empty() {
                // no query candidates to use
                "None".to_string()
            } else if let Some(en) = exact {
                // search exact
                let row = sqlx::query_as!(Latin, "SELECT * FROM latin WHERE en = ($1)", en)
                    .fetch_one(&db)
                    .await
                    .expect("execute query");

                let Latin { id: _, en, la, defn, fr, es, it } = row;
                format!("Here's what I've got for {en},\nfrom the latin: {la}, {defn}\ndescendants:\nfr {fr}\nes {es}\nit {it}")
            } else {
                // greedily search for a similar row
                match query_greedy(words, db.clone()).await {
                    Some(latin) => {
                        let Latin { id: _, en, la, defn, fr, es, it } = latin;
                        format!("❔ Meno male, I found something similar: {en},\nfrom the latin: {la}, {defn}\ndescendants:\nfr {fr}\nes {es}\nit {it}")
                    }
                    None => "None".to_string(),
                }
            };

            bot.send_message(msg.chat.id, &reply).await?
        }
        // Translate text
        Command::T => {
            let text = msg.text().expect("msg text");

            let reply = match parse_translatable(text) {
                Err(e) => e.to_string(),
                Ok((src, trg, s)) => {
                    log::info!("Requesting translate for {s}");
                    match dl
                        .translate_text(s, trg)
                        .source_lang(src)
                        .formality(DEFAULT_FORMALITY)
                        .await
                    {
                        Ok(res) => {
                            let translation = &res.translations[0];
                            translation.text.to_string()
                        }
                        Err(e) => {
                            log::error!("DeepL translate: {e}");
                            "❗️ bad request. refer to logs".to_string()
                        }
                    }
                }
            };

            bot.send_message(msg.chat.id, &reply).await?
        }
        // Get usage stats
        Command::U => {
            let reply = match dl.get_usage().await {
                Ok(resp) => {
                    let count = resp.character_count;
                    let limit = resp.character_limit;
                    format!("{count} / {limit}")
                }
                Err(e) => {
                    log::error!("DeepL usage: {e}");
                    "❗️ bad request. refer to logs".to_string()
                }
            };
            bot.send_message(msg.chat.id, &reply).await?
        }
    };

    Ok(())
}

#[cfg(test)]
mod test;
