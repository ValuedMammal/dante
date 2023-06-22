use deepl::Lang;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use crate::util::query_greedy;

use super::{
    Dictionary,
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
    // syntax: /t src_lang trg_lang text
    let test_vec: Vec<(&str, (Lang, Lang, String))> = vec![
        (
            "/t en de good morning",
            (Lang::EN, Lang::DE, "good morning".to_string())
        ),
        (
            "/t es en-us dos lápices",
            (Lang::ES, Lang::EN_US, "dos lápices".to_string())
        ),
        (
            // test rejects all whitespace
            "/t en de   ",
            (Lang::EN, Lang::DE, "  ".to_string())
        ),
        
    ];
    
    let l = test_vec.len();
    for i in 0..l {
        let t = &test_vec[i];
        let result = parse_translation_candidate(t.0);

        // success path
        if i < 2 {
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), t.1);
        } else {
            // error path
            assert_eq!(result, Err(-1))
        }
    }
}

#[tokio::test]
async fn find_like_or_none() {
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
            // finds LIKE with '%foo%' syntax
            vec![String::from("foo")],
            Some(String::from("focus"))
        ),
        (
            // finds correct substring e.g. 'quire' -> inquire
            vec!["quire".to_string()],
            Some(String::from("inquire"))

        ),
        (
            // test None returned
            vec!["zzz".to_string()],
            None
        ),
        //
        //     //TODO: finds closest latin match e.g. 'ere' -> apparere
        //
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
    
#[test]
fn match_dictionary() {

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
