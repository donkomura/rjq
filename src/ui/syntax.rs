use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// JSONのトークンタイプ
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    String,      // "文字列"
    Number,      // 123, 3.14
    Boolean,     // true, false
    Null,        // null
    Key,         // オブジェクトのキー
    Bracket,     // [], {}
    Punctuation, // , :
    Default,     // その他
}

/// トークン情報
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

/// JSONシンタックスハイライター
pub struct SyntaxHighlighter;

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self
    }

    /// JSONをトークンに分解
    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.char_indices().peekable();

        while let Some((start, ch)) = chars.next() {
            match ch {
                // ホワイトスペース
                ' ' | '\t' | '\n' | '\r' => continue,

                // 文字列
                '"' => {
                    let mut end = start + 1;
                    let mut text = String::from("\"");
                    let mut escaped = false;

                    for (pos, ch) in chars.by_ref() {
                        text.push(ch);
                        end = pos + ch.len_utf8();

                        if escaped {
                            escaped = false;
                        } else if ch == '\\' {
                            escaped = true;
                        } else if ch == '"' {
                            break;
                        }
                    }

                    // 文字列の後にコロンがあるかチェックしてキーかどうか判定
                    let mut temp_chars = chars.clone();
                    let mut is_key = false;

                    // 空白をスキップしてコロンがあるかチェック
                    while let Some(&(_, next_ch)) = temp_chars.peek() {
                        if next_ch.is_whitespace() {
                            temp_chars.next();
                        } else if next_ch == ':' {
                            is_key = true;
                            break;
                        } else {
                            break;
                        }
                    }

                    tokens.push(Token {
                        token_type: if is_key {
                            TokenType::Key
                        } else {
                            TokenType::String
                        },
                        text,
                        start,
                        end,
                    });
                }

                // 数値
                '0'..='9' | '-' => {
                    let mut end = start;
                    let mut text = String::new();
                    text.push(ch);
                    end += ch.len_utf8();

                    // 数値の続きを読む
                    while let Some(&(pos, next_ch)) = chars.peek() {
                        if next_ch.is_ascii_digit()
                            || next_ch == '.'
                            || next_ch == 'e'
                            || next_ch == 'E'
                            || next_ch == '+'
                            || next_ch == '-'
                        {
                            text.push(next_ch);
                            end = pos + next_ch.len_utf8();
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    // マイナス単体の場合は数値ではない
                    if text == "-" {
                        tokens.push(Token {
                            token_type: TokenType::Default,
                            text,
                            start,
                            end,
                        });
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Number,
                            text,
                            start,
                            end,
                        });
                    }
                }

                // ブラケット
                '[' | ']' | '{' | '}' => {
                    tokens.push(Token {
                        token_type: TokenType::Bracket,
                        text: ch.to_string(),
                        start,
                        end: start + ch.len_utf8(),
                    });
                }

                // 区切り文字
                ',' | ':' => {
                    tokens.push(Token {
                        token_type: TokenType::Punctuation,
                        text: ch.to_string(),
                        start,
                        end: start + ch.len_utf8(),
                    });
                }

                // その他（true, false, null など）
                _ => {
                    let mut end = start;
                    let mut text = String::new();
                    text.push(ch);
                    end += ch.len_utf8();

                    // 連続する英字を読む
                    while let Some(&(pos, next_ch)) = chars.peek() {
                        if next_ch.is_alphabetic() {
                            text.push(next_ch);
                            end = pos + next_ch.len_utf8();
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    let token_type = self.classify_token(&text);
                    tokens.push(Token {
                        token_type,
                        text,
                        start,
                        end,
                    });
                }
            }
        }

        tokens
    }

    /// トークンの種類を分類
    fn classify_token(&self, text: &str) -> TokenType {
        match text {
            "true" | "false" => TokenType::Boolean,
            "null" => TokenType::Null,
            _ => TokenType::Default,
        }
    }

    /// トークンタイプに応じたスタイルを取得
    pub fn get_style(&self, token_type: &TokenType) -> Style {
        match token_type {
            TokenType::String => Style::default().fg(Color::Green),
            TokenType::Number => Style::default().fg(Color::Cyan),
            TokenType::Boolean => Style::default().fg(Color::Yellow),
            TokenType::Null => Style::default().fg(Color::Gray),
            TokenType::Key => Style::default().fg(Color::Blue),
            TokenType::Bracket => Style::default().fg(Color::White),
            TokenType::Punctuation => Style::default().fg(Color::Gray),
            TokenType::Default => Style::default(),
        }
    }

    /// 入力文字列をハイライトされたSpanのベクタに変換
    pub fn highlight<'a>(&self, input: &'a str) -> Vec<Span<'a>> {
        let tokens = self.tokenize(input);
        let mut spans = Vec::new();
        let mut last_end = 0;

        for token in tokens {
            // 前のトークンとの間の空白を追加
            if token.start > last_end {
                let whitespace = &input[last_end..token.start];
                if !whitespace.is_empty() {
                    spans.push(Span::raw(whitespace));
                }
            }

            // トークンのスパンを追加
            let style = self.get_style(&token.token_type);
            spans.push(Span::styled(token.text, style));

            last_end = token.end;
        }

        // 最後のトークン以降の文字列を追加
        if last_end < input.len() {
            let remaining = &input[last_end..];
            spans.push(Span::raw(remaining));
        }

        spans
    }

    /// ハイライトされた行を作成
    pub fn highlight_line<'a>(&self, input: &'a str) -> Line<'a> {
        Line::from(self.highlight(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_json() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize(r#"{"name": "John"}");

        assert!(!tokens.is_empty());

        // 左ブラケット、キー、コロン、文字列値、右ブラケットが含まれることを確認
        let brackets: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Bracket)
            .collect();
        assert_eq!(brackets.len(), 2); // { }

        let keys: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Key)
            .collect();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].text, r#""name"");

        let strings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::String)
            .collect();
        assert_eq!(strings.len(), 1);
        assert_eq!(strings[0].text, r#""John"");
    }

    #[test]
    fn test_tokenize_json_array() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize(r#"[1, 2, 3]");

        let brackets: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Bracket)
            .collect();
        assert_eq!(brackets.len(), 2); // [ ]

        let numbers: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Number)
            .collect();
        assert_eq!(numbers.len(), 3);

        let punctuation: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Punctuation)
            .collect();
        assert_eq!(punctuation.len(), 2); // , ,
    }

    #[test]
    fn test_tokenize_json_primitives() {
        let highlighter = SyntaxHighlighter::new();

        // Boolean値のテスト
        let tokens = highlighter.tokenize("true");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Boolean);

        let tokens = highlighter.tokenize("false");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Boolean);

        // null値のテスト
        let tokens = highlighter.tokenize("null");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Null);
    }

    #[test]
    fn test_tokenize_json_numbers() {
        let highlighter = SyntaxHighlighter::new();

        // 整数
        let tokens = highlighter.tokenize("123");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].text, "123");

        // 小数
        let tokens = highlighter.tokenize("123.45");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].text, "123.45");

        // 負数
        let tokens = highlighter.tokenize("-123");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].text, "-123");
    }

    #[test]
    fn test_distinguish_key_and_string() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize(r#"{"key": "value"}");

        let keys: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Key)
            .collect();
        let strings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::String)
            .collect();

        assert_eq!(keys.len(), 1);
        assert_eq!(strings.len(), 1);
        assert_eq!(keys[0].text, r#""key"");
        assert_eq!(strings[0].text, r#""value"");
    }

    #[test]
    fn test_complex_json() {
        let highlighter = SyntaxHighlighter::new();
        let json = r#"{"users": [{"name": "John", "age": 30, "active": true}, {"name": "Jane", "age": null}]}");
        let tokens = highlighter.tokenize(json);

        // トークンが生成されることを確認
        assert!(!tokens.is_empty());

        // 各種トークンタイプが含まれることを確認
        let has_key = tokens.iter().any(|t| t.token_type == TokenType::Key);
        let has_string = tokens.iter().any(|t| t.token_type == TokenType::String);
        let has_number = tokens.iter().any(|t| t.token_type == TokenType::Number);
        let has_boolean = tokens.iter().any(|t| t.token_type == TokenType::Boolean);
        let has_null = tokens.iter().any(|t| t.token_type == TokenType::Null);
        let has_bracket = tokens.iter().any(|t| t.token_type == TokenType::Bracket);
        let has_punctuation = tokens
            .iter()
            .any(|t| t.token_type == TokenType::Punctuation);

        assert!(has_key);
        assert!(has_string);
        assert!(has_number);
        assert!(has_boolean);
        assert!(has_null);
        assert!(has_bracket);
        assert!(has_punctuation);
    }

    #[test]
    fn test_highlight_json() {
        let highlighter = SyntaxHighlighter::new();
        let spans = highlighter.highlight(r#"{"name": "John", "age": 30}");

        // スパンが作成されることを確認
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_highlight_line() {
        let highlighter = SyntaxHighlighter::new();
        let line = highlighter.highlight_line(r#"{"test": true}");

        // ラインが作成されることを確認
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_classify_token() {
        let highlighter = SyntaxHighlighter::new();

        assert_eq!(highlighter.classify_token("true"), TokenType::Boolean);
        assert_eq!(highlighter.classify_token("false"), TokenType::Boolean);
        assert_eq!(highlighter.classify_token("null"), TokenType::Null);
        assert_eq!(highlighter.classify_token("other"), TokenType::Default);
    }
}
