from g2p import Model

model = Model("g2p.fst")
r = model.phoneticize("hello")
print(r)
