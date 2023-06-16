use deepl::Lang;
use lazy_static::lazy_static;
use regex::Regex;
use super::Dictionary;
use std::sync::Arc;

/// Returns a reference to the matching element in `words`
pub fn from_dictionary_option(words: &Vec<String>, dict: Arc<Dictionary>) -> Option<&str> {
    if words.len() == 0 { return None }
    
    let mut i = 0_usize;
    for w in words {
        // let s = w.to_owned();
        let ch = w.chars().next().unwrap();
        if dict.map.get(&ch).unwrap().contains(w) {
            // Ok to unwrap, as `Dictionary::new` populates all keys,
            // and `is_query_candidate` constrains `words` to ascii only
            let s = &words[i];
            return Some(s)            
        }
        i += 1;
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
pub fn parse_translation_candidate(str: &str) -> Result<(deepl::Lang, String), ()> {
    lazy_static!(
        static ref RE: Regex = Regex::new(r"^/t ([A-Za-z\-]{2,5}) ([A-Za-z\u00C0-\u00FF]{2}[A-Za-z\u00C0-\u00FF\s]*)").unwrap();
        // phrase allows ascii, latin-1, and whitespace
        // starts with at least two non-space chars
    );

    let Some(caps) = RE.captures(str) else {
        return Err(())
    };
    
    let trg = (&caps[1]).to_uppercase();
    let Ok(target_lang) = Lang::try_from(&trg) else {
        return Err(())
    };
    
    let phrase = String::from(&caps[2]);
    Ok((target_lang, phrase))
}
    
    