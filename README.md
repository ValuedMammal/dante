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

Contributions welcome. See [contributing guidelines](/contributing.md).

## App Wishlist:
-------
- test web request to wordsense.eu in python
- dynamically add db rows when a new eng/latin pair is discovered
- consider using webhooks
- prompting interface, python LLM ?
- add more quirky comments, a la 'Carpe diem'
- add portuguese and romanian
- performance considerations
- CI, deployment
