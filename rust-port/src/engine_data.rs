//! Engine data parsing and serialization
//!
//! Provides parsing and serialization of Adobe engine data format used for text layers.

use crate::error::{PsdError, Result};
use std::collections::HashMap;

/// Engine data value types
#[derive(Debug, Clone, PartialEq)]
pub enum EngineValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<EngineValue>),
    Object(HashMap<String, EngineValue>),
}

impl EngineValue {
    pub fn as_object(&self) -> Option<&HashMap<String, EngineValue>> {
        if let EngineValue::Object(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&Vec<EngineValue>> {
        if let EngineValue::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let EngineValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let EngineValue::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let EngineValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

/// Parse engine data from bytes
pub fn parse_engine_data(data: &[u8]) -> Result<EngineValue> {
    let mut parser = EngineDataParser::new(data);
    parser.parse()
}

struct EngineDataParser<'a> {
    data: &'a [u8],
    index: usize,
    stack: Vec<StackItem>,
}

enum StackItem {
    Value(EngineValue),
    Property(String),
}

impl<'a> EngineDataParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            index: 0,
            stack: Vec::new(),
        }
    }

    fn is_whitespace(ch: u8) -> bool {
        ch == b' ' || ch == b'\n' || ch == b'\r' || ch == b'\t'
    }

    fn is_number_char(ch: u8) -> bool {
        ch.is_ascii_digit() || ch == b'.' || ch == b'-'
    }

