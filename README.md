## 🗣️ sosap / សូរសព្ទ

Python binding for Phonetisaurus

### Install

```shell
# pypi
pip install sosap

# GitHub
pip install git+https://github.com/seanghay/sosap.git
```

### Usage

```python
from sosap import Model

model = Model("g2p.fst")

model.phoneticize("hello")
```


### License

MIT
