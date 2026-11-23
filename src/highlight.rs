use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, TextFormat};
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Go,
    Java,
    Json,
    Toml,
    Shell,
    Html,
    Css,
    Sql,
    Plain,
}

impl Language {
    pub fn from_setting(value: &str) -> Self {
        match value {
            "rust" => Self::Rust,
            "python" => Self::Python,
            "javascript" => Self::JavaScript,
            "typescript" => Self::TypeScript,
            "c" => Self::C,
            "cpp" => Self::Cpp,
            "go" => Self::Go,
            "java" => Self::Java,
            "json" => Self::Json,
            "toml" => Self::Toml,
            "shell" => Self::Shell,
            "html" => Self::Html,
            "css" => Self::Css,
            "sql" => Self::Sql,
            _ => Self::Plain,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "rs" => Self::Rust,
            "py" | "pyw" => Self::Python,
            "js" | "mjs" | "cjs" | "jsx" => Self::JavaScript,
            "ts" | "tsx" | "mts" | "cts" => Self::TypeScript,
            "c" | "h" => Self::C,
            "cc" | "cpp" | "cxx" | "hpp" | "hh" | "hxx" => Self::Cpp,
            "go" => Self::Go,
            "java" => Self::Java,
            "json" => Self::Json,
            "toml" => Self::Toml,
            "sh" | "bash" | "zsh" | "fish" => Self::Shell,
            "html" | "htm" | "xhtml" => Self::Html,
            "css" | "scss" | "sass" | "less" => Self::Css,
            "sql" => Self::Sql,
            _ => Self::Plain,
        }
    }

    fn line_comments(self) -> &'static [&'static str] {
        match self {
            Self::Rust
            | Self::JavaScript
            | Self::TypeScript
            | Self::C
            | Self::Cpp
            | Self::Go
            | Self::Java => &["//"],
            Self::Python | Self::Toml | Self::Shell => &["#"],
            Self::Sql => &["--"],
            _ => &[],
        }
    }

    fn block_comment(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Rust
            | Self::JavaScript
            | Self::TypeScript
            | Self::C
            | Self::Cpp
            | Self::Go
            | Self::Java
            | Self::Css
            | Self::Sql => Some(("/*", "*/")),
            Self::Html => Some(("<!--", "-->")),
            _ => None,
        }
    }

    fn string_delims(self) -> &'static [char] {
        match self {
            Self::Python | Self::JavaScript | Self::TypeScript | Self::Shell => {
                &['"', '\'', '`']
            }
            Self::Rust | Self::C | Self::Cpp | Self::Go | Self::Java | Self::Json | Self::Toml
            | Self::Html | Self::Css | Self::Sql => &['"', '\''],
            _ => &[],
        }
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Rust => &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
                "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while", "union", "i8",
                "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
                "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Some",
                "None", "Ok", "Err", "Box",
            ],
            Self::Python => &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
                "elif", "else", "except", "False", "finally", "for", "from", "global", "if",
                "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise",
                "return", "True", "try", "while", "with", "yield", "match", "case", "self", "cls",
                "int", "float", "str", "bool", "list", "dict", "set", "tuple", "bytes",
            ],
            Self::JavaScript | Self::TypeScript => &[
                "break", "case", "catch", "class", "const", "continue", "debugger", "default",
                "delete", "do", "else", "export", "extends", "finally", "for", "function", "if",
                "import", "in", "instanceof", "let", "new", "return", "super", "switch", "this",
                "throw", "try", "typeof", "var", "void", "while", "with", "yield", "async", "await",
                "static", "get", "set", "of", "from", "as", "interface", "type", "enum",
                "implements", "public", "private", "protected", "readonly", "declare", "namespace",
                "abstract", "keyof", "infer", "is", "true", "false", "null", "undefined", "NaN",
                "Infinity",
            ],
            Self::C | Self::Cpp => &[
                "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
                "else", "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long",
                "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct",
                "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "_Bool",
                "bool", "class", "namespace", "template", "typename", "public", "private",
                "protected", "virtual", "override", "final", "new", "delete", "this", "nullptr",
                "using", "try", "catch", "throw", "constexpr", "static_cast", "dynamic_cast",
                "const_cast", "reinterpret_cast", "true", "false",
            ],
            Self::Go => &[
                "break", "case", "chan", "const", "continue", "default", "defer", "else",
                "fallthrough", "for", "func", "go", "goto", "if", "import", "interface", "map",
                "package", "range", "return", "select", "struct", "switch", "type", "var", "nil",
                "true", "false", "iota", "string", "int", "int8", "int16", "int32", "int64", "uint",
                "uint8", "uint16", "uint32", "uint64", "float32", "float64", "complex64",
                "complex128", "byte", "rune", "bool", "error", "any",
            ],
            Self::Java => &[
                "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
                "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
                "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int",
                "interface", "long", "native", "new", "package", "private", "protected", "public",
                "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
                "throw", "throws", "transient", "try", "void", "volatile", "while", "true",
                "false", "null", "var", "record", "sealed", "permits", "yield", "String",
            ],
            Self::Json => &["true", "false", "null"],
            Self::Toml => &["true", "false"],
            Self::Shell => &[
                "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac",
                "function", "in", "select", "until", "time", "coproc", "return", "break",
                "continue", "local", "export", "readonly", "declare", "unset", "shift", "exit",
                "source", "alias", "true", "false", "test",
            ],
            Self::Sql => &[
                "select", "from", "where", "insert", "update", "delete", "create", "table", "drop",
                "alter", "join", "inner", "left", "right", "full", "outer", "on", "group", "by",
                "order", "having", "limit", "offset", "as", "and", "or", "not", "null", "is", "in",
                "like", "between", "distinct", "union", "all", "case", "when", "then", "else",
                "end", "primary", "key", "foreign", "references", "index", "values", "into", "set",
                "database", "varchar", "int", "text", "timestamp", "boolean", "select",
            ],
            Self::Html | Self::Css | Self::Plain => &[],
        }
    }

    fn case_insensitive(self) -> bool {
        matches!(self, Self::Sql)
    }

    fn allows_dollar(self) -> bool {
        matches!(
            self,
            Self::JavaScript | Self::TypeScript | Self::Shell | Self::Css
        )
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub keyword: Color32,
    pub string: Color32,
    pub comment: Color32,
    pub number: Color32,
    pub function: Color32,
    pub type_name: Color32,
    pub macro_name: Color32,
}

