use super::{config::*, Dictionary, Latin};
use deepl::Lang;
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;
use teloxide::types::Message;

// Load regex patterns
lazy_static!(
    static ref RE_QUERY: Regex = Regex::new(r"^/q [A-Za-z]{2,}").unwrap();
    // starts with at least 2 ascii char

    static ref RE_TRANS: Regex = Regex::new(r"^/t ([A-Za-z]{2})[, ]([A-Za-z\-]{2,5}) ([\w.!?'\u2019]{2}[\w.!?'\u2019 ]+)$").unwrap();
    // alt: unicode ranges [\u0020-\u007F\u00C0-\u00FF]
    // allow ascii, latin, and whitespace. \u2019 is non-ascii apostrophe
    // TODO: check user didn't send all whitespace
    // TODO: support omitting source lang
);

/// If option is Some, returns a reference to the matching element in `words`
pub fn from_dictionary_option(words: &Vec<String>, dict: Arc<Dictionary>) -> Option<&str> {
    if words.is_empty() {
        return None;
    }

    for w in words {
        let key = w.chars().next().unwrap();
        if dict.map.get(&key).unwrap().contains(w) {
            // Ok to unwrap, as `Dictionary::new` populates all keys,
            // and `is_query_candidate` constrains `words` to ascii only
            return Some(w);
        }
    }
    None
}

/// Validates a query candidate
pub fn is_valid_query(str: &str) -> bool {
    RE_QUERY.is_match(str)
}

/// Attempts to parse a target lang and string to send to the translator
pub fn parse_translatable(text: &str) -> Result<(Lang, Lang, String), i32> {
    let Some(caps) = RE_TRANS.captures(text) else {
        return Err(-1)
    };

    let src = caps[1].to_uppercase();
    let Ok(src_lang) = Lang::try_from(&src) else {
        return Err(-2)
    };
    let trg = caps[2].to_uppercase();
    let Ok(trg_lang) = Lang::try_from(&trg) else {
        return Err(-3)
    };

    let phrase = caps[3].to_string();
    Ok((src_lang, trg_lang, phrase))
}

/// Recursively searches the text, including substrings, for a similar row
pub async fn query_greedy(words: Vec<String>, db: PgPool) -> Option<Latin> {
    let mut row: Option<Latin> = None;

    let mut count = 0_usize;

    for s in &words {
        if row.is_some() {
            break;
        }

        // get owned value
        let mut en = String::from(s);

        // wrap query in '%'
        let mut q = String::from('%');
        q.push_str(&en);
        q.push('%');

        while en.len() > 1 {
            // limit substring to at least 2 char
            row = sqlx::query_as!(
                Latin,
                "SELECT * FROM latin WHERE en LIKE ($1) OR la LIKE ($1)",
                q
            )
            .fetch_optional(&db)
            .await
            .expect("failed to read db");
            count += 1;

            if row.is_some() {
                break;
            }

            // pop last char and rebuild query
            let _ = en.pop();
            q.clear();
            q.push('%');
            q.push_str(&en);
            q.push('%');
        }
    }
    
    if let Some(latin) = row {
        let en = &latin.en;
        log::info!("Found result for {en} in {count} queries");
        Some(latin)
    } else {
        log::info!("{count} queries returned None for {words:?}");
        None
    }
}

pub fn is_authorized(msg: &Message) -> bool {
    let chat_id: i64 = msg.chat.id.0;
    ALLOW.contains(&chat_id)
}
