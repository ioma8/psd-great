# Fix 05 — Engine Data: Separate j$/bo.* Parsers, Type Prefixes, Float Format, String Termination

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 6 bugs in `src/engine_data.rs`: (1) the `j$` (TextEngine blob) and `bo.*` (TypeEngine descriptor) formats are two distinct protocols with separate encoding rules — they must be parsed separately; (2) the `j$` format's `f`/`i`/`s`/`e`/`/` type-prefix encoding is entirely absent; (3) `serialize_float` strips the trailing `.` producing `"5"` where `"5.0"` is required, and omits leading-zero stripping for values in `(-1, 0)` and `(0, 1)`; (4) UTF-16 string termination stops at the first `0x29` byte instead of requiring whitespace or `>` after `)` ; (5) non-BOM `(...)` strings cause a hard error instead of falling back to ASCII; (6) `"NaN"`/`"undefined"` tokens are not handled.

**Architecture:** The current single `EngineDataParser` in `src/engine_data.rs` handles both formats incorrectly. We need to split into two parsers: `parse_type_engine_data` (bo.* format, used in layer descriptor values of type `tdta`) and `parse_text_engine_data` (j$ format, used for `TySh` text layer engine data blob). Each parser is a distinct function/impl in `src/engine_data.rs`. The serializer also splits into two symmetric functions.

**Tech Stack:** Rust, `cargo test`

**TS reference:** `photoshop/psd/src/psd/text-engine.ts` — read both `parseTypeEngineData`/`serializeTypeEngineData` (lines ~100–260) and `parseTextEngineData`/`serializeTextEngineData` (lines ~270–420).

---

### Task 1: Fix `serialize_float` — match TS `formatJFloat` exactly

**Bug:** `engine_data.rs` `serialize_float` strips trailing `.` (gives `"5"` not `"5.0"`) and doesn't strip leading zero for values between -1 and 1.

**Files:**
- Modify: `src/engine_data.rs` (`serialize_float` function)

- [ ] **Step 1: Write failing tests**

Add to `src/engine_data.rs` test module:

```rust
#[test]
fn serialize_float_matches_ts_format_j_float() {
    assert_eq!(serialize_float(5.0),   "5.0");
    assert_eq!(serialize_float(-5.0),  "-5.0");
    assert_eq!(serialize_float(0.5),   ".5");
    assert_eq!(serialize_float(-0.5),  "-.5");
    assert_eq!(serialize_float(1.23456), "1.23456");
    assert_eq!(serialize_float(1.2),   "1.2");
    assert_eq!(serialize_float(0.0),   "0.0");
}
```

- [ ] **Step 2: Run to verify failures**

```bash
cargo test serialize_float_matches_ts_format_j_float 2>&1
```
Expected: FAIL for `"5.0"`, `".5"`, `"-.5"` cases.

- [ ] **Step 3: Rewrite `serialize_float`**

```rust
fn serialize_float(value: f64) -> String {
    if value == value.round() {
        // Whole number: always emit ".0"
        return format!("{}.0", value as i64);
    }
    let mut text = format!("{:.5}", value);
    // Strip trailing zeros after decimal, but leave at least one digit after "."
    while text.ends_with('0') && !text.ends_with(".0") {
        text.pop();
    }
    // Strip leading "0" for values in (0, 1): "0.5" -> ".5"
    if value > 0.0 && value < 1.0 && text.starts_with("0.") {
        text = text[1..].to_string();
    }
    // Strip leading "-0" for values in (-1, 0): "-0.5" -> "-.5"
    if value < 0.0 && value > -1.0 && text.starts_with("-0.") {
        text = format!("-{}", &text[2..]);
    }
    text
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test serialize_float_matches_ts_format_j_float 2>&1
```
Expected: all assertions pass.

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 6: Commit**

```bash
git add src/engine_data.rs
git commit -m "fix: engine data serialize_float matches TS formatJFloat (preserves .0, strips leading zero)"
```

---

### Task 2: Fix UTF-16 string termination — require whitespace or `>` after `)`

**Bug:** The parser stops at the first `0x29` byte. TS only terminates at `)` if the next byte is `0x0A` (newline), `0x20` (space), or `0x3E` (`>`).

**Files:**
- Modify: `src/engine_data.rs` (UTF-16 string read loop)

- [ ] **Step 1: Write a failing test**

```rust
#[test]
fn utf16_string_with_0x29_low_byte_not_truncated() {
    // U+0029 is ')' but here we embed a character whose low byte is 0x29
    // For example 0x0029 = ')'; encode a string containing ')' char
    // The parser should not terminate early just because it sees 0x29
    // Build engine data bytes with: ( FE FF 0x00 0x41 0x00 0x29 0x00 0x42 ) \n
    // = UTF-16 string "A)B"
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"(");
    bytes.extend_from_slice(&[0xFE, 0xFF]); // BOM
    bytes.extend_from_slice(&[0x00, 0x41]); // 'A'
    bytes.extend_from_slice(&[0x00, 0x29]); // ')'  ← 0x29 low byte
    bytes.extend_from_slice(&[0x00, 0x42]); // 'B'
    bytes.extend_from_slice(b") \n");        // terminator with whitespace after
    // parse as a type-engine string — should yield "A)B" not "A"
    // ... use parse_type_engine_data or a lower-level helper
}
```

This test establishes the intent. Implement it once the parser supports the correct termination logic.

- [ ] **Step 2: Fix the UTF-16 string reader**

Find the `while ... != b')'` loop in `src/engine_data.rs`. Change to:

