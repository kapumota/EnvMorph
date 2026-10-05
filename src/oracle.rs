use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquivalenceStatus {
    Identical,
    Equivalent,
    Different,
    Missing,
    Error,
}

impl EquivalenceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Identical => "identical",
            Self::Equivalent => "equivalent",
            Self::Different => "different",
            Self::Missing => "missing",
            Self::Error => "error",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Identical | Self::Equivalent => 0,
            Self::Different => 1,
            Self::Missing | Self::Error => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalenceResult {
    status: EquivalenceStatus,
    reason: String,
    metadata: BTreeMap<String, String>,
}

impl EquivalenceResult {
    fn new(status: EquivalenceStatus, reason: impl Into<String>) -> Self {
        Self {
            status,
            reason: reason.into(),
            metadata: BTreeMap::new(),
        }
    }

    fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn status(&self) -> EquivalenceStatus {
        self.status
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRef {
    path: PathBuf,
    verified_sha256: Option<String>,
}

impl ArtifactRef {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            verified_sha256: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_verified_sha256(
        path: impl Into<PathBuf>,
        verified_sha256: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            verified_sha256: Some(verified_sha256.into()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn verified_sha256(&self) -> Option<&str> {
        self.verified_sha256.as_deref()
    }
}

pub trait EquivalenceOracle {
    fn name(&self) -> &'static str;
    fn compare(&self, left: &ArtifactRef, right: &ArtifactRef) -> EquivalenceResult;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ByteOracle;

impl EquivalenceOracle for ByteOracle {
    fn name(&self) -> &'static str {
        "byte"
    }

    fn compare(&self, left: &ArtifactRef, right: &ArtifactRef) -> EquivalenceResult {
        if let Some(result) = validate_paths(left, right) {
            return result;
        }

        if let (Some(left_hash), Some(right_hash)) =
            (left.verified_sha256(), right.verified_sha256())
        {
            if left_hash != right_hash {
                return EquivalenceResult::new(
                    EquivalenceStatus::Different,
                    "los SHA-256 verificados difieren, por lo que los bytes difieren",
                )
                .with_metadata("comparison", "byte_exact")
                .with_metadata("fast_path", "verified_sha256_mismatch");
            }
        }

        let (left_bytes, right_bytes) = match read_pair(left, right) {
            Ok(bytes) => bytes,
            Err(result) => return result,
        };

        if left_bytes == right_bytes {
            EquivalenceResult::new(
                EquivalenceStatus::Identical,
                "contenido byte a byte idéntico",
            )
            .with_metadata("comparison", "byte_exact")
        } else {
            EquivalenceResult::new(EquivalenceStatus::Different, "diferencia byte a byte")
                .with_metadata("comparison", "byte_exact")
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TextOptions {
    pub normalize_line_endings: bool,
    pub trim_trailing_whitespace: bool,
    pub ignore_final_newline: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TextOracle {
    options: TextOptions,
}

impl TextOracle {
    pub fn new(options: TextOptions) -> Self {
        Self { options }
    }
}

impl EquivalenceOracle for TextOracle {
    fn name(&self) -> &'static str {
        "text"
    }

    fn compare(&self, left: &ArtifactRef, right: &ArtifactRef) -> EquivalenceResult {
        if let Some(result) = validate_paths(left, right) {
            return result;
        }

        let (left_bytes, right_bytes) = match read_pair(left, right) {
            Ok(bytes) => bytes,
            Err(result) => return result,
        };

        if left_bytes == right_bytes {
            return text_metadata(
                EquivalenceResult::new(
                    EquivalenceStatus::Identical,
                    "contenido textual idéntico byte a byte",
                ),
                self.options,
            );
        }

        let left_text = match std::str::from_utf8(&left_bytes) {
            Ok(text) => text,
            Err(error) => {
                return text_metadata(
                    EquivalenceResult::new(
                        EquivalenceStatus::Error,
                        format!("el artefacto izquierdo no es UTF-8 válido: {error}"),
                    ),
                    self.options,
                );
            }
        };

        let right_text = match std::str::from_utf8(&right_bytes) {
            Ok(text) => text,
            Err(error) => {
                return text_metadata(
                    EquivalenceResult::new(
                        EquivalenceStatus::Error,
                        format!("el artefacto derecho no es UTF-8 válido: {error}"),
                    ),
                    self.options,
                );
            }
        };

        let left_normalized = normalize_text(left_text, self.options);
        let right_normalized = normalize_text(right_text, self.options);

        if left_normalized == right_normalized {
            text_metadata(
                EquivalenceResult::new(
                    EquivalenceStatus::Equivalent,
                    "texto equivalente bajo las normalizaciones configuradas",
                ),
                self.options,
            )
        } else {
            text_metadata(
                EquivalenceResult::new(
                    EquivalenceStatus::Different,
                    "el contenido textual difiere bajo la configuración seleccionada",
                ),
                self.options,
            )
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct JsonOracle;

impl EquivalenceOracle for JsonOracle {
    fn name(&self) -> &'static str {
        "json"
    }

    fn compare(&self, left: &ArtifactRef, right: &ArtifactRef) -> EquivalenceResult {
        if let Some(result) = validate_paths(left, right) {
            return result;
        }

        let (left_bytes, right_bytes) = match read_pair(left, right) {
            Ok(bytes) => bytes,
            Err(result) => return result,
        };

        if left_bytes == right_bytes {
            return EquivalenceResult::new(
                EquivalenceStatus::Identical,
                "JSON idéntico byte a byte",
            )
            .with_metadata("object_key_order", "ignored_for_equivalence")
            .with_metadata("array_order", "significant")
            .with_metadata("fields", "all_significant");
        }

        let left_text = match std::str::from_utf8(&left_bytes) {
            Ok(text) => text,
            Err(error) => {
                return EquivalenceResult::new(
                    EquivalenceStatus::Error,
                    format!("el JSON izquierdo no es UTF-8 válido: {error}"),
                );
            }
        };

        let right_text = match std::str::from_utf8(&right_bytes) {
            Ok(text) => text,
            Err(error) => {
                return EquivalenceResult::new(
                    EquivalenceStatus::Error,
                    format!("el JSON derecho no es UTF-8 válido: {error}"),
                );
            }
        };

        let left_value = match JsonParser::parse(left_text) {
            Ok(value) => value,
            Err(error) => {
                return EquivalenceResult::new(
                    EquivalenceStatus::Error,
                    format!("error al analizar JSON izquierdo: {error}"),
                );
            }
        };

        let right_value = match JsonParser::parse(right_text) {
            Ok(value) => value,
            Err(error) => {
                return EquivalenceResult::new(
                    EquivalenceStatus::Error,
                    format!("error al analizar JSON derecho: {error}"),
                );
            }
        };

        let result = if left_value == right_value {
            EquivalenceResult::new(EquivalenceStatus::Equivalent, "estructura JSON equivalente")
        } else {
            EquivalenceResult::new(EquivalenceStatus::Different, "la estructura JSON difiere")
        };

        result
            .with_metadata("object_key_order", "ignored_for_equivalence")
            .with_metadata("array_order", "significant")
            .with_metadata("fields", "all_significant")
            .with_metadata("number_policy", "lexical_after_json_parse")
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CsvOracle;

impl EquivalenceOracle for CsvOracle {
    fn name(&self) -> &'static str {
        "csv"
    }

    fn compare(&self, left: &ArtifactRef, right: &ArtifactRef) -> EquivalenceResult {
        if let Some(result) = validate_paths(left, right) {
            return result;
        }

        let (left_bytes, right_bytes) = match read_pair(left, right) {
            Ok(bytes) => bytes,
            Err(result) => return result,
        };

        if left_bytes == right_bytes {
            return EquivalenceResult::new(
                EquivalenceStatus::Identical,
                "CSV idéntico byte a byte",
            )
            .with_metadata("row_order", "significant")
            .with_metadata("delimiter", ",");
        }

        let left_text = match std::str::from_utf8(&left_bytes) {
            Ok(text) => text,
            Err(error) => {
                return EquivalenceResult::new(
                    EquivalenceStatus::Error,
                    format!("el CSV izquierdo no es UTF-8 válido: {error}"),
                );
            }
        };

        let right_text = match std::str::from_utf8(&right_bytes) {
            Ok(text) => text,
            Err(error) => {
                return EquivalenceResult::new(
                    EquivalenceStatus::Error,
                    format!("el CSV derecho no es UTF-8 válido: {error}"),
                );
            }
        };

        let left_rows = match parse_csv(left_text) {
            Ok(rows) => rows,
            Err(error) => {
                return EquivalenceResult::new(
                    EquivalenceStatus::Error,
                    format!("error al analizar CSV izquierdo: {error}"),
                );
            }
        };

        let right_rows = match parse_csv(right_text) {
            Ok(rows) => rows,
            Err(error) => {
                return EquivalenceResult::new(
                    EquivalenceStatus::Error,
                    format!("error al analizar CSV derecho: {error}"),
                );
            }
        };

        let base = if left_rows.len() != right_rows.len() {
            EquivalenceResult::new(
                EquivalenceStatus::Different,
                format!(
                    "la cantidad de filas CSV difiere: {} frente a {}",
                    left_rows.len(),
                    right_rows.len()
                ),
            )
        } else if let Some((row_index, left_width, right_width)) =
            first_width_difference(&left_rows, &right_rows)
        {
            EquivalenceResult::new(
                EquivalenceStatus::Different,
                format!(
                    "la cantidad de columnas difiere en la fila {}: {} frente a {}",
                    row_index + 1,
                    left_width,
                    right_width
                ),
            )
        } else if left_rows == right_rows {
            EquivalenceResult::new(
                EquivalenceStatus::Equivalent,
                "estructura tabular CSV equivalente",
            )
        } else {
            EquivalenceResult::new(
                EquivalenceStatus::Different,
                "el contenido de las celdas CSV difiere",
            )
        };

        base.with_metadata("row_order", "significant")
            .with_metadata("delimiter", ",")
            .with_metadata("left_rows", left_rows.len().to_string())
            .with_metadata("right_rows", right_rows.len().to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ComparisonReport {
    oracle: String,
    left_artifact: String,
    right_artifact: String,
    result: EquivalenceResult,
}

impl ComparisonReport {
    pub fn new(
        oracle: impl Into<String>,
        left: &ArtifactRef,
        right: &ArtifactRef,
        result: EquivalenceResult,
    ) -> Self {
        Self {
            oracle: oracle.into(),
            left_artifact: left.path().display().to_string(),
            right_artifact: right.path().display().to_string(),
            result,
        }
    }

    pub fn result(&self) -> &EquivalenceResult {
        &self.result
    }

    pub fn render_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Oráculo: {}\n", self.oracle));
        out.push_str(&format!("Artefacto izquierdo: {}\n", self.left_artifact));
        out.push_str(&format!("Artefacto derecho: {}\n", self.right_artifact));
        out.push_str(&format!("Resultado: {}\n", self.result.status().as_str()));
        out.push_str(&format!("Razón: {}\n", self.result.reason()));

        if !self.result.metadata().is_empty() {
            out.push_str("Metadatos:\n");
            for (key, value) in self.result.metadata() {
                out.push_str(&format!("  {}={}\n", key, value));
            }
        }

        out
    }

    pub fn render_json(&self) -> String {
        let metadata = self
            .result
            .metadata()
            .iter()
            .map(|(key, value)| format!("\"{}\":\"{}\"", json_escape(key), json_escape(value)))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"oracle\":\"{}\",\"left_artifact\":\"{}\",\"right_artifact\":\"{}\",\"result\":\"{}\",\"reason\":\"{}\",\"metadata\":{{{}}}}}\n",
            json_escape(&self.oracle),
            json_escape(&self.left_artifact),
            json_escape(&self.right_artifact),
            self.result.status().as_str(),
            json_escape(self.result.reason()),
            metadata
        )
    }
}

fn validate_paths(left: &ArtifactRef, right: &ArtifactRef) -> Option<EquivalenceResult> {
    let left_exists = left.path().exists();
    let right_exists = right.path().exists();

    if !left_exists || !right_exists {
        let reason = match (left_exists, right_exists) {
            (false, false) => "faltan ambos artefactos",
            (false, true) => "falta el artefacto izquierdo",
            (true, false) => "falta el artefacto derecho",
            (true, true) => unreachable!(),
        };

        return Some(EquivalenceResult::new(EquivalenceStatus::Missing, reason));
    }

    if !left.path().is_file() {
        return Some(EquivalenceResult::new(
            EquivalenceStatus::Error,
            "el artefacto izquierdo no es un archivo regular",
        ));
    }

    if !right.path().is_file() {
        return Some(EquivalenceResult::new(
            EquivalenceStatus::Error,
            "el artefacto derecho no es un archivo regular",
        ));
    }

    None
}

fn read_pair(
    left: &ArtifactRef,
    right: &ArtifactRef,
) -> Result<(Vec<u8>, Vec<u8>), EquivalenceResult> {
    let left_bytes = fs::read(left.path()).map_err(|error| {
        EquivalenceResult::new(
            EquivalenceStatus::Error,
            format!(
                "no se pudo leer el artefacto izquierdo '{}': {}",
                left.path().display(),
                error
            ),
        )
    })?;

    let right_bytes = fs::read(right.path()).map_err(|error| {
        EquivalenceResult::new(
            EquivalenceStatus::Error,
            format!(
                "no se pudo leer el artefacto derecho '{}': {}",
                right.path().display(),
                error
            ),
        )
    })?;

    Ok((left_bytes, right_bytes))
}

fn text_metadata(result: EquivalenceResult, options: TextOptions) -> EquivalenceResult {
    result
        .with_metadata(
            "normalize_line_endings",
            options.normalize_line_endings.to_string(),
        )
        .with_metadata(
            "trim_trailing_whitespace",
            options.trim_trailing_whitespace.to_string(),
        )
        .with_metadata(
            "ignore_final_newline",
            options.ignore_final_newline.to_string(),
        )
}

fn normalize_text(text: &str, options: TextOptions) -> String {
    let mut normalized = if options.normalize_line_endings {
        text.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        text.to_string()
    };

    if options.trim_trailing_whitespace {
        let mut trimmed = String::with_capacity(normalized.len());

        for segment in normalized.split_inclusive('\n') {
            let has_newline = segment.ends_with('\n');
            let content = if has_newline {
                &segment[..segment.len() - 1]
            } else {
                segment
            };

            trimmed.push_str(content.trim_end_matches(|ch| ch == ' ' || ch == '\t'));
            if has_newline {
                trimmed.push('\n');
            }
        }

        normalized = trimmed;
    }

    if options.ignore_final_newline && normalized.ends_with('\n') {
        normalized.pop();
    }

    normalized
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

struct JsonParser {
    chars: Vec<char>,
    pos: usize,
}

impl JsonParser {
    fn parse(input: &str) -> Result<JsonValue, String> {
        let mut parser = Self {
            chars: input.chars().collect(),
            pos: 0,
        };

        parser.skip_whitespace();
        let value = parser.parse_value()?;
        parser.skip_whitespace();

        if parser.pos != parser.chars.len() {
            return Err(format!(
                "contenido adicional a partir de la posición {}",
                parser.pos + 1
            ));
        }

        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();

        match self.peek() {
            Some('n') => {
                self.expect_literal("null")?;
                Ok(JsonValue::Null)
            }
            Some('t') => {
                self.expect_literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some('f') => {
                self.expect_literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some('-' | '0'..='9') => self.parse_number().map(JsonValue::Number),
            Some(ch) => Err(format!(
                "token inesperado '{}' en la posición {}",
                ch,
                self.pos + 1
            )),
            None => Err("JSON vacío o incompleto".into()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect_char('[')?;
        self.skip_whitespace();

        let mut values = Vec::new();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(JsonValue::Array(values));
        }

        loop {
            values.push(self.parse_value()?);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                Some(ch) => {
                    return Err(format!(
                        "se esperaba ',' o ']' y se encontró '{}' en la posición {}",
                        ch,
                        self.pos + 1
                    ));
                }
                None => return Err("arreglo JSON sin cierre".into()),
            }
        }

        Ok(JsonValue::Array(values))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect_char('{')?;
        self.skip_whitespace();

        let mut values = BTreeMap::new();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(JsonValue::Object(values));
        }

        loop {
            if self.peek() != Some('"') {
                return Err(format!(
                    "se esperaba una clave JSON en la posición {}",
                    self.pos + 1
                ));
            }

            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_char(':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;

            if values.insert(key.clone(), value).is_some() {
                return Err(format!("clave JSON duplicada: {}", key));
            }

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.pos += 1;
                    break;
                }
                Some(ch) => {
                    return Err(format!(
                        "se esperaba ',' o '}}' y se encontró '{}' en la posición {}",
                        ch,
                        self.pos + 1
                    ));
                }
                None => return Err("objeto JSON sin cierre".into()),
            }
        }

        Ok(JsonValue::Object(values))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let mut out = String::new();

        loop {
            let Some(ch) = self.next() else {
                return Err("cadena JSON sin cierre".into());
            };

            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let Some(escaped) = self.next() else {
                        return Err("escape JSON incompleto".into());
                    };

                    match escaped {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => out.push(self.parse_unicode_escape()?),
                        other => {
                            return Err(format!("escape JSON no válido: \\{}", other));
                        }
                    }
                }
                ch if (ch as u32) < 0x20 => {
                    return Err("carácter de control no escapado en cadena JSON".into());
                }
                ch => out.push(ch),
            }
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        let first = self.parse_hex_quad()?;

        if (0xD800..=0xDBFF).contains(&first) {
            if self.next() != Some('\\') || self.next() != Some('u') {
                return Err("surrogate alto sin surrogate bajo".into());
            }

            let second = self.parse_hex_quad()?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Err("surrogate bajo no válido".into());
            }

            let codepoint = 0x10000 + (((first - 0xD800) as u32) << 10) + (second - 0xDC00) as u32;
            return char::from_u32(codepoint)
                .ok_or_else(|| "punto de código Unicode no válido".to_string());
        }

        if (0xDC00..=0xDFFF).contains(&first) {
            return Err("surrogate bajo aislado".into());
        }

        char::from_u32(first as u32).ok_or_else(|| "punto de código Unicode no válido".to_string())
    }

    fn parse_hex_quad(&mut self) -> Result<u16, String> {
        let mut value = 0u16;

        for _ in 0..4 {
            let Some(ch) = self.next() else {
                return Err("escape Unicode incompleto".into());
            };

            let digit = ch
                .to_digit(16)
                .ok_or_else(|| format!("dígito hexadecimal no válido: {}", ch))?;
            value = (value << 4) | digit as u16;
        }

        Ok(value)
    }

    fn parse_number(&mut self) -> Result<String, String> {
        let start = self.pos;

        if self.peek() == Some('-') {
            self.pos += 1;
        }

        match self.peek() {
            Some('0') => {
                self.pos += 1;
                if matches!(self.peek(), Some('0'..='9')) {
                    return Err("número JSON con cero inicial".into());
                }
            }
            Some('1'..='9') => {
                self.pos += 1;
                while matches!(self.peek(), Some('0'..='9')) {
                    self.pos += 1;
                }
            }
            _ => return Err("número JSON incompleto".into()),
        }

        if self.peek() == Some('.') {
            self.pos += 1;
            let fraction_start = self.pos;
            while matches!(self.peek(), Some('0'..='9')) {
                self.pos += 1;
            }
            if self.pos == fraction_start {
                return Err("fracción JSON incompleta".into());
            }
        }

        if matches!(self.peek(), Some('e' | 'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.pos += 1;
            }

            let exponent_start = self.pos;
            while matches!(self.peek(), Some('0'..='9')) {
                self.pos += 1;
            }
            if self.pos == exponent_start {
                return Err("exponente JSON incompleto".into());
            }
        }

        Ok(self.chars[start..self.pos].iter().collect())
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), String> {
        for expected in literal.chars() {
            match self.next() {
                Some(actual) if actual == expected => {}
                _ => return Err(format!("literal JSON incompleto: {}", literal)),
            }
        }

        Ok(())
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.next() {
            Some(actual) if actual == expected => Ok(()),
            Some(actual) => Err(format!(
                "se esperaba '{}' y se encontró '{}' en la posición {}",
                expected, actual, self.pos
            )),
            None => Err(format!("se esperaba '{}' al final del JSON", expected)),
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\n' | '\r' | '\t')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let value = self.peek();
        if value.is_some() {
            self.pos += 1;
        }
        value
    }
}

fn parse_csv(text: &str) -> Result<Vec<Vec<String>>, String> {
    if text.is_empty() {
        return Ok(Vec::new());
    }

    let normalized = text.replace("\r\n", "\n");
    if normalized.contains('\r') {
        return Err("se encontró un retorno de carro aislado".into());
    }

    let chars: Vec<char> = normalized.chars().collect();
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut pos = 0usize;
    let mut in_quotes = false;
    let mut closed_quote = false;

    while pos < chars.len() {
        let ch = chars[pos];

        if in_quotes {
            if ch == '"' {
                if pos + 1 < chars.len() && chars[pos + 1] == '"' {
                    field.push('"');
                    pos += 2;
                    continue;
                }

                in_quotes = false;
                closed_quote = true;
                pos += 1;
                continue;
            }

            field.push(ch);
            pos += 1;
            continue;
        }

        if closed_quote {
            match ch {
                ',' => {
                    row.push(std::mem::take(&mut field));
                    closed_quote = false;
                    pos += 1;
                }
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    closed_quote = false;
                    pos += 1;
                }
                _ => {
                    return Err(format!(
                        "carácter inesperado después de una celda entrecomillada en la posición {}",
                        pos + 1
                    ));
                }
            }
            continue;
        }

        match ch {
            '"' if field.is_empty() => {
                in_quotes = true;
                pos += 1;
            }
            '"' => {
                return Err(format!(
                    "comilla inesperada dentro de una celda no entrecomillada en la posición {}",
                    pos + 1
                ));
            }
            ',' => {
                row.push(std::mem::take(&mut field));
                pos += 1;
            }
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
                pos += 1;
            }
            _ => {
                field.push(ch);
                pos += 1;
            }
        }
    }

    if in_quotes {
        return Err("celda CSV entrecomillada sin cierre".into());
    }

    if closed_quote || !field.is_empty() || !row.is_empty() || !normalized.ends_with('\n') {
        row.push(field);
        rows.push(row);
    }

    Ok(rows)
}

fn first_width_difference(
    left: &[Vec<String>],
    right: &[Vec<String>],
) -> Option<(usize, usize, usize)> {
    left.iter()
        .zip(right.iter())
        .enumerate()
        .find_map(|(index, (left_row, right_row))| {
            if left_row.len() != right_row.len() {
                Some((index, left_row.len(), right_row.len()))
            } else {
                None
            }
        })
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!(
            "envmorph-oracle-test-{}-{}",
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn byte_oracle_reports_identical() {
        let left = ArtifactRef::new(temp_file("left.bin", b"abc"));
        let right = ArtifactRef::new(temp_file("right.bin", b"abc"));
        assert_eq!(
            ByteOracle.compare(&left, &right).status(),
            EquivalenceStatus::Identical
        );
    }

    #[test]
    fn byte_oracle_reports_different() {
        let left = ArtifactRef::new(temp_file("left.bin", b"abc"));
        let right = ArtifactRef::new(temp_file("right.bin", b"abd"));
        assert_eq!(
            ByteOracle.compare(&left, &right).status(),
            EquivalenceStatus::Different
        );
    }

    #[test]
    fn byte_oracle_can_use_verified_hash_mismatch() {
        let left = ArtifactRef::with_verified_sha256(temp_file("left.bin", b"abc"), "1111");
        let right = ArtifactRef::with_verified_sha256(temp_file("right.bin", b"abd"), "2222");
        let result = ByteOracle.compare(&left, &right);
        assert_eq!(result.status(), EquivalenceStatus::Different);
        assert_eq!(
            result.metadata().get("fast_path").map(String::as_str),
            Some("verified_sha256_mismatch")
        );
    }

    #[test]
    fn missing_artifact_is_not_execution_error() {
        let missing =
            ArtifactRef::new(env::temp_dir().join("envmorph-oracle-definitely-missing-left"));
        let right = ArtifactRef::new(temp_file("right.bin", b"abc"));
        assert_eq!(
            ByteOracle.compare(&missing, &right).status(),
            EquivalenceStatus::Missing
        );
    }

    #[test]
    fn text_oracle_is_strict_by_default() {
        let left = ArtifactRef::new(temp_file("left.txt", b"a\r\n"));
        let right = ArtifactRef::new(temp_file("right.txt", b"a\n"));
        let result = TextOracle::new(TextOptions::default()).compare(&left, &right);
        assert_eq!(result.status(), EquivalenceStatus::Different);
    }

    #[test]
    fn text_oracle_normalizes_only_selected_rules() {
        let left = ArtifactRef::new(temp_file("left.txt", b"a  \r\nb\t\r\n"));
        let right = ArtifactRef::new(temp_file("right.txt", b"a\nb"));
        let result = TextOracle::new(TextOptions {
            normalize_line_endings: true,
            trim_trailing_whitespace: true,
            ignore_final_newline: true,
        })
        .compare(&left, &right);
        assert_eq!(result.status(), EquivalenceStatus::Equivalent);
    }

    #[test]
    fn text_oracle_rejects_invalid_utf8() {
        let left = ArtifactRef::new(temp_file("left.txt", &[0xff]));
        let right = ArtifactRef::new(temp_file("right.txt", b"x"));
        let result = TextOracle::new(TextOptions::default()).compare(&left, &right);
        assert_eq!(result.status(), EquivalenceStatus::Error);
    }

    #[test]
    fn json_object_key_order_is_not_significant() {
        let left = ArtifactRef::new(temp_file("left.json", br#"{"a":1,"b":2}"#));
        let right = ArtifactRef::new(temp_file("right.json", br#"{"b":2,"a":1}"#));
        assert_eq!(
            JsonOracle.compare(&left, &right).status(),
            EquivalenceStatus::Equivalent
        );
    }

    #[test]
    fn json_array_order_is_significant() {
        let left = ArtifactRef::new(temp_file("left.json", br#"[1,2]"#));
        let right = ArtifactRef::new(temp_file("right.json", br#"[2,1]"#));
        assert_eq!(
            JsonOracle.compare(&left, &right).status(),
            EquivalenceStatus::Different
        );
    }

    #[test]
    fn json_duplicate_keys_are_rejected() {
        assert!(JsonParser::parse(r#"{"a":1,"a":2}"#).is_err());
    }

    #[test]
    fn json_unicode_surrogate_pair_is_supported() {
        let value = JsonParser::parse(r#""\uD83D\uDE00""#).unwrap();
        assert_eq!(value, JsonValue::String("😀".into()));
    }

    #[test]
    fn json_parse_error_is_explicit() {
        let left = ArtifactRef::new(temp_file("left.json", br#"{"a":}"#));
        let right = ArtifactRef::new(temp_file("right.json", br#"{"a":1}"#));
        assert_eq!(
            JsonOracle.compare(&left, &right).status(),
            EquivalenceStatus::Error
        );
    }

    #[test]
    fn csv_representation_can_be_equivalent() {
        let left = ArtifactRef::new(temp_file("left.csv", b"a,\"b\"\r\n"));
        let right = ArtifactRef::new(temp_file("right.csv", b"a,b\n"));
        assert_eq!(
            CsvOracle.compare(&left, &right).status(),
            EquivalenceStatus::Equivalent
        );
    }

    #[test]
    fn csv_row_order_is_significant() {
        let left = ArtifactRef::new(temp_file("left.csv", b"a\nb\n"));
        let right = ArtifactRef::new(temp_file("right.csv", b"b\na\n"));
        assert_eq!(
            CsvOracle.compare(&left, &right).status(),
            EquivalenceStatus::Different
        );
    }

    #[test]
    fn csv_row_count_difference_has_reason() {
        let left = ArtifactRef::new(temp_file("left.csv", b"a\nb\n"));
        let right = ArtifactRef::new(temp_file("right.csv", b"a\n"));
        let result = CsvOracle.compare(&left, &right);
        assert_eq!(result.status(), EquivalenceStatus::Different);
        assert!(result.reason().contains("cantidad de filas"));
    }

    #[test]
    fn csv_unclosed_quote_is_error() {
        let left = ArtifactRef::new(temp_file("left.csv", b"\"a\n"));
        let right = ArtifactRef::new(temp_file("right.csv", b"a\n"));
        assert_eq!(
            CsvOracle.compare(&left, &right).status(),
            EquivalenceStatus::Error
        );
    }

    #[test]
    fn comparison_report_json_is_deterministic() {
        let left = ArtifactRef::new(PathBuf::from("a"));
        let right = ArtifactRef::new(PathBuf::from("b"));
        let result =
            EquivalenceResult::new(EquivalenceStatus::Equivalent, "estructura JSON equivalente")
                .with_metadata("z", "2")
                .with_metadata("a", "1");
        let report = ComparisonReport::new("json", &left, &right, result);
        assert_eq!(report.render_json(), report.render_json());
        assert!(report.render_json().contains("\"result\":\"equivalent\""));
        assert!(
            report.render_json().find("\"a\":\"1\"").unwrap()
                < report.render_json().find("\"z\":\"2\"").unwrap()
        );
    }

    #[test]
    fn status_exit_codes_are_stable() {
        assert_eq!(EquivalenceStatus::Identical.exit_code(), 0);
        assert_eq!(EquivalenceStatus::Equivalent.exit_code(), 0);
        assert_eq!(EquivalenceStatus::Different.exit_code(), 1);
        assert_eq!(EquivalenceStatus::Missing.exit_code(), 2);
        assert_eq!(EquivalenceStatus::Error.exit_code(), 2);
    }
}
