# Dante
A romance language companion

### Concept  
general UI: 
- make a service (telegram bot) for users to query an english word for its latin origin. 
- also serves as a translator (through DeepL)

impln details: 
- python/sqlite initial prototype. uses pyTelegramBotApi, py deepl
- rust/postgres uses teloxide, tokio, deepl crate (async)
- py/postgres work in progress.

share with language teachers/learners  

### Dictionary approach:
-------
- There's a unique constraint on the english word, as this is the primary search term.
- From Dante help: "modern analogs aim to capture lexical forms that most closely resemble their latin origin, but because the meaning of words drifts over time, they may no longer track semantically, or are seldom used in modern parlance." give an example ?  
- For latin roots: generally try to match the grammar of the english term. In cases of ambiguity (which is most cases), go with what feels familiar, or where better translations exist.
- Prefer verbs over adjectives. Prefer the noun when it can be clearly deduced by the etymology 
    - e.g. aquatic, from the latin: aqua, (n) water
- Latin verbs: prefer -are -ere -ire endings, i.e. the present active infinitive
- Ok to include two terms in one column of analogs [fr, es, it] separated by a space
- Adjectives: gender normally follows masculine or neuter -  for no reason other than laziness or consistency

### App Wishlist:
-------
- allow for auto-detecting source lang (currently required to include both source and target languages - it's often better to be explicit, but indeed requires more keystrokes)
- test web request to wordsense.eu in python
    - dynamically add db rows when a new eng/latin pair is discovered
- prompting interface, python LLM ?
- add more quirky comments, a la 'Carpe diem'
- add portuguese and romanian
- performance / DOS considerations
- CI, deployment
