<img width=144 src="https://github.com/seanghay/sosap/assets/15277233/25c2ae30-4dd6-4350-a387-c30353cb2a98">

Python binding for [Phonetisaurus](https://github.com/AdolfVonKleist/Phonetisaurus) using Cython.

### Install

```shell
# pypi
pip install sosap

# GitHub
pip install git+https://github.com/seanghay/sosap.git
```

### Phoneticize


```python
from sosap import Model

model = Model("g2p.fst")
model.phoneticize("hello")
```

### Enable Sampling

```python
from sosap import Model

model = Model("model")
results = model.phoneticize_sampling("hello", nbest=4)
# => [['h', 'ɛɛ', 'l', 'oo'], ['h', 'ee', '.', 'l', 'oo'], ['h', 'ɛɛ', 'l', '.', 'l', 'ɔɔ'], ['h', 'ɛɛ', '.', 'l', 'oo']]

results = model.phoneticize_sampling("hello", nbest=4, beam=1000, threshold=99.0, pmass=99.0)
# => [['h', 'ɛɛ', 'l', 'oo'], ['h', 'ee', '.', 'l', 'oo'], ['h', 'ɛɛ', 'l', '.', 'l', 'ɔɔ'], ['h', 'ɛɛ', '.', 'l', 'oo']]
```

---

### License

`MIT`