```rust
// UTF-16 string: terminate at ')' only when followed by whitespace, newline, or '>'
loop {
    if self.index >= self.data.len() {
        break;
    }
    // Peek: if current byte is b')' and next byte is b'\n', b' ', or b'>'
    if self.data[self.index] == b')' {
        let next = self.data.get(self.index + 1).copied().unwrap_or(0);
        if next == b'\n' || next == b' ' || next == b'>' {
            break;
        }
    }
    let high = self.get_text_byte()? as u16;
    let low  = self.get_text_byte()? as u16;
    result.push((high << 8) | low);
}
self.index += 1; // consume the ')'
```

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/engine_data.rs
git commit -m "fix: UTF-16 string in engine data only terminates at ) followed by whitespace or >"
```

---

### Task 3: Fix non-BOM `(...)` strings — fall back to ASCII

**Bug:** Missing BOM `FE FF` causes a hard error. TS falls back to `readEscapedAsciiString`.

**Files:**
- Modify: `src/engine_data.rs` (string open-paren handler)

- [ ] **Step 1: Write a failing test**

```rust
#[test]
fn non_bom_string_parsed_as_ascii() {
    // Engine data with a plain ASCII string (no BOM)
    // e.g. /key (hello) \n
    let input = b"/key (hello) \n";
    // parse_type_engine_data should return key="key", value="hello" (or similar)
    // not an error
}
```

- [ ] **Step 2: Fix the open-paren handler**

```rust
b'(' => {
    self.index += 1;
    if self.index + 1 < self.data.len()
        && self.data[self.index] == 0xFE
        && self.data[self.index + 1] == 0xFF
    {
        // UTF-16 string
        self.index += 2;
        self.read_utf16_string()
    } else {
        // ASCII string — read bytes until unescaped ')'
        self.read_escaped_ascii_string()
    }
}
```

Add `read_escaped_ascii_string`:
```rust
fn read_escaped_ascii_string(&mut self) -> Result<String> {
    let mut result = Vec::new();
    while self.index < self.data.len() {
        let b = self.data[self.index];
        if b == b')' {
            self.index += 1;
            break;
        }
        if b == b'\\' && self.index + 1 < self.data.len() {
            self.index += 1;
            result.push(self.data[self.index]);
        } else {
            result.push(b);
        }
        self.index += 1;
    }
    Ok(String::from_utf8_lossy(&result).to_string())
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/engine_data.rs
git commit -m "fix: engine data non-BOM (ASCII) strings fall back to escaped-ASCII reader instead of error"
```

---

### Task 4: Fix `NaN`/`undefined` tokens — map to float zero

**Bug:** `"NaN"` and `"undefined"` tokens cause a `skip(1)` which corrupts parse state. TS maps both to float zero in the j$ format.

**Files:**
- Modify: `src/engine_data.rs` (token parser)

- [ ] **Step 1: Find the token parser**

```bash
grep -n "NaN\|undefined\|null\|Unknown character" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/engine_data.rs | head -20
```

- [ ] **Step 2: Fix the token handler**

Find where `"null"` is handled and add:

```rust
// In the token classification:
if token == "NaN" || token == "undefined" {
    // j$ format: map to float zero
    return Ok(EngineValue::Number(0.0));
}
if token == "null" {
    return Ok(EngineValue::Null);
}
```

Make sure `"NaN"` and `"undefined"` are parsed as full tokens (advance `self.index` by the token length, not by 1).

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/engine_data.rs
git commit -m "fix: NaN/undefined tokens in engine data map to float 0 instead of corrupting parse"
```

---

### Task 5: Add j$ type-prefix encoding

**Bug:** The j$ format (TextEngine data blob) requires value type prefixes: `f` for floats, `i` for integers, `s` for strings, `e` for empty string, `/` for name tokens. The current parser discards these, causing wrong types on serialization.

**Files:**
- Modify: `src/engine_data.rs`

- [ ] **Step 1: Understand the current `EngineValue` enum**

```bash
grep -n "pub enum EngineValue\|EngineValue::" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/engine_data.rs | head -20
```

- [ ] **Step 2: Add `Integer` variant to `EngineValue`**

```rust
pub enum EngineValue {
    Number(f64),    // float (j$ prefix 'f')
    Integer(i64),   // integer (j$ prefix 'i')
    String(String), // string (j$ prefix 's')
    Name(String),   // name token starting with '/' (j$ prefix '/')
    Null,
    Array(Vec<EngineValue>),
    Object(Vec<(String, EngineValue)>),
}
```

- [ ] **Step 3: Fix j$ parser to detect and store type prefixes**

In `parse_text_engine_data` (the j$ parser), when reading a token:

```rust
fn parse_jvalue(token: &str) -> EngineValue {
    // NaN / undefined → float 0
    if token == "NaN" || token == "undefined" {
        return EngineValue::Number(0.0);
    }
    // Name token
    if token.starts_with('/') {
        return EngineValue::Name(token.to_string());
    }
    // Try float (has decimal point)
    if let Ok(f) = token.parse::<f64>() {
        if token.contains('.') {
            return EngineValue::Number(f);
        } else if let Ok(i) = token.parse::<i64>() {
            return EngineValue::Integer(i);
        }
    }
    EngineValue::String(token.to_string())
}
```

- [ ] **Step 4: Fix j$ serializer to emit type prefixes**

In `serialize_text_engine_data`:

```rust
fn serialize_jvalue(val: &EngineValue) -> String {
    match val {
        EngineValue::Number(f) => serialize_float(*f),
        EngineValue::Integer(i) => i.to_string(),
        EngineValue::Name(n) => n.clone(),
        EngineValue::String(s) if s.is_empty() => "()".to_string(),
        EngineValue::String(s) => format!("({} )", encode_utf16_string(s)),
        _ => String::new(),
    }
}
```

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 6: Commit**

```bash
git add src/engine_data.rs
git commit -m "feat: j$ engine data parser distinguishes float/integer/name/string value types"
```
