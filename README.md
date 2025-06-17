# rgpy

**rgpy** is a blazing-fast, Rust-powered regular expression search tool for Python.  
It brings the speed of [`ripgrep`](https://github.com/BurntSushi/ripgrep) and [`regex`](https://docs.rs/regex/) into your Python environment, ideal for scanning large log files and datasets.

---

## 🚀 Features

- Super-fast regex matching (powered by Rust)
- Supports both [`regex`](https://docs.rs/regex/) and [`pcre2`](https://docs.rs/pcre2/)
- Multithreaded line scanning (via Rayon)
- Easy-to-use Python API
- Supports:
  - Count-only mode (`count=True`)
  - Invert match (`invert_match=True`)
  - Recursive directory search (`search_dir()`)

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
    print(f"{line.path}:{line.line_number}: {line.text}")
```

🔢 Count-only mode
```python
from rgpy import search_file

count = search_file("error", "./logs/sample.log", count=True)
print("Total matches:", count)
```

🚫 Invert match (return lines that do NOT match)
```python
from rgpy import search_file

non_matches = search_file("error", "./logs/sample.log", invert_match=True)
for line in non_matches:
    print(f"{line.line_number}: {line.text}")
```

📂 Recursive search in a directory
```python
from rgpy import search_dir

results = search_dir("timeout", "./logs", ignore_case=True)
for line in results:
    print(f"{line.path}:{line.line_number}: {line.text}")
```

`multiprocessing` support
```python
from multiprocessing import Pool, cpu_count
from rgpy import search_file

LOG_FILE = "./logs/sample.log"

with open(LOG_FILE, "r", encoding="utf-8", errors="ignore") as f:
        lines = f.readlines()

chunk_size = len(lines) // cpu_count()
chunks = [lines[i:i + chunk_size] for i in range(0, len(lines), chunk_size)]

with Pool() as pool:
    counts = pool.map(search_file("error", LOG_FILE, invert_match=True), chunks)
```

---

## 🛠 Engine Options

- `"regex"`: Rust’s built-in, fast and safe regex engine (no backreferences or lookbehind)
- `"pcre2"`: Perl-compatible regex with full support for lookaround, backreference, etc. (slightly slower)

---

## ⚙️ Arguments

| Parameter     | Type    | Description                          |
|---------------|---------|--------------------------------------|
| `pattern`     | `str`   | Regex pattern to search              |
| `path`/`dir`        | `str`   | File path (`search_file`) or directory (`search_dir`)|
| `engine`      | `str`   | `"regex"` (default) or `"pcre2"`     |
| `ignore_case` | `bool`  | Case-insensitive matching (optional) |
| `count` | `bool`  | Return number of matches only (optional) |
| `invert_match` | `bool`  | Return lines that do not match the pattern (optional) |

---

## ⚡ Performance

rgpy uses **Rust + Rayon** for multithreaded file processing.  
This makes it significantly faster than Python’s built-in `re` module, especially for large files.
Without multiprocessing, `rgpy` is about 24% faster than `re`.

== Time Comparison (in seconds) ==  
rgpy:                  3.463s  
rgpy.compile:          3.466s  
rgpy.compile + mp:     2.441s  
re:                    4.541s  
re.compile:            4.228s  
re.compile + mp:       1.898s  


---

## 📜 License

MIT License

---

## 👤 Author

Developed by **[cpprhtn/Junwon Lee]**  
Inspired by [`ripgrep`](https://github.com/BurntSushi/ripgrep) and [`pyo3`](https://github.com/PyO3/pyo3)

---
