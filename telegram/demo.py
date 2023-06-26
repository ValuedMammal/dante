import deepl
from config import config

# Create translator instance
DEEPL_AUTH = config["deepl"]
dl = deepl.Translator(auth_key=DEEPL_AUTH)

# Build request parameters - text + target_lang required
src = "IT"
trg = "EN-US"
less = deepl.Formality.PREFER_LESS
s = "Cosa sarebbe l'intelligenza senza il tocco umano?"

# Call `translate_text`
try:
    response = dl.translate_text(
        text=s,
        target_lang=trg,
        source_lang=src,
        formality=less,
    )
    response = response.text
except deepl.DeepLException as e:
    response = f"Deepl raised an exception: {e}"
finally:
    print(response)
    # > "What would intelligence be without the human touch?"
