# Alejandro

## proof of concept  
stuck between calling it Dante or Alejandro. either is fine  
general api: make a service for users to query an english word for latin origin  

should include a method for searching a dictionary of eng/latin words (look at DeepL API) and adding it to our db
if it's not there already

can be used as a template for other use case: auto respond to messages that meet some criteria e.g. spammy election texts  

game-like design: one main loop with various subroutines that only go "one level deep," so easy to reason about. /start /query /more /stop  
see [fCC py tutorial](https://www.freecodecamp.org/news/how-to-create-a-telegram-bot-using-python/)

share with language teachers/learners - davide, elissa, polymathy, nacho  

Note: before copying py script to ~/Library/Messages we'll need to substitute the correct sql query and add tables to chat.db schema

want a way to programatically add rows to the db. this may prove more complicated than simply getting translations due to "semantic drift"

still marinating on this
add verbiage to Dante help: "modern equiv aims to capture lexical forms that most closely resemble their latin origin"

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
    - started english/latin word list, csv ~375 words, rand source proofreadingservices.com
    - need to write defns, translations (fr, en, it), then write csv to table 'latin' in chat.db

### can be added to word list
annul
nuptial
nubile
decadent
