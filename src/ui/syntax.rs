use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// jaqクエリのトークンタイプ
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,     // select, map, if, then, else, end, and, or, not
    Operator,    // |, =, ==, !=, <, >, <=, >=, +, -, *, /, %
    Function,    // length, keys, values, type, empty, error
    String,      // "文字列"
    Number,      // 123, 3.14
    Boolean,     // true, false
    Null,        // null
    Identifier,  // .foo, .bar, 変数名
    Bracket,     // [], {}
    Parenthesis, // ()
    Punctuation, // , ; :
    Pipe,        // |
    Comment,     // #コメント
    Error,       // 構文エラー
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

/// シンタックスハイライター
pub struct SyntaxHighlighter {
    keywords: Vec<&'static str>,
    functions: Vec<&'static str>,
    operators: Vec<&'static str>,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "if",
                "then",
                "else",
                "elif",
                "end",
                "and",
                "or",
                "not",
                "try",
                "catch",
                "def",
                "as",
                "reduce",
                "foreach",
                "while",
                "until",
                "import",
                "include",
                "module",
                "empty",
                "error",
                "select",
                "map",
                "has",
                "in",
                "contains",
                "inside",
                "startswith",
                "endswith",
                "ltrimstr",
                "rtrimstr",
                "split",
                "join",
                "reverse",
                "sort",
                "sort_by",
                "group_by",
                "unique",
                "unique_by",
                "min",
                "max",
                "min_by",
                "max_by",
                "add",
                "any",
                "all",
                "flatten",
                "from_entries",
                "to_entries",
                "with_entries",
                "paths",
                "leaf_paths",
            ],
            functions: vec![
                "length",
                "keys",
                "keys_unsorted",
                "values",
                "type",
                "tonumber",
                "tostring",
                "todate",
                "todateiso8601",
                "fromdateiso8601",
                "now",
                "strftime",
                "strptime",
                "mktime",
                "gmtime",
                "floor",
                "ceil",
                "round",
                "sqrt",
                "log",
                "log10",
                "log2",
                "exp",
                "exp10",
                "exp2",
                "sin",
                "cos",
                "tan",
                "asin",
                "acos",
                "atan",
                "atan2",
                "sinh",
                "cosh",
                "tanh",
                "asinh",
                "acosh",
                "atanh",
                "abs",
                "fabs",
                "ascii_downcase",
                "ascii_upcase",
                "recurse",
                "recurse_down",
                "walk",
                "transpose",
                "combinations",
                "range",
                "repeat",
                "until",
                "while",
                "limit",
                "first",
                "last",
                "nth",
                "indices",
                "index",
                "rindex",
                "debug",
                "input",
                "inputs",
                "format",
                "test",
                "match",
                "capture",
                "splits",
                "sub",
                "gsub",
                "ascii_downcase",
                "ascii_upcase",
            ],
            operators: vec![
                "==", "!=", "<=", ">=", "<", ">", "=", "+=", "-=", "*=", "/=", "%=", "//=", "+",
                "-", "*", "/", "%", "//", "?", ":", ";", ",", ".", "..", "?//", "//", "and", "or",
                "not", "|", "[]", "{}", "()",
            ],
        }
    }

    /// jaqクエリをトークンに分解
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

                    tokens.push(Token {
                        token_type: TokenType::String,
                        text,
                        start,
                        end,
                    });
                }

                // 数値
                '0'..='9' => {
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

                    tokens.push(Token {
                        token_type: TokenType::Number,
                        text,
                        start,
                        end,
                    });
                }

                // コメント
                '#' => {
                    let mut end = start;
                    let mut text = String::new();
                    text.push(ch);
                    end += ch.len_utf8();

                    // 行末まで読む
                    for (pos, next_ch) in chars.by_ref() {
                        if next_ch == '\n' || next_ch == '\r' {
                            break;
                        }
                        text.push(next_ch);
                        end = pos + next_ch.len_utf8();
                    }

                    tokens.push(Token {
                        token_type: TokenType::Comment,
                        text,
                        start,
                        end,
                    });
                }

                // パイプ
                '|' => {
                    tokens.push(Token {
                        token_type: TokenType::Pipe,
                        text: "|".to_string(),
                        start,
                        end: start + 1,
                    });
                }

                // ブラケット
                '[' | ']' => {
                    tokens.push(Token {
                        token_type: TokenType::Bracket,
                        text: ch.to_string(),
                        start,
                        end: start + ch.len_utf8(),
                    });
                }
                '{' | '}' => {
                    tokens.push(Token {
                        token_type: TokenType::Bracket,
                        text: ch.to_string(),
                        start,
                        end: start + ch.len_utf8(),
                    });
                }

                // 括弧
                '(' | ')' => {
                    tokens.push(Token {
                        token_type: TokenType::Parenthesis,
                        text: ch.to_string(),
                        start,
                        end: start + ch.len_utf8(),
                    });
                }

                // オペレータと識別子
                _ => {
                    let mut end = start;
                    let mut text = String::new();
                    text.push(ch);
                    end += ch.len_utf8();

                    // 連続する英数字やアンダースコア、ドットを読む
                    while let Some(&(pos, next_ch)) = chars.peek() {
                        if next_ch.is_alphanumeric()
                            || next_ch == '_'
                            || next_ch == '.'
                            || (text.starts_with('.') && next_ch.is_alphabetic())
                            || (self.is_operator_char(next_ch) && self.is_operator_char(ch))
                        {
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

    /// 文字がオペレータの一部かどうか判定
    fn is_operator_char(&self, ch: char) -> bool {
        matches!(
            ch,
            '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '?' | ':' | ';' | ',' | '.'
        )
    }

    /// トークンの種類を分類
    fn classify_token(&self, text: &str) -> TokenType {
        match text {
            "true" | "false" => TokenType::Boolean,
            "null" => TokenType::Null,
            "," | ";" | ":" => TokenType::Punctuation,
            _ if self.keywords.contains(&text) => TokenType::Keyword,
            _ if self.functions.contains(&text) => TokenType::Function,
            _ if self.operators.contains(&text) => TokenType::Operator,
            _ if text.starts_with('.') => TokenType::Identifier,
            _ if text.chars().all(|c| c.is_alphanumeric() || c == '_') => TokenType::Identifier,
            _ => TokenType::Default,
        }
    }

    /// トークンタイプに応じたスタイルを取得
    pub fn get_style(&self, token_type: &TokenType) -> Style {
        match token_type {
            TokenType::Keyword => Style::default().fg(Color::Blue),
            TokenType::Operator => Style::default().fg(Color::Red),
            TokenType::Function => Style::default().fg(Color::Green),
            TokenType::String => Style::default().fg(Color::Yellow),
            TokenType::Number => Style::default().fg(Color::Cyan),
            TokenType::Boolean => Style::default().fg(Color::Magenta),
            TokenType::Null => Style::default().fg(Color::Gray),
            TokenType::Identifier => Style::default().fg(Color::White),
            TokenType::Bracket => Style::default().fg(Color::LightBlue),
            TokenType::Parenthesis => Style::default().fg(Color::LightBlue),
            TokenType::Punctuation => Style::default().fg(Color::Gray),
            TokenType::Pipe => Style::default().fg(Color::LightRed),
            TokenType::Comment => Style::default().fg(Color::DarkGray),
            TokenType::Error => Style::default().fg(Color::Red),
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
    fn test_tokenize_simple_query() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize(".name");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].text, ".name");
    }

    #[test]
    fn test_tokenize_complex_query() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize(".users[] | select(.age > 25)");

        // トークンの存在確認
        assert!(!tokens.is_empty());

        // パイプトークンの確認
        let pipe_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Pipe)
            .collect();
        assert_eq!(pipe_tokens.len(), 1);
    }

    #[test]
    fn test_tokenize_string() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize(r#"select(.name == "John")"#);

        let string_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::String)
            .collect();
        assert_eq!(string_tokens.len(), 1);
        assert_eq!(string_tokens[0].text, r#""John""#);
    }

    #[test]
    fn test_tokenize_number() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize("select(.age > 25)");

        let number_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Number)
            .collect();
        assert_eq!(number_tokens.len(), 1);
        assert_eq!(number_tokens[0].text, "25");
    }

    #[test]
    fn test_tokenize_keywords() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize("if .age > 18 then .name else empty end");

        let keyword_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Keyword)
            .collect();
        assert!(keyword_tokens.len() >= 4); // if, then, else, empty, end
    }

    #[test]
    fn test_tokenize_functions() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize("length | keys | type");

        let function_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Function)
            .collect();
        assert_eq!(function_tokens.len(), 3);
    }

    #[test]
    fn test_tokenize_comments() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.tokenize(".name # get user name");

        let comment_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Comment)
            .collect();
        assert_eq!(comment_tokens.len(), 1);
        assert!(comment_tokens[0].text.starts_with('#'));
    }

    #[test]
    fn test_highlight() {
        let highlighter = SyntaxHighlighter::new();
        let spans = highlighter.highlight(".name | length");

        // スパンが作成されることを確認
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_classify_token() {
        let highlighter = SyntaxHighlighter::new();

        assert_eq!(highlighter.classify_token("select"), TokenType::Keyword);
        assert_eq!(highlighter.classify_token("length"), TokenType::Function);
        assert_eq!(highlighter.classify_token("true"), TokenType::Boolean);
        assert_eq!(highlighter.classify_token("null"), TokenType::Null);
        assert_eq!(highlighter.classify_token(".name"), TokenType::Identifier);
        assert_eq!(highlighter.classify_token(","), TokenType::Punctuation);

        // オペレータとして分類されるべきテスト
        assert_eq!(highlighter.classify_token("=="), TokenType::Operator);
        assert_eq!(highlighter.classify_token("+"), TokenType::Operator);
    }
}