impl Palette {
    pub fn new(dark: bool, theme: &str) -> Self {
        match (dark, theme) {
            (true, "vivid") => Self {
                keyword: Color32::from_rgb(255, 123, 114),
                string: Color32::from_rgb(126, 231, 135),
                comment: Color32::from_rgb(139, 148, 158),
                number: Color32::from_rgb(121, 192, 255),
                function: Color32::from_rgb(210, 168, 255),
                type_name: Color32::from_rgb(255, 166, 87),
                macro_name: Color32::from_rgb(255, 123, 114),
            },
            (true, "soft") => Self {
                keyword: Color32::from_rgb(180, 167, 214),
                string: Color32::from_rgb(157, 193, 131),
                comment: Color32::from_rgb(122, 122, 122),
                number: Color32::from_rgb(217, 160, 102),
                function: Color32::from_rgb(143, 175, 217),
                type_name: Color32::from_rgb(217, 201, 122),
                macro_name: Color32::from_rgb(217, 143, 175),
            },
            (true, _) => Self {
                keyword: Color32::from_rgb(199, 146, 234),
                string: Color32::from_rgb(195, 232, 141),
                comment: Color32::from_rgb(106, 115, 125),
                number: Color32::from_rgb(247, 140, 108),
                function: Color32::from_rgb(130, 170, 255),
                type_name: Color32::from_rgb(255, 203, 107),
                macro_name: Color32::from_rgb(255, 83, 112),
            },
            (false, "vivid") => Self {
                keyword: Color32::from_rgb(207, 34, 46),
                string: Color32::from_rgb(17, 99, 41),
                comment: Color32::from_rgb(110, 119, 129),
                number: Color32::from_rgb(5, 80, 174),
                function: Color32::from_rgb(130, 80, 223),
                type_name: Color32::from_rgb(149, 56, 0),
                macro_name: Color32::from_rgb(207, 34, 46),
            },
            (false, "soft") => Self {
                keyword: Color32::from_rgb(124, 90, 168),
                string: Color32::from_rgb(80, 130, 70),
                comment: Color32::from_rgb(130, 130, 130),
                number: Color32::from_rgb(160, 110, 60),
                function: Color32::from_rgb(70, 110, 160),
                type_name: Color32::from_rgb(150, 125, 60),
                macro_name: Color32::from_rgb(160, 85, 110),
            },
            (false, _) => Self {
                keyword: Color32::from_rgb(124, 58, 237),
                string: Color32::from_rgb(26, 127, 55),
                comment: Color32::from_rgb(110, 119, 129),
                number: Color32::from_rgb(179, 89, 0),
                function: Color32::from_rgb(9, 105, 218),
                type_name: Color32::from_rgb(154, 103, 0),
                macro_name: Color32::from_rgb(207, 34, 46),
            },
        }
    }
}

