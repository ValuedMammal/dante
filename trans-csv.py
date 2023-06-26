"""
Create postgres-compatible csv for table `latin`, swapping comma delimiters for semicolon,
but keeping any ',' in the definition. Also we don't need the double quotes.
e.g. from:
    1,absent,absens,"(adj) foo; not present",absent,ausente,assente
to:
    1;absent;absens;(adj) foo, not present,absent;ausente;assente
"""

import re

# debug
# a = '1,absent,absens,"(adj) not present",absent,ausente,assente' # basic
# b = '1,absent,absens,"(adj) foo, not present",absent,ausente,assente' # comma in the defn
# c = '1,absent,absens,"(adj) foo; not present",absent,ausente,assente' # ';' in the defn
# d = '2,accelerate,accelerare,"(v) speed up, quicken",accélérer,acelerar,accelerare' # diacritics in last 3 col
# lines = [a, b, c, d]

pat = re.compile('^([\da-z;\s]+)"([()a-z;\s\-]+)"([a-z();\s\u00C0-\u017F]+)$')
# group 1: id,en,la # note \d for digit
# group 2: defn
# group 3: fr,es,it # note range of extra latin char

# Read in
with open('temp-latin.csv', 'r') as f:
    lines = f.readlines()

out: list[str] = []

for src in lines:
    # replace all comma
    src = src.replace(',', ';')

    match = re.match(pat, src)
    if match is None:
        print(src)
        exit(1)

    # replace semicolon in defn
    defn = match.group(2)
    defn = defn.replace(';', ',')

    # make string
    pre = match.group(1)
    post = match.group(3)
    
    s = pre + defn + post
    out.append(s)

# Write out
with open('temp-pglatin.csv', 'w') as f:
    f.writelines(out)
