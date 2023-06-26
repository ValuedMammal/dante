# Dante
A romance language companion

## Concept  
general UI: 
- make a service (telegram bot) for users to query an english word for its latin origin. 
- also serves as a translator (via DeepL)
- share with language teachers/learners  

implementation:  
This is not an ai. It's currently just a database and web requester
- python/sqlite prototype. uses pyTelegramBotApi, py deepl
- rust/postgres uses teloxide, tokio, deepl crate (async)
- py/postgres work in progress.

Contributions welcome. See [contribution guidelines](/contributing.md).

## App Wishlist:
-------
- allow optionally passing source lang (currently required to include both source and target languages - it's often better to be explicit, but indeed requires more keystrokes). This can probably be done by testing an additional regex pattern
- test web request to wordsense.eu in python
    - dynamically add db rows when a new eng/latin pair is discovered
- consider using webhooks
- prompting interface, python LLM ?
- add more quirky comments, a la 'Carpe diem'
- add portuguese and romanian
- performance considerations
- CI, deployment
