"""
Take a string from stdin (as csv) and either insert it in the database
or update the row if it exists. The approach is to assume we don't know the row id
and instead to query on the english word, so be sure to drop the id from the input
e.g.
    absent;absens;(adj) not present;absent;ausente;assente

Execution will fail if the input doesn't contain exactly 6 columns separated by ';'
Script returns the id of the affected row, which you can use to ensure the new row is 
properly reflected in the source csv

Usage, assuming the file temp.csv with a single line:
    cat temp.csv | py upsert.py
"""

import sys
from sqlalchemy import create_engine, text
from config import config

# Read line from stdin
s = sys.stdin.read()
l = s.split(';')

# Optionally get user-passed row id,
# and check vector len
row_id = None
if len(l) == 7:
    row_id = int(l.pop(0))
assert len(l) == 6

# note: id column not present
en = l[0]
la = l[1]
defn = l[2]
fr = l[3]
es = l[4]
it = l[5].rstrip()

# Connect db
url = config["db_url"]
ngin = create_engine(url)

action = ""
with ngin.connect() as db:
    # query for existing row
    latin_id = db.execute(
        text('SELECT id FROM latin WHERE en =:s'), {'s': en}
    ).scalar_one_or_none()
    
    # Insert
    if latin_id is None:
        # get best row id
        best_id = db.execute(
            text('SELECT MAX(id) from latin')
        ).scalar_one()

        # increment to next available id. hacky
        # pg id sequence not equal row count ?
        latin_id = best_id + 1
        row = (latin_id, en, la, defn, fr, es, it)
        db.execute(
            text('INSERT INTO latin (id, en, la, defn, fr, es, it) VALUES :tup'), {'tup': row}
        )
        action += "Inserted"
    
    # Update
    else:
        # primary key integrity check if user gave id
        if row_id != None and latin_id != row_id:
            print(f"WARN row id mismatch expected {row_id}, found {latin_id}")

        row = (en, la, defn, fr, es, it)
        db.execute(
            text('UPDATE latin SET (en, la, defn, fr, es, it) =:tup WHERE id =:i'), {'tup': row, 'i': latin_id}
        )
        action += "Updated"
    
    db.commit()

print(f"{action} row at {latin_id}")
