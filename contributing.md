### Contributing
We encourage anyone to suggest fixes and improvements to the app, particularly when it comes to adding entries to our dictionary of Latin words. There is an [ongoing list](#can-be-added-to-latin-csv) at the bottom of the page of potential words to be added. Some common action steps are:
- If you're familiar with github
    - you can submit a pull request modifying the main latin.csv here on github, or submit other code changes
    - you can create an issue in this repo describing any problems you noticed with the telegram bot.
- Share thoughts and criticism in the Lengua Americana telegram group chat (link?)

The beauty of language is that by studying the history of a word, you very often find a slew of related words that are equally fascinating. Granted there are more possible words out there than we can possibly keep track of, and there are many dictionaries and resources on the web already, but we think the value Dante brings is to consolidate useful information into a multi-lingual experience that facilitates learning and social interaction.

The only stipulation to expanding our existing dictionary collection is that the word should exist in English today and have a clear etymology that stems from Latin. Ideally there also exists a range of decendants and analogs in modern romance languages, but if no analogs are found in French for example, it's ok to simply put 'unknown' in the column. Naturally, we'd prefer that new rows to the csv have all columns completed (none left blank) before pushing updates. See [latin.csv](/latin.csv) for examples.

### Dictionary approach:
-------
- There's a unique constraint on the english word, as this is the primary search term. That means the same word can't appear in more than one row, but again this only applies to the english word column.
- From Dante help: "modern analogs aim to capture lexical forms that most closely resemble their latin origin, but because the meaning of words drifts over time, they may no longer track semantically, or are seldom used in modern parlance." give an example ?  
- For latin roots: generally try to match the grammar of the english term. In cases of ambiguity (which is most cases), go with what feels familiar, or where better translations exist.
- Prefer verbs over adjectives. Prefer the noun when it can be clearly deduced by the etymology 
    - e.g. aquatic, from the latin: aqua, (n) water
- Latin verbs: prefer -are -ere -ire endings, i.e. the present active infinitive
- Ok to include two terms in one column of analogs [fr, es, it] separated by a space
- Adjectives: gender normally follows masculine or neuter -  for no reason other than laziness or consistency

### For maintainers - procedure for adding db rows
-------
*Update: it's now easier to insert/update a single database row using upsert.py. details on how to use it are in the source file.*

At a high level, it's desirable that the source csv and the database remain in sync. This can be accomplished in many ways, and some redundancy in our process is warranted in order to prevent loss. When in doubt, assume the csv is the source of truth and should be handled with care. Specifically, we ended up with two csv's because the original was used for sqlite and another csv was made in a different format for postgres (there's probably a way to keep 1 csv compatible with both).

As far as the database is concerned, we could simply add rows to pglatin.csv and use the COPY command with a WHERE clause on the row id to skip duplicates. Perhaps also migrating the python implementation to postgres will obviate the need for two csv's. In the meantime, it may be easier for contributers to work with the sqlite-compatible format (plus github makes it look nice). 

- Therefore, to keep both csv's in sync, the procedure is outlined:
    - Add rows to latin.csv in normal format - comma delimiter, definition column is always double quoted, which allows ignoring commas in the defn
    - Create a temporary copy of latin.csv containing ONLY the diff, i.e. the newly added rows. Call it temp-latin.csv
    - Run `py trans-csv.py` (where py is an alias for python3). The script will look for temp-latin.csv in the same directory and output the file temp-pglatin.csv. In case of formatting issue, the script will fail and dump the offending row to the console for debugging. The newly formatted rows will need to be appended to the master pglatin.csv, e.g. `cat temp-pglatin.csv >> pglatin.csv`.
    - Using psql, execute `\copy latin from temp-pglatin.csv with delimiter ';';`
    - At this point, latin.csv, pglatin.csv, and the database should match. Both temporary files (temp-latin.csv, temp-pglatin.csv) can be safely removed.
    - Note: we can easily run trans-csv.py on the entire latin.csv (by modifying the in/out file names), generating a new pglatin.csv, then run psql copy with a where clause as stated above. Though in principle it saves cpu to only operate on the diff rather than repeatedly crunch existing rows, plus we avoid manipulating the master copies. 
- Things to be aware of: make sure id column numbers remain consistent, with no duplicates, as this is the table primary key. in addition, the column for the english word is defined to be unique by the schema, so no duplicates allowed there either. See the schema.
- New csv rows should always be appended to the end, no need to alphabetize.
- If you need to remove a row of the csv from somewhere in the middle: take the current last row and put it in place of the row to be removed, leaving the id column unchanged to preserve continuity. After that, you'll need to emit psql statements on the affected rows (updating one and deleting another). In case of serious discrepancy, defer to the csv as canonical - we can always drop the db table and begin afresh.

### Can be added to latin csv
annul  
nuptial  
nubile  
decadent  
assiduous  
dubious  
reputation repute  
grace  
inferior  
disaster - 'ill-starred event'  
none/null  
laudable  
enthusiasm  
gentile  
liberty  
clairty  
magnanimous  
majority  
sanctuary  
obfuscate  
vulgar  
ovum  
parsimony  
penultimate  
famine  
genial/gentile  
impudent  
irascible  
please/pleasure piacere  
peace  
affable  
price pretium  
provenance  
persuade  
acquiesce  
coy  
reason ratio  
royal  
rule regula  
nephew nepotism  
rude  
erudite  
volition  
serum  
signal  
conduct  
tavern  
tangible  
terrarium  
ointment  
valor  
valiant  
vent  
case/casette/chasse  
vowel  
letter  
