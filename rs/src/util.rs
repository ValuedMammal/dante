use super::{config::*, Dictionary, Error, Latin};
use deepl::Lang;
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;

// Load regex patterns
lazy_static!(
    static ref RE_QUERY: Regex = Regex::new(r"^/q [A-Za-z]{2,}").unwrap();
    // starts with at least 2 ascii char

    static ref RE_TRANS: Regex = Regex::new(r"^/t ([A-Za-z]{2})[, ]([A-Za-z\-]{2,5}) ([\w,.!?'\u2019]{2}[\w,.!?'\u2019 ]+)$").unwrap();
    // allow ascii, latin, and whitespace. \u2019 is non-ascii apostrophe
    // alternatively: unicode ranges [\u0020-\u007F\u00C0-\u00FF]
    // TODO: support omitting source lang
);

/// Returns a reference to the first matching entry in `dict` if it exists, else `None`
pub fn try_from_dictionary(words: &[String], dict: Arc<Dictionary>) -> Option<&str> {
    if words.is_empty() {
        return None;
    }

    for word in words {
        let key = word.chars().next().expect("char is some");
        if dict.map.get(&key).expect("key exist").contains(word) {
            return Some(word);
        }
    }
    None
}

/// Validates a query candidate
pub fn is_valid_query(str: &str) -> bool {
    RE_QUERY.is_match(str)
}

/// Attempts to parse a target lang and string to send to the translator
pub fn parse_translatable(text: &str) -> Result<(Lang, Lang, String), Error> {
    let Some(caps) = RE_TRANS.captures(text) else {
        return Err(Error::Usage);
    };

    let src = caps[1].to_uppercase();
    let trg = caps[2].to_uppercase();
    let phrase = caps[3].to_string();

    let src_lang = Lang::try_from(&src).map_err(|e| Error::Language(e))?;
    let trg_lang = Lang::try_from(&trg).map_err(|e| Error::Language(e))?;

    Ok((src_lang, trg_lang, phrase))
}

/// Recursively searches the text, including substrings, for a similar row
/// using `LIKE %str%` syntax. Note, we return a row if the query resembles
/// a word from either the english or latin column
pub async fn query_greedy(words: Vec<String>, db: PgPool) -> Option<Latin> {
    let mut row: Option<Latin> = None;

    // track number of query attempts
    let mut count = 0usize;

    for mut s in words.iter().cloned() {
        if row.is_some() {
            break;
        }

        // wrap query in '%'
        let mut query = format!("%{}%", &s);

        // limit substring to at least 2 char
        while s.len() > 1 {
            row = sqlx::query_as!(
                Latin,
                "SELECT * FROM latin WHERE en LIKE ($1) OR la LIKE ($1)",
                query
            )
            .fetch_optional(&db)
            .await
            .expect("execute query");

            count += 1;
            if row.is_some() {
                break;
            }
            // pop last char and rebuild query
            s.pop();
            query = format!("%{}%", &s);
        }
    }

    let info = match row {
        Some(ref latin) => {
            let en = &latin.en;
            format!("Found result for {en} in {count} queries")
        }
        None => format!("{count} queries returned None for {words:?}"),
    };

    log::info!("{info}");
    row
}

/// Whether the given `msg` comes from an authorized chat id
pub fn is_authorized(msg: &teloxide::types::Message) -> bool {
    let chat_id: i64 = msg.chat.id.0;
    ALLOW.contains(&chat_id)
}
