use super::{
    util::{is_valid_query, parse_translatable, query_greedy, try_from_dictionary},
    Dictionary,
};
use deepl::Lang;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;

const DB_PATH: &str = env!("DATABASE_URL");

#[test]
fn valid_query() {
    let bad = "/q";
    assert!(!is_valid_query(bad));

    let raw = "/q foo bar absent";
    assert!(is_valid_query(raw));
}

#[test]
fn valid_translatable() {
    // syntax: /t src_lang trg_lang text
    let test_vec: Vec<(&str, (Lang, Lang, String))> = vec![
        (
            "/t en de good morning",
            (Lang::EN, Lang::DE, "good morning".to_string()),
        ),
        (
            "/t it en-us Cos'è l'intelligenza?",
            (Lang::IT, Lang::EN_US, "Cos'è l'intelligenza?".to_string()),
        ),
    ];

    // successfully parsed
    for t in test_vec {
        let result = parse_translatable(t.0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), t.1);
    }

    // rejects empty text
    let s = "/t en de   ";
    let result = parse_translatable(s);
    assert!(result.is_err());
    //TODO consider deriving `PartialEq` for `Error`
    //assert_eq!(result.unwrap_err(), Error::Usage);
}

#[tokio::test]
async fn find_like_or_none() {
    let db_path = DB_PATH;
    let db: PgPool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_path)
        .await
        .unwrap();

    let test_vec: Vec<(Vec<String>, Option<String>)> = vec![
        (
            // walks the vector,
            // and strips trailing char
            vec![String::from("zzz"), String::from("spelunkerzzz")],
            Some(String::from("spelunker")),
        ),
        (
            // finds LIKE with '%foo%' syntax
            vec![String::from("foo")],
            Some(String::from("focus")),
        ),
        (
            // finds correct substring e.g. 'quire' -> inquire
            vec!["quire".to_string()],
            Some(String::from("inquire")),
        ),
        (
            // finds closest latin match e.g. '-arium' -> aviary
            vec!["viarium".to_string()],
            Some(String::from("aviary")),
        ),
        (
            // test None returned
            vec!["zzz".to_string()],
            None,
        ),
    ];

    for case in test_vec {
        let words = case.0;
        let expect = case.1;

        let mut en: Option<String> = None; // an english word

        let row = query_greedy(words, db.clone()).await;

        if let Some(latin) = row {
            en = Some(latin.en)
        }
        assert_eq!(en, expect);
    }
}

#[test]
fn dict_new() {
    //struct Dictionary { map: HashMap<char, Vec<String>> }
    let d = Dictionary::new();

    // all keys present, with empty vectors
    for i in 0_u8..26 {
        let key = (b'a' + i) as char;
        let val = d.map.get(&key);
        assert!(val.is_some());
        assert!(val.unwrap().is_empty())
    }
}

#[test]
fn dict_match() {
    // test iterate dictionary match
    // e.g. [foo, bar, absent] yields 'absent'

    let mut d = Dictionary::new();
    d.map.get_mut(&'s').unwrap().push(String::from("spelunker"));

    let dict = Arc::new(d);

    let mut words = vec![
        "foo".to_string(),
        "bar".to_string(),
        "spelunker".to_string(),
    ];

    // match returned
    let exact = try_from_dictionary(&words, dict.clone());
    assert!(exact.is_some());
    assert_eq!(exact.unwrap(), "spelunker");

    // no match
    words.pop(); // spelunker
    words.push(String::from("baz"));
    let exact = try_from_dictionary(&words, dict.clone());
    assert!(exact.is_none());
}
