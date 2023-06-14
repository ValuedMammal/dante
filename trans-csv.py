## Create postgres-compatible csv for table `latin`,
## swapping comma delimiters for semicolon
##      but retain ',' in defn column
##
## e.g. from:
##      1,absent,absens,"(adj) foo; not present",absent,ausente,assente
## to   1;absent;absens;(adj) foo, not present,absent;ausente;assente
##

import re

# Approach
# replace comma with semicolon
# remove double quotes
# in the defn string, replace any semicolon with comma
# concat capture groups into string and collect

# tests
# a = '1,absent,absens,"(adj) not present",absent,ausente,assente' # basic
# b = '1,absent,absens,"(adj) foo, not present",absent,ausente,assente' # 2 element defn
# c = '1,absent,absens,"(adj) foo; not present",absent,ausente,assente' # ';' in the defn
# d = '2,accelerate,accelerare,"(v) speed up, quicken",accélérer,acelerar,accelerare' # diacritics in last 3 elems
# lines = [a, b, c, d]

p = re.compile('^([\da-z;\s]+)("[()a-z;\s\-]+")([a-z();\s\u00C0-\u017F]+)$')
# group 1: id,en,la # note \d for digit
# group 2: defn
# group 3: fr,es,it # note range of extra latin char

# Read in
with open('latin.csv', 'r') as f:
    lines = f.readlines()

out: list[str] = []

for src in lines:
    # replace all comma
    src = src.replace(',', ';')

    match = re.match(p, src)
    if match is None:
        print(src)
        exit(1)

    # strip quotes
    # note: can simply capture inside the quotes in regex?
    defn: str = match.group(2)
    defn = defn.strip('"')

    # replace semicolon in defn
    defn = defn.replace(';', ',')

    # make string
    pre: str = match.group(1)
    post: str = match.group(3)
    
    s: str = pre + defn + post
    out.append(s)

# Write out
with open('pglatin.csv', 'w') as f:
    f.writelines(out)
