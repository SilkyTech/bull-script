use core::fmt::Debug;
use regex::Regex;

use crate::error::error_at;

pub struct Lexer {
    pub text: String,
    pub filename: String,
}

#[derive(Debug)]
pub enum Token<'a> {
    ImportKeyword(),
    OnKeyword(),
    EndKeyword(),
    ThenKeyword(),
    ProcKeyword(),

    Identifier(Vec<String>),
    Unknown(String),
    LibraryPath(String),
    RelativePath(String),
    StringLiteral(String),
    NumericLiteral(f64, String),
    BuiltinType(&'a str),

    Child(),
    OpenParen(),
    CloseParen(),
    OpenSquare(),
    CloseSquare(),
    Semicolon(),

    OperatorAdd(),
    OperatorSubtract(),
    OperatorMultiply(),
    OperatorDivide(),
    OperatorMod(),
    OperatorSet(),
    OperatorEquals(),
    OperatorLogicalAnd(),
    OperatorLogicalOr(),
    OperatorLogicalNot(),
    OperatorGreater(),
    OperatorLesser()
}

pub struct TWL<'a> {
    token: Token<'a>,
    charn: i32,
    linen: i32,
    filen: String,
}

struct Loc {
    charn: i32,
    linen: i32,
    filen: String,
}

impl Debug for TWL<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "At {}:{}:{} | {:?}",
            self.filen, self.linen, self.charn, self.token
        )
    }
}

struct Keyword<'a> {
    str: &'a str,
    token: Token<'a>,
}
const KEYWORDS: [Keyword; 9] = [
    Keyword {
        str: "import",
        token: Token::ImportKeyword(),
    },
    Keyword {
        str: "on",
        token: Token::OnKeyword(),
    },
    Keyword {
        str: "is",
        token: Token::OperatorEquals(),
    },
    Keyword {
        str: "or",
        token: Token::OperatorLogicalOr(),
    },
    Keyword {
        str: "and",
        token: Token::OperatorLogicalAnd(),
    },
    Keyword {
        str: "not",
        token: Token::OperatorLogicalNot(),
    },
    Keyword {
        str: "end",
        token: Token::EndKeyword()
    },
    Keyword {
        str: "then",
        token: Token::ThenKeyword()
    },
    Keyword {
        str: "proc",
        token: Token::ProcKeyword()
    }
];

const SEPERATORS: [Keyword; 12] = [
    Keyword {
        str: "(",
        token: Token::OpenParen(),
    },
    Keyword {
        str: ")",
        token: Token::CloseParen(),
    },
    Keyword {
        str: "[",
        token: Token::OpenSquare(),
    },
    Keyword {
        str: "]",
        token: Token::CloseSquare(),
    },
    Keyword {
        str: "+",
        token: Token::OperatorAdd(),
    },
    Keyword {
        str: "-",
        token: Token::OperatorSubtract(),
    },
    Keyword {
        str: "*",
        token: Token::OperatorMultiply(),
    },
    Keyword {
        str: "/",
        token: Token::OperatorDivide(),
    },
    Keyword {
        str: "%",
        token: Token::OperatorMod(),
    },
    Keyword {
        str: "=",
        token: Token::OperatorSet(),
    },
    
    Keyword {
        str: ";",
        token: Token::Semicolon(),
    },
    Keyword {
        str: ":",
        token: Token::ThenKeyword(),
    }
];

const BUILTIN_TYPES: [Keyword; 3] = [
    Keyword {
        str: "int",
        token: Token::BuiltinType("int"),
    },
    Keyword {
        str: "str",
        token: Token::BuiltinType("str"),
    },
    Keyword {
        str: "bool",
        token: Token::BuiltinType("bool"),
    },
];