pub fn layout_job(
    text: &str,
    font: &FontId,
    language: Language,
    palette: &Palette,
    plain: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut block_end: Option<(&'static str, Color32)> = None;

    for line in text.split_inclusive('\n') {
        let has_newline = line.ends_with('\n');
        let content = if has_newline {
            &line[..line.len() - 1]
        } else {
            line
        };

        let mut pos = 0usize;

        while pos < content.len() {
            if let Some((end_marker, color)) = block_end {
                if let Some(found) = content[pos..].find(end_marker) {
                    let end = pos + found + end_marker.len();
                    append(&mut job, &content[pos..end], color, font);
                    pos = end;
                    block_end = None;
                } else {
                    append(&mut job, &content[pos..], color, font);
                    pos = content.len();
                }
                continue;
            }

            if let Some(_prefix) = language
                .line_comments()
                .iter()
                .find(|prefix| content[pos..].starts_with(**prefix))
            {
                append(&mut job, &content[pos..], palette.comment, font);
                pos = content.len();
                continue;
            }

            if let Some((open, close)) = language.block_comment() {
                if content[pos..].starts_with(open) {
                    if let Some(found) = content[pos + open.len()..].find(close) {
                        let end = pos + open.len() + found + close.len();
                        append(&mut job, &content[pos..end], palette.comment, font);
                        pos = end;
                    } else {
                        append(&mut job, &content[pos..], palette.comment, font);
                        block_end = Some((close, palette.comment));
                        pos = content.len();
                    }
                    continue;
                }
            }

            if language == Language::Python {
                let rest = &content[pos..];
                let triple = if rest.starts_with("\"\"\"") {
                    Some("\"\"\"")
                } else if rest.starts_with("'''") {
                    Some("'''")
                } else {
                    None
                };
                if let Some(triple) = triple {
                    if let Some(found) = rest[3..].find(triple) {
                        let end = pos + 3 + found + 3;
                        append(&mut job, &content[pos..end], palette.string, font);
                        pos = end;
                    } else {
                        append(&mut job, rest, palette.string, font);
                        block_end = Some((triple, palette.string));
                        pos = content.len();
                    }
                    continue;
                }
            }

            let ch = content[pos..].chars().next().unwrap();

            if language.string_delims().contains(&ch) {
                if let Some(end) = scan_string(content, pos, ch) {
                    append(&mut job, &content[pos..end], palette.string, font);
                    pos = end;
                } else if ch == '\'' {
                    let next = pos + ch.len_utf8();
                    append(&mut job, &content[pos..next], plain, font);
                    pos = next;
                } else {
                    append(&mut job, &content[pos..], palette.string, font);
                    pos = content.len();
                }
                continue;
            }

            if ch.is_ascii_digit() {
                let end = scan_number(content, pos);
                append(&mut job, &content[pos..end], palette.number, font);
                pos = end;
                continue;
            }

            if is_word_start(ch, language) {
                let end = scan_word(content, pos, language);
                let word = &content[pos..end];
                let color = word_color(language, word, &content[end..], palette, plain);
                append(&mut job, word, color, font);
                pos = end;
                continue;
            }

            let next = pos + ch.len_utf8();
            append(&mut job, &content[pos..next], plain, font);
            pos = next;
        }

        if has_newline {
            append(&mut job, "\n", plain, font);
        }
    }

    job
}

fn append(job: &mut LayoutJob, text: &str, color: Color32, font: &FontId) {
    if text.is_empty() {
        return;
    }
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font.clone(),
            color,
            ..Default::default()
        },
    );
}

fn scan_string(line: &str, start: usize, quote: char) -> Option<usize> {
    let mut escaped = false;
    let offset = start + quote.len_utf8();
    for (index, ch) in line[offset..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some(offset + index + ch.len_utf8());
        }
    }
    None
}

fn scan_number(line: &str, start: usize) -> usize {
    let mut end = start;
    for ch in line[start..].chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    end.max(start + 1)
}

fn is_word_start(ch: char, language: Language) -> bool {
    ch.is_alphabetic() || ch == '_' || (language.allows_dollar() && ch == '$')
}

fn scan_word(line: &str, start: usize, language: Language) -> usize {
    let mut end = start;
    for ch in line[start..].chars() {
        if ch.is_alphanumeric() || ch == '_' || (language.allows_dollar() && ch == '$') {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    end.max(start + 1)
}

fn word_color(
    language: Language,
    word: &str,
    rest: &str,
    palette: &Palette,
    plain: Color32,
) -> Color32 {
    let keyword = if language.case_insensitive() {
        let lower = word.to_ascii_lowercase();
        language.keywords().iter().any(|item| *item == lower.as_str())
    } else {
        language.keywords().iter().any(|item| *item == word)
    };
    if keyword {
        return palette.keyword;
    }

    if rest.trim_start().starts_with('!') {
        return palette.macro_name;
    }

    if rest.starts_with('(') {
        return palette.function;
    }

    if word.chars().next().is_some_and(char::is_uppercase) {
        return palette.type_name;
    }

    plain
}
