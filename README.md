# Dante
A romance language companion

## Concept  
general UI: 
- make a service (telegram bot) for users to query an english word for its latin origin. 
- also serves as a translator (via DeepL)
- share with language teachers/learners  

implementation:  
- This is not an ai. It's currently just a database and web requester.  
- The [current implementation](./rs) uses crates [teloxide](https://docs.rs/teloxide/) and [deepl](https://docs.rs/deepl/) 

Contributions welcome. See [contributing guidelines](./contributing.md).

## App Wishlist:
-------
- consider using webhooks instead of long polling
- prompting interface, python LLM ?
- add more quirky comments, a la 'Carpe diem'
- add portuguese and romanian
- CI, deployment