    fn skip_whitespace(&mut self) {
        while self.index < self.data.len() && Self::is_whitespace(self.data[self.index]) {
            self.index += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.index < self.data.len() {
            Some(self.data[self.index])
        } else {
            None
        }
    }

    fn peek2(&self) -> Option<(u8, u8)> {
        if self.index + 1 < self.data.len() {
            Some((self.data[self.index], self.data[self.index + 1]))
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<u8> {
        if self.index < self.data.len() {
            let ch = self.data[self.index];
            self.index += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn get_text_byte(&mut self) -> Result<u8> {
        let byte = self.data.get(self.index)
            .ok_or(PsdError::InvalidEngineData("Unexpected end of data".to_string()))?;
        self.index += 1;

        if *byte == b'\\' {
            let next = self.data.get(self.index)
                .ok_or(PsdError::InvalidEngineData("Unexpected end after escape".to_string()))?;
            self.index += 1;
            Ok(*next)
        } else {
            Ok(*byte)
        }
    }

    fn read_text(&mut self) -> Result<String> {
        if self.peek() == Some(b')') {
            self.advance();
            return Ok(String::new());
        }

        // Check for UTF-16 BOM
        if self.index + 1 < self.data.len() 
            && self.data[self.index] == 0xFE 
            && self.data[self.index + 1] == 0xFF {
            self.index += 2;
        } else {
            return Err(PsdError::InvalidEngineData("Missing UTF-16 BOM".to_string()));
        }

        let mut result = Vec::new();

        while self.index < self.data.len() && self.data[self.index] != b')' {
            let high = self.get_text_byte()? as u16;
            let low = self.get_text_byte()? as u16;
            let ch = (high << 8) | low;
            result.push(ch);
        }

        if self.peek() == Some(b')') {
            self.advance();
        }

        String::from_utf16(&result)
            .map_err(|e| PsdError::InvalidEngineData(format!("Invalid UTF-16: {}", e)))
    }

    fn push_container(&mut self, value: EngineValue) {
        self.stack.push(StackItem::Value(value));
    }

    fn push_value(&mut self, value: EngineValue) -> Result<()> {
        if self.stack.is_empty() {
            self.stack.push(StackItem::Value(value));
            return Ok(());
        }

        // Check if top is a property name
        if let Some(StackItem::Property(key)) = self.stack.last() {
            let key = key.clone();
            self.stack.pop();

            if let Some(StackItem::Value(EngineValue::Object(ref mut map))) = self.stack.last_mut() {
                map.insert(key, value);
            } else {
                return Err(PsdError::InvalidEngineData("Expected object on stack".to_string()));
            }
        } else if let Some(StackItem::Value(EngineValue::Array(ref mut arr))) = self.stack.last_mut() {
            arr.push(value);
        } else {
            return Err(PsdError::InvalidEngineData("Invalid stack state".to_string()));
        }

        Ok(())
    }

    fn push_property(&mut self, name: String) -> Result<()> {
        if self.stack.is_empty() {
            self.push_container(EngineValue::Object(HashMap::new()));
        }

        if let Some(StackItem::Property(ref key)) = self.stack.last() {
            if name == "nil" {
                self.push_value(EngineValue::Null)?;
            } else {
                let property_value = format!("/{}", name);
                self.push_value(EngineValue::String(property_value))?;
            }
        } else if let Some(StackItem::Value(EngineValue::Object(_))) = self.stack.last() {
            self.stack.push(StackItem::Property(name));
        } else {
            return Err(PsdError::InvalidEngineData("Invalid property context".to_string()));
        }

        Ok(())
    }

    fn pop(&mut self) -> Result<()> {
        if self.stack.is_empty() {
            return Err(PsdError::InvalidEngineData("Stack underflow".to_string()));
        }
        self.stack.pop();
        Ok(())
    }

    fn parse(&mut self) -> Result<EngineValue> {
        self.skip_whitespace();

        // Trim trailing zeros
        let mut data_len = self.data.len();
        while data_len > 0 && self.data[data_len - 1] == 0 {
            data_len -= 1;
        }

        while self.index < data_len {
            self.skip_whitespace();

            if self.index >= data_len {
                break;
            }

            let ch = self.peek().unwrap();

            if ch == b'<' && self.peek2() == Some((b'<', b'<')) {
                // Object start: <<
                self.index += 2;
                self.push_container(EngineValue::Object(HashMap::new()));
            } else if ch == b'>' && self.peek2() == Some((b'>', b'>')) {
                // Object end: >>
                self.index += 2;
                self.pop()?;
            } else if ch == b'/' {
                // Property name: /name
                self.advance();
                let start = self.index;

                while self.index < self.data.len() && !Self::is_whitespace(self.data[self.index]) {
                    self.index += 1;
                }

                let name = String::from_utf8_lossy(&self.data[start..self.index]).to_string();
                self.push_property(name)?;
            } else if ch == b'(' {
                // String: (text)
                self.advance();
                let text = self.read_text()?;
                self.push_value(EngineValue::String(text))?;
            } else if ch == b'[' {
                // Array: [...]
                self.advance();
                self.push_container(EngineValue::Array(Vec::new()));
            } else if ch == b']' {
                // Array end
                self.advance();
                self.pop()?;
            } else if self.index + 3 < self.data.len()
                && &self.data[self.index..self.index + 4] == b"null"
            {
                // null
                self.index += 4;
                self.push_value(EngineValue::Null)?;
            } else if self.index + 3 < self.data.len()
                && &self.data[self.index..self.index + 4] == b"true"
            {
                // true
                self.index += 4;
                self.push_value(EngineValue::Boolean(true))?;
            } else if self.index + 4 < self.data.len()
                && &self.data[self.index..self.index + 5] == b"false"
            {
                // false
                self.index += 5;
                self.push_value(EngineValue::Boolean(false))?;
            } else if Self::is_number_char(ch) {
                // Number
                let start = self.index;

                while self.index < self.data.len() && Self::is_number_char(self.data[self.index]) {
                    self.index += 1;
                }

                let num_str = String::from_utf8_lossy(&self.data[start..self.index]);
                let num = num_str.parse::<f64>()
                    .map_err(|e| PsdError::InvalidEngineData(format!("Invalid number: {}", e)))?;
                self.push_value(EngineValue::Number(num))?;
            } else {
                // Unknown character - skip
                self.index += 1;
            }
        }

        // Return the root value
        if self.stack.is_empty() {
            Ok(EngineValue::Null)
        } else if self.stack.len() == 1 {
            if let Some(StackItem::Value(value)) = self.stack.pop() {
                Ok(value)
            } else {
                Err(PsdError::InvalidEngineData("Expected value on stack".to_string()))
            }
        } else {
            Err(PsdError::InvalidEngineData("Incomplete parsing".to_string()))
        }
    }
}

/// Float keys that should be serialized with decimal points
const FLOAT_KEYS: &[&str] = &[
    "Axis", "XY", "Zone", "WordSpacing", "FirstLineIndent", "GlyphSpacing",
    "StartIndent", "EndIndent", "SpaceBefore", "SpaceAfter", "LetterSpacing",
    "Values", "GridSize", "GridLeading", "PointBase", "BoxBounds",
    "TransformPoint0", "TransformPoint1", "TransformPoint2", "FontSize",
    "Leading", "HorizontalScale", "VerticalScale", "BaselineShift", "Tsume",
    "OutlineWidth", "AutoLeading",
];

/// Int array keys
const INT_ARRAYS: &[&str] = &["RunLengthArray"];

/// Serialize engine data to bytes
pub fn serialize_engine_data(data: &EngineValue, condensed: bool) -> Result<Vec<u8>> {
    let mut serializer = EngineDataSerializer::new(condensed);
    serializer.serialize(data)?;
    Ok(serializer.buffer)
}

struct EngineDataSerializer {
    buffer: Vec<u8>,
    condensed: bool,
    indent: usize,
}

impl EngineDataSerializer {
    fn new(condensed: bool) -> Self {
        Self {
            buffer: Vec::with_capacity(1024),
            condensed,
            indent: 0,
        }
    }

    fn write(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    fn write_str(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
    }

    fn write_indent(&mut self) {
        if self.condensed {
            self.write_str(" ");
        } else {
            for _ in 0..self.indent {
                self.write_str("\t");
            }
        }
    }

    fn serialize_number(&self, value: f64, key: Option<&str>) -> String {
        let is_float = key.map_or(false, |k| FLOAT_KEYS.contains(&k)) || value.fract() != 0.0;

        if is_float {
            self.serialize_float(value)
        } else {
            (value as i64).to_string()
        }
    }

    fn serialize_float(&self, value: f64) -> String {
        format!("{:.5}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }

    fn get_keys(map: &HashMap<String, EngineValue>) -> Vec<String> {
        let mut keys: Vec<String> = map.keys().cloned().collect();
        
        // Move "99" and "98" to front if present
        if let Some(pos) = keys.iter().position(|k| k == "99") {
            let key = keys.remove(pos);
            keys.insert(0, key);
        }
        if let Some(pos) = keys.iter().position(|k| k == "98") {
            let key = keys.remove(pos);
            keys.insert(0, key);
        }
        
        keys
    }

    fn write_string_byte(&mut self, byte: u8) {
        if byte == b'(' || byte == b')' || byte == b'\\' {
            self.write(b'\\');
        }
        self.write(byte);
    }

    fn write_value(&mut self, value: &EngineValue, key: Option<&str>, in_property: bool) -> Result<()> {
        match value {
            EngineValue::Null => {
                if !in_property {
                    self.write_indent();
                } else {
                    self.write_str(" ");
                }
                self.write_str(if self.condensed { "/nil" } else { "null" });
            }
            EngineValue::Number(n) => {
                if !in_property {
                    self.write_indent();
                } else {
                    self.write_str(" ");
                }
                self.write_str(&self.serialize_number(*n, key));
            }
            EngineValue::Boolean(b) => {
                if !in_property {
                    self.write_indent();
                } else {
                    self.write_str(" ");
                }
                self.write_str(if *b { "true" } else { "false" });
            }
            EngineValue::String(s) => {
                if !in_property {
                    self.write_indent();
                } else {
                    self.write_str(" ");
                }

                if (key == Some("99") || key == Some("98")) && s.starts_with('/') {
                    self.write_str(s);
                } else {
                    self.write_str("(");
                    self.write(0xFE);
                    self.write(0xFF);

                    for ch in s.encode_utf16() {
                        self.write_string_byte((ch >> 8) as u8);
                        self.write_string_byte((ch & 0xFF) as u8);
                    }

                    self.write_str(")");
                }
            }
            EngineValue::Array(arr) => {
                if !in_property {
                    self.write_indent();
                } else {
                    self.write_str(" ");
                }

                // Check if all elements are numbers
                let all_numbers = arr.iter().all(|v| matches!(v, EngineValue::Number(_)));

                if all_numbers {
                    self.write_str("[");

                    let is_int_array = key.map_or(false, |k| INT_ARRAYS.contains(&k));

                    for val in arr {
                        if let EngineValue::Number(n) = val {
                            self.write_str(" ");
                            if is_int_array {
                                self.write_str(&self.serialize_number(*n, None));
                            } else {
                                self.write_str(&self.serialize_float(*n));
                            }
                        }
                    }

                    self.write_str(" ]");
                } else {
                    self.write_str("[");
                    if !self.condensed {
                        self.write_str("\n");
                    }

                    for val in arr {
                        self.write_value(val, key, false)?;
                        if !self.condensed {
                            self.write_str("\n");
                        }
                    }

                    self.write_indent();
                    self.write_str("]");
                }
            }
            EngineValue::Object(map) => {
                if in_property && !self.condensed {
                    self.write_str("\n");
                }

                self.write_indent();
                self.write_str("<<");

                if !self.condensed {
                    self.write_str("\n");
                }

                self.indent += 1;

                for key in Self::get_keys(map) {
                    if let Some(val) = map.get(&key) {
                        self.write_indent();
                        self.write_str(&format!("/{}", key));
                        self.write_value(val, Some(&key), true)?;
                        if !self.condensed {
                            self.write_str("\n");
                        }
                    }
                }

                self.indent -= 1;
                self.write_indent();
                self.write_str(">>");
            }
        }

        Ok(())
    }

    fn serialize(&mut self, data: &EngineValue) -> Result<()> {
        if self.condensed {
            if let EngineValue::Object(map) = data {
                for key in Self::get_keys(map) {
                    if let Some(val) = map.get(&key) {
                        self.write_indent();
                        self.write_str(&format!("/{}", key));
                        self.write_value(val, Some(&key), true)?;
                    }
                }
            }
        } else {
            self.write_str("\n\n");
            self.write_value(data, None, false)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let data = b"<< /key 123 >>";
        let result = parse_engine_data(data).unwrap();
        
        if let EngineValue::Object(map) = result {
            assert_eq!(map.len(), 1);
            assert_eq!(map.get("key").and_then(|v| v.as_number()), Some(123.0));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let data = b"[ 1 2 3 ]";
        let result = parse_engine_data(data).unwrap();
        
        if let EngineValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_serialize_number() {
        let value = EngineValue::Number(123.0);
        let result = serialize_engine_data(&value, true).unwrap();
        let text = String::from_utf8_lossy(&result);
        assert!(text.contains("123"));
    }
}
