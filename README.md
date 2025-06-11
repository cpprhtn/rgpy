# rgpy

**rgpy** is a blazing-fast, Rust-powered regular expression search tool for Python.  
It brings the speed of [`ripgrep`](https://github.com/BurntSushi/ripgrep) and [`regex`](https://docs.rs/regex/) into your Python environment, ideal for scanning large log files and datasets.

---

## 🚀 Features

- Super-fast regex matching (powered by Rust)
- Supports both [`regex`](https://docs.rs/regex/) and [`pcre2`](https://docs.rs/pcre2/)
- Multithreaded line scanning (via Rayon)
- Easy-to-use Python API

---

## 📦 Installation

Install via pip using [maturin](https://github.com/PyO3/maturin):

```bash
pip install maturin
maturin develop  # for local development
```

> You can also install from PyPI once published:
> ```bash
> pip install rgpy
> ```

---

## 🧪 Example Usage

```python
from rgpy import search_file

results = search_file(
    pattern="error",
    path="./logs/sample.log",
    engine="regex",         # or "pcre2"
    ignore_case=True
)

for line in results:
    print(line)
```

---

## ⚙️ Arguments

| Parameter     | Type    | Description                          |
|---------------|---------|--------------------------------------|
| `pattern`     | `str`   | Regex pattern to search              |
| `path`        | `str`   | Path to the target file              |
| `engine`      | `str`   | `"regex"` (default) or `"pcre2"`     |
| `ignore_case` | `bool`  | Case-insensitive matching (optional) |

---

## 🛠 Engine Options

- `"regex"`: Rust’s built-in, fast and safe regex engine (no backreferences or lookbehind)
- `"pcre2"`: Perl-compatible regex with full support for lookaround, backreference, etc. (slightly slower)

---

## ⚡ Performance

rgpy uses **Rust + Rayon** for multithreaded file processing.  
This makes it significantly faster than Python’s built-in `re` module, especially for large files.

---

## 📜 License

MIT License

---

## 👤 Author

Developed by **[cpprhtn/Junwon Lee]**  
Inspired by [`ripgrep`](https://github.com/BurntSushi/ripgrep) and [`pyo3`](https://github.com/PyO3/pyo3)

---