impl Lexer {
    pub fn lex(&self) -> Vec<TWL> {
        let mut in_string = false;
        let mut in_escape = false;
        let mut buffer = String::from("");
        let mut tokens: Vec<TWL> = vec![];
        let mut linen = 1;
        let mut charn = 1;
        let filen = self.filename.clone();

        let add_token = |buffer: &mut String, tokens: &mut Vec<TWL>, linen: i32, charn: i32| {
            if buffer == "" {
                return;
            }
            let mut found = false;
            for keyword_rule in KEYWORDS {
                if buffer == keyword_rule.str {
                    tokens.push(TWL {
                        token: keyword_rule.token,
                        charn: charn.clone(),
                        linen: linen.clone(),
                        filen: filen.clone(),
                    });

                    found = true;
                    break;
                }
            }
            if !found {
                if buffer.len() > 0 && is_valid_identifier(&buffer) {
                    let mut found = false;
                    for _type in BUILTIN_TYPES {
                        if buffer == _type.str {
                            found = true;
                            tokens.push(TWL {
                                token: _type.token,
                                charn: charn.clone(),
                                linen: linen.clone(),
                                filen: filen.clone(),
                            });
                            break;
                        }
                    }
                    if !found {
                        tokens.push(TWL {
                            token: Token::Identifier(
                                buffer
                                    .split(".")
                                    .collect::<Vec<&str>>()
                                    .iter()
                                    .map(|f| f.to_string())
                                    .collect(),
                            ),
                            charn: charn.clone(),
                            linen: linen.clone(),
                            filen: filen.clone(),
                        });
                    }
                } else if is_valid_standard_path(&buffer) {
                    tokens.push(TWL {
                        token: Token::LibraryPath(buffer.to_owned()),
                        charn: charn.clone(),
                        linen: linen.clone(),
                        filen: filen.clone(),
                    });
                } else if is_valid_number(
                    &buffer,
                    &Loc {
                        charn: charn.clone(),
                        filen: filen.clone(),
                        linen: linen.clone(),
                    },
                )
                .0
                {
                    let valid_num = is_valid_number(
                        &buffer,
                        &Loc {
                            charn: charn.clone(),
                            filen: filen.clone(),
                            linen: linen.clone(),
                        },
                    );
                    tokens.push(TWL {
                        token: Token::NumericLiteral(valid_num.2, valid_num.1),
                        charn: charn.clone(),
                        linen: linen.clone(),
                        filen: filen.clone(),
                    })
                } else {
                    tokens.push(TWL {
                        token: Token::Unknown(buffer.to_owned()),
                        charn: charn.clone(),
                        linen: linen.clone(),
                        filen: filen.clone(),
                    });
                    error_at(
                        &filen,
                        &linen,
                        &charn,
                        &format!("\"{}\" is not a valid token", &buffer),
                    );
                }
            }
        };

        for char in self.text.chars() {
            if char == '\n' {
                linen += 1;
                charn = 1;
            }
            if in_string && in_escape {
                buffer += &(match &char {
                    '\\' => '\\'.to_string(),
                    'a' => (0x07 as char).to_string(),
                    'b' => (0x08 as char).to_string(),
                    'e' => (0x1b as char).to_string(),
                    'f' => (0x0c as char).to_string(),
                    'n' => '\n'.to_string(),
                    'r' => '\r'.to_string(),
                    't' => '\t'.to_string(),
                    'v' => (0x0b as char).to_string(),
                    '\'' => '\''.to_string(),
                    '"' => '"'.to_string(),
                    _ => error_at(&filen, &linen, &charn, &"Invalid escape character".to_string())
                });
                in_escape = false;
            } else if in_string && char == '\\' {
                in_escape = true;
            } else if char == '"' {
                in_string = !in_string;
                if !in_string {
                    tokens.push(TWL {
                        token: Token::StringLiteral(buffer.to_owned()),
                        charn: charn.clone(),
                        linen: linen.clone(),
                        filen: filen.clone(),
                    });
                    buffer = String::from("");
                }
            } else if in_string {
                buffer += &char.to_string();
            } else if char == ' ' || char == '\n' {
                add_token(&mut buffer, &mut tokens, linen, charn);
                buffer = String::from("");
            } else {
                let mut found = false;
                for sep in SEPERATORS {
                    if char.to_string() == sep.str {
                        add_token(&mut buffer, &mut tokens, linen, charn);
                        buffer = String::from("");
                        found = true;
                        tokens.push(TWL {
                            token: sep.token,
                            charn,
                            linen,
                            filen: filen.clone(),
                        });
                        break;
                    }
                }
                if !found {
                    buffer += &char.to_string();
                }
            }

            charn += 1;
        }
        add_token(&mut buffer, &mut tokens, linen, charn);

        tokens
    }
}

fn is_valid_identifier(str: &String) -> bool {
    let re = Regex::new(r"^([a-zA-Z0-9_]+\.?)*$").expect("Invalid regex at `is_valid_identifier`");
    let (first, rem) = str_frem(&str);
    if str.len() < 1 {
        return false;
    }
    let first_char = first
        .chars()
        .nth(0)
        .expect("Unreachable: `is_valid_identifier`");
    if first_char.is_alphabetic() || first_char == '_' {
        if re.is_match(&rem) {
            return true;
        }
    }
    return false;
}
fn is_valid_standard_path(str: &String) -> bool {
    let re = Regex::new(r"^<[a-zA-Z0-9_]*>$").expect("Invalid regex at `is_valid_standard_path`");
    re.is_match(str)
}
fn is_valid_number(str: &String, loc: &Loc) -> (bool, String, f64) {
    let re = Regex::new(r"^[0-9]+(\.[0-9]+f)?$").expect("Invalid regex at `is_valid_number`");
    if re.is_match(&str) {
        if str.ends_with("f") {
            let float = str[0..str.len() - 2].parse::<f64>();
            match float {
                Ok(value) => {
                    return (true, "float".to_owned(), value);
                }
                Err(_) => {
                    error_at(
                        &loc.filen,
                        &loc.linen,
                        &loc.charn,
                        &format!("Failed to parse float value: {}", &str),
                    );
                    return (false, "float".to_owned(), -1.0);
                }
            }
        } else if is_string_numeric(str) {
            let float = str.parse::<f64>();
            match float {
                Ok(value) => {
                    return (true, "int".to_owned(), value);
                }
                Err(_) => return (false, "int".to_owned(), -1.0),
            }
        } else {
            return (false, "".to_owned(), -1.0);
        }
    } else {
        (false, "".to_owned(), -1.0)
    }
}

fn is_string_numeric(str: &String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    return true;
}

/// Splits a string into the first character
/// and the remainder
///
/// Example (psuedocode):
///
/// `str_frem("test") -> ("t", "est")`
fn str_frem(s: &str) -> (&str, &str) {
    for i in 1..=s.len() {
        let r = s.get(0..i);
        match r {
            Some(x) => return (x, &s[i..]),
            None => (),
        }
    }

    (&s[0..0], s)
}
