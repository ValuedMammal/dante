import telebot
import re
import deepl
from util import get_db, is_authorized, is_a_word
import logging as log

from config import config

DB_PATH = "chat.db"
BOT_AUTH = config["tg"]
L_AUTH = config["deepl"]
bot = telebot.TeleBot(BOT_AUTH)
log.basicConfig(filename="dante.log", encoding="utf-8", level=log.INFO)
log.info("Starting tg bot...")

# Load dictionary from db
# e.g. latins = {'a': ["absent",], ...}
latins = {
    'a': [],
    'b': [],
    'c': [],
    'd': [],
    'e': [],
    'f': [],
    'g': [],
    'h': [],
    'i': [],
    'j': [],
    'k': [],
    'l': [],
    'm': [],
    'n': [],
    'o': [],
    'p': [],
    'q': [],
    'r': [],
    's': [],
    't': [],
    'u': [],
    'v': [],
    'w': [],
    'x': [],
    'y': [],
    'z': [],
}
conn = get_db(DB_PATH)
db = conn.cursor()
rows = db.execute('select en from latin').fetchall()
for row in rows:
    word: str = row[0]
    first: str = word[0]
    l: list = latins[first]
    l.append(word)
conn.close()
log.info("Loaded dictionary")

# Create instance of translator
dl = deepl.Translator(L_AUTH)


@bot.message_handler(commands=['start', 'help'])
def start(msg):
    """Show telegram bot help message"""
    if not is_authorized(msg.chat.id): return

    help = "I am Dante, the romantic. I'll tell you whether an English word has roots in the Latin language./q Where applicable, I include modern translations for other languages (currently \"FR\", \"ES\", & \"IT\"). I can also translate words to and from various languages./t\n\nSee the commands list for usage and syntax.\n\ntips: A query result contains a grammatical part (noun, adj, verb) that refers to the latin root, and not necessarily the english word. However, I will do my best to ensure the 'modern equivalents' correspond grammatically to the english term.\n\nKeep in mind, the 'modern equivalents' aim to capture lexical forms that most closely resemble their latin origin, but since the meaning of words has drifted over time, they may no longer track semantically or are only rarely used. For more dynamic translations, the translate command should come in handy.\n\nBy the way, we're adding words to the dictionary all the time - let us know if you believe a common English/Latin pair is missing.\n\nOk enough preamble,\nCarpe Diem!"
    bot.send_message(msg.chat.id, help)
    

@bot.message_handler(commands=['q'])
def query(msg):
    """Performs a database query by english word for the latin equivalent(s)"""

    global latins
    if not is_authorized(msg.chat.id): return

    s: str = msg.text
    words = s.split()

    # Check input
    if len(words) < 2:
        bot.reply_to(msg, "Usage: /q <word>\nExample: /q foo")
        return
    _cmd = words.pop(0) # the cmd str
    
    # Though not explicit, the handler accepts a list of words, but will leave iteration on first match
    en = None
    for w in words:
        w = w.lower()
        if not is_a_word(w): continue
        first = w[0]
        if w in latins[first]:
            en = w
            break

    if en == None:
        reply = "None"
    else: 
        # Get translations for this word
        conn = get_db(DB_PATH)
        db = conn.cursor()
        row = db.execute("select * from latin where en = ?", (en,)).fetchone()
        la = row[2]
        defn = row[3]
        fr = row[4]
        es = row[5]
        it = row[6]
        reply = f"Here's what I found for {en},\nfrom the latin: {la}, {defn}\nmodern equivalents:\nfr {fr},\nes {es},\nit {it}"
        conn.close()
    
    bot.send_message(msg.chat.id, reply)

@bot.message_handler(commands=['t'])
def translate(msg):
    """Calls the DeepL API for translation."""

    global dl
    if not is_authorized(msg.chat.id): return
    
    text: str = msg.text
    words = text.split()

    # Check input
    if len(words) < 3:
        bot.reply_to(msg, "Usage: /t <target_lang> <phrase>\nExample: /t ES good morning")
        return
    _cmd = words.pop(0)
    trg = words.pop(0)
    
    # Remake query string
    # note: can we extract it with regexp?
    q = ''
    for w in words:
        q += f"{w} "
    
    # Call api with query string and target lang
    log.info(f"Sending request with: {q}")
    try:
        response = dl.translate_text(
            text=q,
            target_lang=trg, 
            formality=deepl.Formality.PREFER_LESS
        )
        reply = response.text
    except deepl.DeepLException as e:
        log.debug(f"Translator returned exception: {e} on query: {q}")
        reply = "translate error, refer to logs"
    finally:
        bot.send_message(msg.chat.id, reply)

@bot.message_handler(commands=['p'])
def prompt(msg):
    """Prompts the bot for an intelligent response"""

    default = "unimplemented"

    # lorem ipsum easter egg
    if re.search('/p lorem ipsum', msg.text) != None:
        reply = "You must be flattering me."
    else:
        reply = default

    # TODO: implement LLM
    
    bot.reply_to(msg, reply)
    

@bot.message_handler(commands=['id'])
def get_chat_id(msg):
    """Retrieves telegram Chat id for this chat"""
    bot.reply_to(msg, f"{msg.chat.id}")
    

bot.infinity_polling()

## Test auth token
# @bot.message_handler(commands=['me'])
# def get_me(msg):
#     reply = bot.get_me()
#     bot.reply_to(msg, reply.id)

## Gnu/Linux suggester
# @bot.message_handler(regexp='linux')
# def gnu_linux(message):
#     bot.reply_to(message, "You mean GNU/Linux?")

## Test echo message
# @bot.message_handler(func=lambda message: True)
# def echo(message):
# 	bot.reply_to(message, message.text)