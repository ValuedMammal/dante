# Dante
A romance language companion

### Concept  
stuck between calling it Dante or Alejandro. either is fine  
general api: make a service for users to query an english word for its latin origin  

looks like DeepL doesn't have latin dict

add a prompting interface + LLM  

first 350 words is minimally viable. obviously there is potentially many more.

related use case: read iMessage db and auto respond to messages that meet some criteria e.g. spammy election texts. implement a lightning invoice fetcher

game-like design: one main loop with various subroutines that only go "one level deep," so easy to reason about. see [fCC py tutorial](https://www.freecodecamp.org/news/how-to-create-a-telegram-bot-using-python/). see also: finite state machine

possible implementations: python/sqlite, py/postgres, rust/postgres

share with language teachers/learners - davide, elissa, polymathy, nacho  

want a way to programatically add rows to the db. this may prove more complicated than simply getting translations due to "semantic drift"

### Dictionary approach:
- Unique constraint on the english word, as this is the primary search term
- From Dante help: "modern equivalents aim to capture lexical forms that most closely resemble their latin origin, but because the meaning of words drifts over time, may not track semantically, or are seldom used"  
- For latin roots: generally try to match the grammar of the english term. in cases of ambiguity, go with what feels familiar, or where better translations exist
    - prefer verbs over adjectives. prefer noun when it can be clearly deduced by the etymology 
    - e.g. aquatic, from the latin: aqua, (n) water
    - ok to include two terms in a column (fr, es, it) separated by a space
- Adjectives: gender normally follows masculine or neuter -  for no reason other than laziness or consistency
- Wishlist
    - visual indicator of success/failure of last command
    - greedily search for the closest db match in case we don't have exact
    - add more quirky comments, a la 'Carpe diem'
    - performance / DOS considerations
    - prompting interface, state machine


### Dev log:
-------
### 15Jun 2023  
- worked on unit tests

### 14Jun 2023  
-  rewrote trans-csv using regex
    - fixed typos in latin.csv
    - re-ran both scripts and diff'ed the outputs. identical result
    - tested \copy to postgres, syntax: `\copy latin from pglatin.csv with delimiter ';'` ( `;` to commit)

### 13Jun 2023  
- py script for populating rows in postgres csv (~375 rows)  
- rust: lots of debugging. Alejandro can /help, /query, /trans, /usage

### 12Jun 2023  
- impl deepl translate first attempt github.com/mgruner/deepl-api-rs/
    - 'tokio-runtime-worker' panicked at 'Cannot drop a runtime in a context where blocking is not allowed.
    - i guess because this crate is not async
    - or we need a synchronous tg bot handler
- finally got a prototype up for Alejandro 
    - teloxide + sqlx Postgres
    - github.com/Avimitin/deepl-rs
- py: consider option to allow db query by column
- consider suggest (you might be interested in...) if first db query returns no rows 

### 11Jun 2023  
- for sqlite i used double quotes in definition column so we could include commas, however postgres doesn't like this when we try to `\copy` table from csv. instead, we use semicolon ; as a delimiter to allow use of comma inside a definition.
- work on deepl-rust api

### 10Jun 2023  
- rust: I had trouble connecting existing bot through teloxide, but creating a new bot worked fine
- alejandro rust bot. tested requst/response
- tested sql query (remember start/stop postgres)
- try impl deepl api

### 9Jun 2023  
- hooked up DeepL api and implemented the translate command in telegram, so Dante can now translate using deepl
- nice to have words alphabetical, but shouldn't be necessary

### 8Jun 2023  
- latinlexicon (Numen) uses dicts: elementary (charlton t lewis) and latin dict (lewis + short)
- latin defn: prefer the literal root meaning to show etymology
- for latin verbs: prefer -re over -us
- any newly added db rows must also be included in the in-memory word list. otherwise, we'll have to run a db query on every word
in a text. (streamline this step?)

### 7Jun 2023  
- in the definition, does the grammar part (n / adj / v) refer only to the latin counterpart or the english word?
- do the translations always correspond in gramm structure to the english word?
- example: aquiline (adj), latin aquila (n) - an eagle
    - does it create more ambiguity to include both (adj/n) in the response?
- modern forms seek to follow linguistic similarity, not necessarily semantic
- find out why some latin dict entries have many forms e.g. associo, associare, associavi, associatus

### 4Jun 2023  
- wrote some in rust - send repeat text
- in python, impld lookup latin words  
    - reponds with translations
    - started english/latin word list, csv ~375 words, took it from a random source proofreadingservices.com because it was already in an html table
    - need to write defns, translations (fr, en, it), then write csv to table 'latin' in chat.db

### Can be added to dict
annul  
nuptial  
nubile  
decadent  
assiduous  
dubious  
reputation repute  
grace  
disaster - 'ill-starred event'  
none/null  
travail - 'three stakes'  
laudable  
enthusiasm  
gentile  
liberty  
clairty  
magnanimous  
majority  
sanctuary  
