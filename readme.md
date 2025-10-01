# godot-pack

`godot-pack` is a Rust-based alternative to Python’s [`struct`](https://docs.python.org/3/library/struct.html) module, designed for **Godot Engine**.
It allows packing and unpacking binary data directly from **GDScript** using familiar format strings.

This is useful when working with binary protocols, file parsing, or network messages in Godot without relying on Python.

---

## ✨ Features
- Format string syntax inspired by Python’s `struct`.
- Support for endianness (`<`, `>`, `@`, `!`).
- Packing / unpacking of primitive data types (`bool`, integers, floats, strings).
- Padding support (`x`).
- Optimized and written in Rust, exposed to Godot via GDExtension.

---

## 📦 Supported Format Characters

| Character | Meaning | Size (bytes) |
|-----------|---------|--------------|
| `@` / `=` | Native endianness | – |
| `<`       | Little-endian | – |
| `>`       | Big-endian | – |
| `!`       | Network (big-endian) | – |
| `...s`    | Preceded by `...` digits as length, a null terminated string          | ... or at least one byte |
| `...x`       | Preceded by `...` digits as length, padding space                     | ... or at least one byte |
| `?`       | Boolean | 1 |
| `c`       | Character (byte) | 1 |
| `b`       | Signed 8-bit integer | 1 |
| `B`       | Unsigned 8-bit integer | 1 |
| `h`       | Signed 16-bit integer | 2 |
| `H`       | Unsigned 16-bit integer | 2 |
| `i`       | Signed 32-bit integer | 4 |
| `I`       | Unsigned 32-bit integer | 4 |
| `l`       | Signed 32-bit integer (long) | 4 |
| `L`       | Unsigned 32-bit integer (long) | 4 |
| `q`       | Signed 64-bit integer | 8 |
| `Q`       | Unsigned 64-bit integer | 8 |
| `f`       | 32-bit floating point | 4 |
| `d`       | 64-bit floating point | 8 |
| *other*   | Invalid pattern | – |

---

## 🚀 Example (GDScript)

```gdscript

# Pack two integers and a float
var pack = Pack.from("<iif32s")
var packed = pack.pack([42, -1, 3.14,"Hello world!"])

# Unpack them back
var unpacked = pack.unpack(packed)
```

## 🎉 Installation
Copy `godot-pack.gdextension` to your Godot's project folder, replace the paths to a relative ones to where the repository is put.
Compile with `cargo build --release`, have fun.

### 🫣 P.S.
Sorry for the laziest ai generated readme, it's been fixed by me if that would help.
