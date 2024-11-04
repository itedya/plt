use std::cmp::PartialEq;
use std::fmt::Display;
use std::str::FromStr;
use rustc_lexer::{LiteralKind, Token, TokenKind};

#[derive(Debug, Clone)]
enum TextCodeFSAState {
    ParsingText,
    ParsingCode,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Part {
    Text(String),
    Code(String),
}

impl Part {
    pub fn is_text(&self) -> bool {
        match self {
            Part::Text(_) => true,
            _ => false,
        }
    }

    pub fn get_content(&self) -> &String {
        match self {
            Part::Code(ref content) => content,
            Part::Text(ref content) => content,
        }
    }
}

// Text-code finite state automata
//
// It parses its input and splits it into code and text.
#[derive(Debug)]
pub struct TextCodeFSA {
    state: TextCodeFSAState,
    data: Vec<Part>,
}

#[cfg(test)]
pub fn dbg_vec_token(tokens: Vec<Token>, content: &str) {
    let mut token_idx = 0;
    for token in tokens {
        match token.kind {
            TokenKind::LineComment => {
                println!("Line Comment: {}", &content[token_idx..token_idx + token.len]);
            }
            TokenKind::BlockComment { terminated } => {
                if terminated {
                    println!("Terminated Block Comment: {}", &content[token_idx..token_idx + token.len]);
                } else {
                    println!("Unterminated Block Comment: {}", &content[token_idx..token_idx + token.len]);
                }
            }
            TokenKind::Whitespace => println!("Whitespace"),
            TokenKind::Ident => println!("Identifier: {}", &content[token_idx..token_idx + token.len]),
            TokenKind::RawIdent => println!("Raw Identifier: {}", &content[token_idx..token_idx + token.len]),
            TokenKind::Literal { kind, suffix_start } => {
                println!("Literal of kind {:?} with suffix at {}: {}", kind, suffix_start, &content[token_idx..token_idx + token.len]);
            }
            TokenKind::Lifetime { starts_with_number } => {
                if starts_with_number {
                    println!("Lifetime (starts with number): {}", &content[token_idx..token_idx + token.len]);
                } else {
                    println!("Lifetime: {}", &content[token_idx..token_idx + token.len]);
                }
            }
            TokenKind::Semi => println!("Semicolon: ;"),
            TokenKind::Comma => println!("Comma: ,"),
            TokenKind::Dot => println!("Dot: ."),
            TokenKind::OpenParen => println!("Open Parenthesis: ("),
            TokenKind::CloseParen => println!("Close Parenthesis: )"),
            TokenKind::OpenBrace => println!("Open Brace: {{"),
            TokenKind::CloseBrace => println!("Close Brace: }}"),
            TokenKind::OpenBracket => println!("Open Bracket: ["),
            TokenKind::CloseBracket => println!("Close Bracket: ]"),
            TokenKind::At => println!("At Symbol: @"),
            TokenKind::Pound => println!("Pound Symbol: #"),
            TokenKind::Tilde => println!("Tilde: ~"),
            TokenKind::Question => println!("Question Mark: ?"),
            TokenKind::Colon => println!("Colon: :"),
            TokenKind::Dollar => println!("Dollar Sign: $"),
            TokenKind::Eq => println!("Equals: ="),
            TokenKind::Lt => println!("Less Than: <"),
            TokenKind::Gt => println!("Greater Than: >"),
            TokenKind::Minus => println!("Minus: -"),
            TokenKind::And => println!("Ampersand: &"),
            TokenKind::Or => println!("Vertical Bar: |"),
            TokenKind::Plus => println!("Plus: +"),
            TokenKind::Star => println!("Asterisk: *"),
            TokenKind::Slash => println!("Slash: /"),
            TokenKind::Caret => println!("Caret: ^"),
            TokenKind::Not => println!("Not: !"),
            TokenKind::Percent => println!("Percent: %"),
            TokenKind::Unknown => println!("Unknown Token: {}", &content[token_idx..token_idx + token.len]),
        }

        token_idx += token.len;
    }
}

impl TextCodeFSA {
    pub fn new() -> TextCodeFSA {
        Self {
            state: TextCodeFSAState::ParsingText,
            data: Vec::new(),
        }
    }

    fn check_if_rust_code_is_valid(rust_code: &str) -> bool {
        proc_macro2::TokenStream::from_str(rust_code).is_ok()
    }

    fn get_last_part_content(&self) -> Option<&str> {
        if self.data.last().is_some() {
            Some(self.data.last().unwrap().get_content())
        } else {
            match self.state {
                TextCodeFSAState::ParsingCode => None,
                TextCodeFSAState::ParsingText => None,
            }
        }
    }

    fn tokenize_code_from_str(content: &str) -> Vec<Token> {
        rustc_lexer::tokenize(content).collect::<Vec<_>>()
    }

    fn is_inside_line_comment(tokens: &Vec<Token>) -> bool {
        tokens.iter().last()
            .map_or(false, |token| token.kind == TokenKind::LineComment)
    }

    fn is_inside_str_literal(tokens: &Vec<Token>) -> bool {
        tokens.iter().last()
            .map_or(false, |token| {
                match token.kind {
                    TokenKind::Literal { kind, .. } => {
                        match kind {
                            LiteralKind::Str { terminated } => terminated == false,
                            _ => false,
                        }
                    }
                    _ => false
                }
            })
    }

    fn push_char_to_latest_entry(&mut self, c: char) {
        let is_correct_type = match (&self.state, self.data.last()) {
            (TextCodeFSAState::ParsingCode, Some(Part::Code(_))) => true,
            (TextCodeFSAState::ParsingText, Some(Part::Text(_))) => true,
            _ => false,
        };

        if self.data.last().is_some() && is_correct_type {
            match self.data.last_mut().unwrap() {
                Part::Text(ref mut text) => text.push(c),
                Part::Code(ref mut code) => code.push(c),
            }
        } else {
            match self.state {
                TextCodeFSAState::ParsingText => self.data.push(Part::Text(c.to_string())),
                TextCodeFSAState::ParsingCode => self.data.push(Part::Code(c.to_string())),
            }
        }
    }

