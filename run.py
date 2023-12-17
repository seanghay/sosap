from g2p import Model
from time import time

model = Model("g2p.fst")

while True:
  r = model.phoneticize("hello")
  print(r, time())