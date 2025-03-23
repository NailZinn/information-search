import os

from string import punctuation
from nltk.corpus import stopwords
from pymystem3 import Mystem

# uncomment for first launch
# nltk.download('stopwords')
# nltk.download('punkt_tab')

russian_stopwords = stopwords.words('russian')

non_letters = punctuation + '«' + '»' + '…' + '°' + ' '

stemmer = Mystem()

def validToken(token: str) -> bool:
  return (
    token != ' ' and
    token != '' and
    token != '\n' and
    token not in russian_stopwords and
    all(map(lambda x: x not in non_letters, token))
  )

if not os.path.exists('tokens'):
  os.mkdir('tokens')

with open('../task-1/index.txt', 'r', encoding='utf-8') as index_file:
  index = index_file.readlines()

document_names = [entry.split(' - ')[0] for entry in index]

for document_name in document_names:
  with open(f'../task-1/pages/{document_name}.txt', 'r', encoding='utf-8') as document:
    print(f'processing {document.name}')
    document_content = document.readline().lower()

  tokens = stemmer.lemmatize(document_content)
  tokens = [token + '\n' for token in tokens if validToken(token)]

  with open(f'tokens/{document_name}.txt', 'w+', encoding='utf-8') as document:
    document.writelines(tokens)