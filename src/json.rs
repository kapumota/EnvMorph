//! Tipo JSON pequeño con formateador y parser. Los números del modelo se conservan como enteros.

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn obj(fields: Vec<(&str, Json)>) -> Json {
        Json::Obj(
            fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn str<S: Into<String>>(s: S) -> Json {
        Json::Str(s.into())
    }

    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(v) => v.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    fn write(&self, out: &mut String, indent: usize) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Num(n) => out.push_str(&n.to_string()),
            Json::Str(s) => write_str(out, s),
            Json::Arr(a) => {
                if a.is_empty() {
                    out.push_str("[]");
                    return;
                }
                let scalar = a.iter().all(|v| !matches!(v, Json::Arr(_) | Json::Obj(_)));
                if scalar {
                    out.push('[');
                    for (i, v) in a.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        v.write(out, indent);
                    }
                    out.push(']');
                    return;
                }
                out.push_str("[\n");
                for (i, v) in a.iter().enumerate() {
                    out.push_str(&"  ".repeat(indent + 1));
                    v.write(out, indent + 1);
                    if i + 1 < a.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str(&"  ".repeat(indent));
                out.push(']');
            }
            Json::Obj(o) => {
                if o.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push_str("{\n");
                for (i, (k, v)) in o.iter().enumerate() {
                    out.push_str(&"  ".repeat(indent + 1));
                    write_str(out, k);
                    out.push_str(": ");
                    v.write(out, indent + 1);
                    if i + 1 < o.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str(&"  ".repeat(indent));
                out.push('}');
            }
        }
    }
}

fn write_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn parse(src: &str) -> Result<Json, String> {
    let mut p = Parser {
        s: src.as_bytes(),
        i: 0,
    };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("datos adicionales a partir del byte {}", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\n' | b'\r' | b'\t') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn expect(&mut self, c: u8) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("se esperaba '{}' en el byte {}", c as char, self.i))
        }
    }

    fn lit(&mut self, word: &str, v: Json) -> Result<Json, String> {
        if self.s[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(format!("literal inválido en el byte {}", self.i))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            None => Err("fin inesperado de la entrada".into()),
            Some(b'n') => self.lit("null", Json::Null),
            Some(b't') => self.lit("true", Json::Bool(true)),
            Some(b'f') => self.lit("false", Json::Bool(false)),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b'[') => {
                self.i += 1;
                let mut a = Vec::new();
                self.ws();
                if self.peek() == Some(b']') {
                    self.i += 1;
                    return Ok(Json::Arr(a));
                }
                loop {
                    self.ws();
                    a.push(self.value()?);
                    self.ws();
                    match self.peek() {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Json::Arr(a));
                        }
                        _ => return Err(format!("se esperaba ',' o ']' en el byte {}", self.i)),
                    }
                }
            }
            Some(b'{') => {
                self.i += 1;
                let mut o = Vec::new();
                self.ws();
                if self.peek() == Some(b'}') {
                    self.i += 1;
                    return Ok(Json::Obj(o));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.ws();
                    self.expect(b':')?;
                    self.ws();
                    let v = self.value()?;
                    o.push((k, v));
                    self.ws();
                    match self.peek() {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Json::Obj(o));
                        }
                        _ => return Err(format!("se esperaba ',' o '}}' en el byte {}", self.i)),
                    }
                }
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => {
                let start = self.i;
                self.i += 1;
                while self.i < self.s.len()
                    && matches!(
                        self.s[self.i],
                        b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-'
                    )
                {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.s[start..self.i]).map_err(|e| e.to_string())?;
                if let Ok(n) = t.parse::<i64>() {
                    Ok(Json::Num(n))
                } else {
                    t.parse::<f64>()
                        .map(|f| Json::Num(f as i64))
                        .map_err(|e| e.to_string())
                }
            }
            Some(c) => Err(format!(
                "carácter inesperado '{}' en el byte {}",
                c as char, self.i
            )),
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        if self.i + 4 > self.s.len() {
            return Err("escape \\u truncado".into());
        }
        let t = std::str::from_utf8(&self.s[self.i..self.i + 4]).map_err(|e| e.to_string())?;
        self.i += 4;
        u32::from_str_radix(t, 16).map_err(|e| e.to_string())
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = self.peek().ok_or("cadena sin terminar")?;
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = self.peek().ok_or("escape inválido")?;
                    self.i += 1;
                    match e {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let mut cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp)
                                && self.s[self.i..].starts_with(b"\\u")
                            {
                                self.i += 2;
                                let lo = self.hex4()?;
                                if (0xDC00..0xE000).contains(&lo) {
                                    cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                }
                            }
                            let ch = char::from_u32(cp).unwrap_or('\u{fffd}');
                            let mut tmp = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut tmp).as_bytes());
                        }
                        _ => return Err("escape desconocido".into()),
                    }
                }
                c => out.push(c),
            }
        }
        String::from_utf8(out).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let v = Json::obj(vec![
            ("a", Json::Num(-5)),
            ("b", Json::str("x\"y\n\u{1}é")),
            (
                "c",
                Json::Arr(vec![Json::Null, Json::Bool(true), Json::Num(7)]),
            ),
            ("d", Json::Arr(vec![Json::obj(vec![("k", Json::str("v"))])])),
            ("e", Json::Obj(vec![])),
        ]);
        assert_eq!(parse(&v.pretty()).unwrap(), v);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse("{\"a\": }").is_err());
        assert!(parse("[1, 2").is_err());
        assert!(parse("{} x").is_err());
    }

    #[test]
    fn unicode_escape() {
        assert_eq!(
            parse("\"\\u00e9\\ud83d\\ude00\"").unwrap(),
            Json::str("é😀")
        );
    }
}
