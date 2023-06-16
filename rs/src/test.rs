use std::sync::Arc;
use deepl::Lang;
use sqlx::postgres::{PgPool, PgPoolOptions};
use super::{
    Dictionary,
    Latin,
    util::{from_dictionary_option, is_query_candidate, parse_translation_candidate},
};

#[test]
fn valid_query() {
    let bad = "/q";
    assert!(!is_query_candidate(bad));
    
    let raw = "/q foo bar absent";
    assert!(is_query_candidate(raw));
}

#[test]
fn valid_translatable() {
    let test_vec: Vec<(&str, (Lang, String))> = vec![
        (
            "/t de good morning",
            (Lang::DE, "good morning".to_string())
        ),
        (
            "/t en-us dos lápices",
            (Lang::EN_US, "dos lápices".to_string())
        ),
    ];
    
    for t in test_vec {
        let result = parse_translation_candidate(t.0);

        assert!(result.is_ok());
        let (trg, s) = t.1;
        assert_eq!(result.unwrap(), (trg, s));
    }
}

#[tokio::test]
async fn find_closest_or_none() {
    // recursively perform query on vector of words
    // consuming each word by popping a char from the back
    // returning first match on db row, else None
    
    let db_path = std::env::var("DATABASE_URL").expect("failed to read db path from env");
    let db: PgPool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_path)
        .await.expect("failed to connect postgres");

    let test_vec: Vec<(Vec<String>, Option<String>)> = vec![
        (
            // walks the vector,
            // and strips trailing char
            vec![String::from("zzz"), String::from("spelunkerzzz")],
            Some(String::from("spelunker"))
        ),
        (
            // finds LIKE with 'foo%' syntax
            vec![String::from("foo")],
            Some(String::from("focus"))
        ),
        (
            // no result for empty buckets
            vec![String::from("zzz"), String::from("xxx")],
            None
        ),
    ];

    for case in test_vec {

        // note: the below code doesn't belong to a function, so the same algorithm must be mirrored in main.rs
        // fn might look like:
            // let row: Option<Latin> = query_greedy(&words, db.clone());
        let v = case.0;
        let mut iter = v.iter();
        let expect = case.1;

        let mut en: Option<String> = None;

        while let Some(s) = iter.next() {
            if en.is_some() { break } // happy path

            let mut query = s.clone();
            query.push('%');

            loop  {
                if let Some(latin) = sqlx::query_as!(Latin, "SELECT * FROM latin WHERE en LIKE ($1)", query).fetch_optional(&db).await.expect("failed to read db") {
                    en = Some(latin.en);
                    break
                } else if query.len() == 2 {
                    // avoid popping the last element
                    // and go to next word
                    break 
                } else {
                    // continue
                    query.pop(); // %
                    query.pop();
                    query.push('%');
                }
            }
        }
        assert_eq!(en, expect);
    }
}

#[test]
fn dictionary() {
    //struct Dictionary { map: HashMap<char, Vec<String>> }
    let mut d = Dictionary::new();
    
    // all keys present, with empty vectors
    for i in 0_u8..26 {
        let key = ('a' as u8 + i) as char;
        let val = d.map.get(&key);
        assert!(val.is_some());
    }

    let empty = d.get_empty();
    assert_eq!(26, empty.len());

    // push a value to an entry
    let s = "foo".to_string();
    d.map.get_mut(&'f').unwrap().push(s);

    let empty = d.get_empty();
    assert_eq!(25, empty.len());
}
    
#[tokio::test]
async fn match_dictionary() {

    // test iterate dictionary match 
    // e.g. [foo, bar, absent] yields 'absent'

    let mut d = Dictionary::new();
    d.map.get_mut(&'s')
        .unwrap()
        .push(String::from("spelunker"));

    let dict = Arc::new(d);

    let mut words = vec![
        "foo".to_string(),
        "bar".to_string(),
        "spelunker".to_string(),
    ];

    // match returned
    let exact = from_dictionary_option(&words, dict.clone());
    assert!(exact.is_some());
    assert_eq!(exact.unwrap(), "spelunker");
    
    // no match
    words.pop(); // spelunker
    words.push(String::from("baz"));
    let exact = from_dictionary_option(&words, dict.clone());
    assert!(exact.is_none());
}


