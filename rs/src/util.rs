// #![allow(unused)]
use deepl::Lang;
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::PgPool;
use super::Latin;

use super::Dictionary;
use std::sync::Arc;

/// If option is Some, returns a reference to the matching element in `words`
pub fn from_dictionary_option(words: &Vec<String>, dict: Arc<Dictionary>) -> Option<&str> {
    if words.len() == 0 { return None }
    
    for w in words {
        let key = w.chars().next().unwrap();
        if dict.map.get(&key).unwrap().contains(w) {
            // Ok to unwrap, as `Dictionary::new` populates all keys,
            // and `is_query_candidate` constrains `words` to ascii only
            return Some(w)
        }
    }
    None
}

/// Validates a query candidate
pub fn is_query_candidate(str: &str) -> bool {
    lazy_static!(
        static ref RE: Regex = Regex::new(r"^/q [A-Za-z]{2,}").unwrap();
        // starts with at least 2 ascii char
    );
    RE.is_match(str)
}

/// Attempts to parse a target lang and word/phrase to send to the translator
pub fn parse_translation_candidate(text: &str) -> Result<(Lang, Lang, String), i32> {
    lazy_static!(
        static ref RE: Regex = Regex::new(r"^/t ([A-Za-z]{2})[, ]([A-Za-z\-]{2,5}) ([\S&&[\u0020-\u007F\u00C0-\u00FF]{2}][\u0020-\u007F\u00C0-\u00FF\s]*)$").unwrap();
        // phrase allows ascii, latin-1, and whitespace
        // must start with at least two non-space chars
        // TODO: support omitting source lang 
    );

    let Some(caps) = RE.captures(text) else {
        return Err(-1)
    };
    
    let src = (&caps[1]).to_uppercase();
    let Ok(src_lang) = Lang::try_from(&src) else {
        return Err(-2)
    };
    let trg = (&caps[2]).to_uppercase();
    let Ok(trg_lang) = Lang::try_from(&trg) else {
        return Err(-3)
    };
    
    let phrase = String::from(&caps[3]);
    Ok((src_lang, trg_lang, phrase))
}

/// Recursively searches the text, including substrings, for a similar row
pub async fn query_greedy(words: Vec<String>, db: PgPool) -> Option<Latin> {
    let mut row: Option<Latin> = None;

    let mut iter = words.iter();
    let mut count = 0_usize;
    
    while let Some(s) = iter.next() {
        if row.is_some() { break } // success
        
        // get owned value
        let mut en = String::from(s);
        
        // wrap query in '%'
        let mut q = String::from('%');
        q.push_str(&en);
        q.push('%');
    
        while en.len() > 1 {
            // limit substring to at least 2 char
            row = sqlx::query_as!(Latin, "SELECT * FROM latin WHERE en LIKE ($1) OR la LIKE ($1)", q)
                .fetch_optional(&db)
                .await.expect("failed to read db");
            count += 1;
            
            if row.is_some() { break } else {
                // pop last char and rebuild query
                let _ = en.pop();
                q.clear();
                q.push('%');
                q.push_str(&en);
                q.push('%');
            }
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
    