    pub fn run(&mut self, payload: String) -> &Vec<Part> {
        let payload_chars = payload.chars().collect::<Vec<_>>();

        let mut payload_char_index: usize = 0;

        while payload_char_index < payload_chars.len() {
            match self.state {
                TextCodeFSAState::ParsingCode => {
                    if payload[payload_char_index..].starts_with("?>") {
                        let latest_rust_code_part = self.get_last_part_content().unwrap_or("");

                        let tokens = Self::tokenize_code_from_str(latest_rust_code_part);

                        if Self::is_inside_str_literal(&tokens) {
                            self.push_char_to_latest_entry(payload_chars[payload_char_index]);
                            payload_char_index += 1;
                            continue;
                        }

                        if Self::is_inside_line_comment(&tokens) {
                            // dbg_vec_token(tokens, latest_rust_code_part);
                            self.push_char_to_latest_entry(payload_chars[payload_char_index]);
                            payload_char_index += 1;
                            continue;
                        }

                        // println!("inside line comment: {}", Self::is_inside_line_comment(&tokens));
                        // dbg_vec_token(tokens, latest_rust_code_part);

                        payload_char_index += "?>".len();
                        self.state = TextCodeFSAState::ParsingText;
                        continue;
                    } else {
                        self.push_char_to_latest_entry(payload_chars[payload_char_index]);
                    }
                }
                TextCodeFSAState::ParsingText => {
                    if payload[payload_char_index..].starts_with("<?rs") {
                        payload_char_index += "<?rs".len();
                        self.state = TextCodeFSAState::ParsingCode;
                        continue;
                    } else {
                        self.push_char_to_latest_entry(payload_chars[payload_char_index]);
                    }
                }
            }

            payload_char_index += 1;
        }

        &self.data
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use crate::text_code_fsa::{Part, TextCodeFSA};

    #[test]
    fn it_works() {
        let test_file = read_to_string("src/test-files/01.plt").unwrap();

        let mut fsa = TextCodeFSA::new();

        let result = fsa.run(test_file);

        assert_eq!(result.len(), 3);
        assert!(matches!(result[0].clone(), Part::Text(content) if content == "<!DOCTYPE html>\r\n<html>\r\n    <head>\r\n        <title>"));
        assert!(matches!(result[1].clone(), Part::Code(content) if content == " \"hello world\" "));
        assert!(matches!(result[2].clone(), Part::Text(content) if content == "</title>\r\n    </head>\r\n</html>"));
    }

    #[test]
    fn it_does_not_end_when_the_end_tag_is_inside_rust_string_literal() {
        let test_file = read_to_string("src/test-files/02.plt").unwrap();

        let mut fsa = TextCodeFSA::new();

        let result = fsa.run(test_file);

        assert_eq!(result.len(), 3);

        assert!(matches!(result[0].clone(), Part::Text(content) if content == "<!DOCTYPE html>\r\n<html>\r\n    <head>\r\n        <title>"));
        assert!(matches!(result[1].clone(), Part::Code(content) if content == " \"hello ?> world\" "));
        assert!(matches!(result[2].clone(), Part::Text(content) if content == "</title>\r\n    </head>\r\n</html>"));
    }

    #[test]
    fn it_does_not_end_when_the_end_tag_is_inside_rust_comment() {
        let test_file = read_to_string("src/test-files/03.plt").unwrap();

        let mut fsa = TextCodeFSA::new();

        let result = fsa.run(test_file);

        assert_eq!(result.len(), 3);

        assert!(matches!(result[0].clone(), Part::Text(content) if content == "<!DOCTYPE html>\r\n<html>\r\n    <head>\r\n        <title>"));
        assert!(matches!(result[1].clone(), Part::Code(content) if content == " \"hello ?> world\"; // some string\r\n        "));
        assert!(matches!(result[2].clone(), Part::Text(content) if content == "</title>\r\n    </head>\r\n</html>"));
    }

    #[test]
    fn it_ends_the_code_part_when_end_tag_is_incorrectly_placed_inside_the_line_comment() {
        let test_file = read_to_string("src/test-files/04.plt").unwrap();

        let mut fsa = TextCodeFSA::new();

        let result = fsa.run(test_file);

        assert_eq!(result.len(), 2);

        assert!(matches!(result[0].clone(), Part::Text(content) if content == "<!DOCTYPE html>\r\n<html>\r\n    <head>\r\n        <title>"));
        assert!(matches!(result[1].clone(), Part::Code(content) if content == " \"hello ?> world\"; // some string ?></title>\r\n    </head>\r\n</html>"));
    }

    #[test]
    fn it_omits_starting_sequence_inside_code_part() {
        let mut fsa = TextCodeFSA::new();

        let result = fsa.run("<?rs<?rs".to_string());

        assert_eq!(result.len(), 1);

        assert!(matches!(result[0].clone(), Part::Code(content) if content == "<?rs"));
    }

    #[test]
    fn it_handles_block_comments_correctly() {
        //TODO
        unimplemented!()
    }

    #[test]
    fn test_valid_rust_code_check() {
        assert!(TextCodeFSA::check_if_rust_code_is_valid(" \"hello world\" "));
        assert!(TextCodeFSA::check_if_rust_code_is_valid(" \"hello ?> world\" "));

        assert_eq!(TextCodeFSA::check_if_rust_code_is_valid(" \"hello ?"), false);
    }
}
