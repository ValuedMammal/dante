## Procedure for adding db rows
This inadvertently became a two-step process when the original csv was used for sqlite and another csv was made in a different format for postgres (there's probably a way to keep 1 csv compatible with both).

Now, we could simply add rows to pglatin.csv and use the COPY command with a WHERE clause on the row id to skip duplicates. Also, migrating the python implementation to postgres will obviate the need for two csv's. In the meantime, it may be easier for contributers to work with the sqlite-compatible format. 

- Therefore, to keep both csv's in sync, the procedure is outlined:
    - Add rows to latin.csv in normal format - comma delimiter, definition column is always double quoted, which allows ignoring commas in the defn
    - Create a temporary copy of latin.csv containing ONLY the diff, i.e. the newly added rows. Call it temp-latin.csv
    - Run `py trans-csv.py` (where py is an alias for python3). The script will look for temp-latin.csv in the same directory and output the file temp-pglatin.csv. In case of formatting issue, the script will fail and dump the offending row to the console for debugging. The newly formatted rows will need to be appended to the master pglatin.csv, e.g. `cat temp-pglatin.csv >> pglatin.csv`.
    - Using psql, execute `\copy latin from temp-pglatin.csv with delimiter ';';`
    - Both temp-latin.csv and temp-pglatin.csv can be safely removed.
    - Note: we can easily run trans-csv.py on the entire latin.csv (by modifying the in/out file names), generating a new pglatin.csv, then run psql copy with a where clause as stated above. Though in principle it saves cpu to only operate on the diff rather than repeatedly crunch existing rows, plus we avoid manipulating the master copies. 
- Things to be aware of: make sure id column numbers remain consecutive, with no duplicates, as this is the table primary key. in addition, the column for the english word is defined to be unique by the schema, so no duplicates allowed there either. See the schema.
- New csv rows should always be appended to the end, no need to alphabetize.
- If you need to remove a row of the csv from somewhere in the middle: take the current last row and put it in place of the row to be removed, leaving the id column unchanged (to preserve continuity). In this case, you'll need to execute a psql update statement on the affected row to set the new column data. In case of serious discrepancy, defer to the csv as canonical - we can always drop the db table and begin afresh.

### Can be added to latin.csv
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
travail - 'three stakes'  
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
