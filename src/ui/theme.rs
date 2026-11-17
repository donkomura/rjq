use super::syntax::TokenType;
use ratatui::style::{Color, Modifier, Style};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use syntect::highlighting::{FontStyle, Highlighter, ThemeSet};
use syntect::parsing::ScopeStack;
use thiserror::Error;

pub const DEFAULT_THEME: &str = "ansi";
const THEME_EXTENSION: &str = "tmTheme";

const BUILTIN_THEMES: [(&str, &str); 7] = [
    ("eighties-dark", "base16-eighties.dark"),
    ("github", "InspiredGitHub"),
    ("mocha-dark", "base16-mocha.dark"),
    ("ocean-dark", "base16-ocean.dark"),
    ("ocean-light", "base16-ocean.light"),
    ("solarized-dark", "Solarized (dark)"),
    ("solarized-light", "Solarized (light)"),
];

const KEY_SCOPES: &str =
    "source.json meta.mapping.key.json string.quoted.double.json support.type.property-name.json";
const STRING_SCOPES: &str = "source.json meta.mapping.value.json string.quoted.double.json";
const NUMBER_SCOPES: &str = "source.json constant.numeric.json";
const BOOLEAN_SCOPES: &str = "source.json constant.language.boolean.json";
const NULL_SCOPES: &str = "source.json constant.language.null.json";
const BRACKET_SCOPES: &str = "source.json punctuation.section.mapping.json";
const PUNCTUATION_SCOPES: &str = "source.json punctuation.separator.json";

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("failed to load theme file {path}: {source}")]
    Load {
        path: PathBuf,
        source: syntect::LoadingError,
    },
    #[error("failed to read theme directory {path}: {source}")]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("theme not found: {0} (run `rjq --list-themes` to see available themes)")]
    NotFound(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    string: Style,
    number: Style,
    boolean: Style,
    null: Style,
    key: Style,
    bracket: Style,
    punctuation: Style,
    default: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::ansi()
    }
}

impl Theme {
    pub fn ansi() -> Self {
        Self {
            string: Style::default().fg(Color::Green),
            number: Style::default().fg(Color::Cyan),
            boolean: Style::default().fg(Color::Yellow),
            null: Style::default().fg(Color::Gray),
            key: Style::default().fg(Color::Blue),
            bracket: Style::default().fg(Color::White),
            punctuation: Style::default().fg(Color::Gray),
            default: Style::default(),
        }
    }

    pub fn from_textmate(theme: &syntect::highlighting::Theme) -> Self {
        let highlighter = Highlighter::new(theme);
        let style_for = |scopes: &str| {
            let stack = ScopeStack::from_str(scopes).expect("scope literals are valid");
            let modifier = highlighter.style_mod_for_stack(stack.as_slice());
            to_style(
                modifier.foreground.or(theme.settings.foreground),
                modifier.font_style.unwrap_or_else(FontStyle::empty),
            )
        };

        Self {
            string: style_for(STRING_SCOPES),
            number: style_for(NUMBER_SCOPES),
            boolean: style_for(BOOLEAN_SCOPES),
            null: style_for(NULL_SCOPES),
            key: style_for(KEY_SCOPES),
            bracket: style_for(BRACKET_SCOPES),
            punctuation: style_for(PUNCTUATION_SCOPES),
            default: to_style(theme.settings.foreground, FontStyle::empty()),
        }
    }

    pub fn style(&self, token_type: &TokenType) -> Style {
        match token_type {
            TokenType::String => self.string,
            TokenType::Number => self.number,
            TokenType::Boolean => self.boolean,
            TokenType::Null => self.null,
            TokenType::Key => self.key,
            TokenType::Bracket => self.bracket,
            TokenType::Punctuation => self.punctuation,
            TokenType::Default => self.default,
        }
    }
}

fn to_style(foreground: Option<syntect::highlighting::Color>, font_style: FontStyle) -> Style {
    let mut style = Style::default();
    if let Some(color) = foreground {
        style = style.fg(Color::Rgb(color.r, color.g, color.b));
    }
    for (flag, modifier) in [
        (FontStyle::BOLD, Modifier::BOLD),
        (FontStyle::ITALIC, Modifier::ITALIC),
        (FontStyle::UNDERLINE, Modifier::UNDERLINED),
    ] {
        if font_style.contains(flag) {
            style = style.add_modifier(modifier);
        }
    }
    style
}

pub struct ThemeCatalog {
    builtin: Vec<(&'static str, syntect::highlighting::Theme)>,
    themes_dir: Option<PathBuf>,
}

impl ThemeCatalog {
    pub fn new(themes_dir: Option<PathBuf>) -> Self {
        let mut bundled = ThemeSet::load_defaults().themes;
        let builtin = BUILTIN_THEMES
            .iter()
            .map(|(name, bundled_name)| {
                let theme = bundled
                    .remove(*bundled_name)
                    .expect("BUILTIN_THEMES refers only to themes bundled with syntect");
                (*name, theme)
            })
            .collect();
        Self {
            builtin,
            themes_dir,
        }
    }

    pub fn resolve(&self, spec: &str) -> Result<Theme, ThemeError> {
        if spec.chars().any(std::path::is_separator) {
            return Err(ThemeError::NotFound(spec.to_string()));
        }
        if spec.eq_ignore_ascii_case(DEFAULT_THEME) {
            return Ok(Theme::ansi());
        }
        if let Some(theme) = self
            .builtin
            .iter()
            .find_map(|(name, theme)| name.eq_ignore_ascii_case(spec).then_some(theme))
        {
            return Ok(Theme::from_textmate(theme));
        }
        if let Some(dir) = &self.themes_dir {
            let candidate = dir.join(format!("{spec}.{THEME_EXTENSION}"));
            if candidate.is_file() {
                return load_theme_file(&candidate);
            }
        }
        Err(ThemeError::NotFound(spec.to_string()))
    }

    pub fn names(&self) -> Result<Vec<String>, ThemeError> {
        let mut names = vec![DEFAULT_THEME.to_string()];
        names.extend(self.builtin.iter().map(|(name, _)| name.to_string()));
        if let Some(dir) = &self.themes_dir {
            names.extend(theme_names_in(dir)?);
        }
        Ok(names)
    }
}

pub fn default_themes_dir() -> Option<PathBuf> {
    themes_dir_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

fn themes_dir_from(xdg_config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let non_empty = |value: Option<OsString>| value.filter(|v| !v.is_empty()).map(PathBuf::from);
    non_empty(xdg_config_home)
        .or_else(|| non_empty(home).map(|home| home.join(".config")))
        .map(|config| config.join("rjq").join("themes"))
}

fn load_theme_file(path: &Path) -> Result<Theme, ThemeError> {
    let theme = ThemeSet::get_theme(path).map_err(|source| ThemeError::Load {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(Theme::from_textmate(&theme))
}

fn theme_names_in(dir: &Path) -> Result<Vec<String>, ThemeError> {
    let read_dir_error = |source| ThemeError::ReadDir {
        path: dir.to_path_buf(),
        source,
    };
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(read_dir_error(e)),
    };

    let mut names = Vec::new();
    for entry in entries {
        let path = entry.map_err(read_dir_error)?.path();
        if path.extension().is_some_and(|ext| ext == THEME_EXTENSION)
            && let Some(stem) = path.file_stem()
        {
            names.push(stem.to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const FIXTURE_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>name</key>
  <string>Fixture</string>
  <key>settings</key>
  <array>
    <dict>
      <key>settings</key>
      <dict>
        <key>foreground</key>
        <string>#010101</string>
        <key>background</key>
        <string>#000000</string>
      </dict>
    </dict>
    RULES
  </array>
</dict>
</plist>
"#;

    fn rule(scope: &str, foreground: &str, font_style: &str) -> String {
        format!(
            "<dict><key>scope</key><string>{scope}</string><key>settings</key><dict>\
             <key>foreground</key><string>{foreground}</string>\
             <key>fontStyle</key><string>{font_style}</string></dict></dict>"
        )
    }

    fn fixture(rules: &[String]) -> String {
        FIXTURE_TEMPLATE.replace("RULES", &rules.concat())
    }

    fn parse_fixture(rules: &[String]) -> syntect::highlighting::Theme {
        ThemeSet::load_from_reader(&mut std::io::Cursor::new(fixture(rules))).unwrap()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rjq_theme_{}_{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fg(hex: u32) -> Style {
        Style::default().fg(Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8))
    }

    #[test]
    fn test_ansi_theme_keeps_original_colors() {
        let theme = Theme::ansi();

        let tests = vec![
            (TokenType::String, Style::default().fg(Color::Green)),
            (TokenType::Number, Style::default().fg(Color::Cyan)),
            (TokenType::Boolean, Style::default().fg(Color::Yellow)),
            (TokenType::Null, Style::default().fg(Color::Gray)),
            (TokenType::Key, Style::default().fg(Color::Blue)),
            (TokenType::Bracket, Style::default().fg(Color::White)),
            (TokenType::Punctuation, Style::default().fg(Color::Gray)),
            (TokenType::Default, Style::default()),
        ];

        for (token_type, expected) in tests {
            assert_eq!(expected, theme.style(&token_type), "{token_type:?}");
        }
    }

    #[test]
    fn test_from_textmate_maps_json_scopes_to_tokens() {
        let theme = Theme::from_textmate(&parse_fixture(&[
            rule("string", "#110000", ""),
            rule("constant.numeric", "#220000", ""),
            rule("constant.language.boolean", "#330000", ""),
            rule("constant.language.null", "#440000", ""),
            rule("support.type.property-name", "#550000", "bold"),
            rule("punctuation.section", "#660000", ""),
            rule("punctuation.separator", "#770000", "italic underline"),
        ]));

        let tests = vec![
            (TokenType::String, fg(0x110000)),
            (TokenType::Number, fg(0x220000)),
            (TokenType::Boolean, fg(0x330000)),
            (TokenType::Null, fg(0x440000)),
            (TokenType::Key, fg(0x550000).add_modifier(Modifier::BOLD)),
            (TokenType::Bracket, fg(0x660000)),
            (
                TokenType::Punctuation,
                fg(0x770000).add_modifier(Modifier::ITALIC | Modifier::UNDERLINED),
            ),
            (TokenType::Default, fg(0x010101)),
        ];

        for (token_type, expected) in tests {
            assert_eq!(expected, theme.style(&token_type), "{token_type:?}");
        }
    }

    #[test]
    fn test_from_textmate_resolves_key_in_both_grammar_conventions() {
        let tests = vec![
            (
                "VS Code style property-name scope",
                vec![
                    rule("string", "#110000", ""),
                    rule("support.type.property-name", "#220000", ""),
                ],
            ),
            (
                "Sublime Text style mapping key scope",
                vec![
                    rule("string", "#110000", ""),
                    rule("meta.mapping.key string", "#220000", ""),
                ],
            ),
        ];

        for (name, rules) in tests {
            let theme = Theme::from_textmate(&parse_fixture(&rules));

            assert_eq!(fg(0x220000), theme.style(&TokenType::Key), "{name}");
            assert_eq!(fg(0x110000), theme.style(&TokenType::String), "{name}");
        }
    }

    #[test]
    fn test_from_textmate_falls_back_to_theme_foreground_for_unstyled_tokens() {
        let theme = Theme::from_textmate(&parse_fixture(&[]));

        assert_eq!(fg(0x010101), theme.style(&TokenType::Number));
    }

    #[test]
    fn test_resolve_theme() {
        let dir = temp_dir("resolve");
        let themes_dir = dir.join("themes");
        fs::create_dir_all(&themes_dir).unwrap();
        fs::write(
            themes_dir.join("mine.tmTheme"),
            fixture(&[rule("string", "#123456", "")]),
        )
        .unwrap();
        let catalog = ThemeCatalog::new(Some(themes_dir));

        let tests = vec![
            (
                "default ansi theme",
                DEFAULT_THEME.to_string(),
                Theme::ansi().style(&TokenType::String),
            ),
            ("theme directory", "mine".to_string(), fg(0x123456)),
        ];

        for (name, spec, expected) in tests {
            let theme = catalog.resolve(&spec).unwrap();
            assert_eq!(expected, theme.style(&TokenType::String), "{name}");
        }
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_resolve_builtin_theme_by_hyphenated_name() {
        let catalog = ThemeCatalog::new(None);
        let bundled = ThemeSet::load_defaults();

        let tests = vec![
            ("eighties-dark", "base16-eighties.dark"),
            ("github", "InspiredGitHub"),
            ("mocha-dark", "base16-mocha.dark"),
            ("ocean-dark", "base16-ocean.dark"),
            ("ocean-light", "base16-ocean.light"),
            ("solarized-dark", "Solarized (dark)"),
            ("solarized-light", "Solarized (light)"),
            ("Solarized-Dark", "Solarized (dark)"),
        ];

        for (spec, bundled_name) in tests {
            let expected = Theme::from_textmate(&bundled.themes[bundled_name]);
            assert_eq!(expected, catalog.resolve(spec).unwrap(), "{spec}");
        }
    }

    #[test]
    fn test_resolve_rejects_bundled_theme_names() {
        let catalog = ThemeCatalog::new(None);

        for spec in ThemeSet::load_defaults().themes.into_keys() {
            assert!(
                matches!(catalog.resolve(&spec), Err(ThemeError::NotFound(s)) if s == spec),
                "{spec}"
            );
        }
    }

    #[test]
    fn test_resolve_rejects_file_paths() {
        let dir = temp_dir("paths");
        let themes_dir = dir.join("themes");
        fs::create_dir_all(&themes_dir).unwrap();
        let outside = dir.join("outside.tmTheme");
        fs::write(&outside, fixture(&[])).unwrap();
        let catalog = ThemeCatalog::new(Some(themes_dir));

        let tests = vec![
            (
                "absolute path to a theme file",
                outside.to_str().unwrap().to_string(),
            ),
            (
                "absolute path without extension",
                dir.join("outside").to_str().unwrap().to_string(),
            ),
            (
                "relative path out of theme directory",
                "../outside".to_string(),
            ),
        ];

        for (name, spec) in tests {
            assert!(
                matches!(catalog.resolve(&spec), Err(ThemeError::NotFound(s)) if s == spec),
                "{name}"
            );
        }
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_resolve_errors() {
        let dir = temp_dir("errors");
        let broken = dir.join("broken.tmTheme");
        fs::write(&broken, "not a plist").unwrap();
        let catalog = ThemeCatalog::new(Some(dir.clone()));

        assert!(matches!(
            catalog.resolve("no-such-theme"),
            Err(ThemeError::NotFound(name)) if name == "no-such-theme"
        ));
        assert!(matches!(
            catalog.resolve("broken"),
            Err(ThemeError::Load { path, .. }) if path == broken
        ));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_names_lists_builtin_then_theme_directory() {
        let dir = temp_dir("names");
        fs::write(dir.join("zeta.tmTheme"), fixture(&[])).unwrap();
        fs::write(dir.join("alpha.tmTheme"), fixture(&[])).unwrap();
        fs::write(dir.join("notes.txt"), "ignored").unwrap();
        let builtin: Vec<String> = [
            "ansi",
            "eighties-dark",
            "github",
            "mocha-dark",
            "ocean-dark",
            "ocean-light",
            "solarized-dark",
            "solarized-light",
        ]
        .map(String::from)
        .to_vec();

        let tests = vec![
            ("with theme directory", Some(dir.clone()), {
                let mut names = builtin.clone();
                names.extend(["alpha".to_string(), "zeta".to_string()]);
                names
            }),
            (
                "missing theme directory",
                Some(dir.join("missing")),
                builtin.clone(),
            ),
            ("no theme directory", None, builtin.clone()),
        ];

        for (name, themes_dir, expected) in tests {
            assert_eq!(
                expected,
                ThemeCatalog::new(themes_dir).names().unwrap(),
                "{name}"
            );
        }
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_themes_dir_from_environment() {
        let tests = vec![
            (
                "XDG_CONFIG_HOME takes precedence",
                Some("/xdg"),
                Some("/home/me"),
                Some(PathBuf::from("/xdg/rjq/themes")),
            ),
            (
                "empty XDG_CONFIG_HOME is treated as unset",
                Some(""),
                Some("/home/me"),
                Some(PathBuf::from("/home/me/.config/rjq/themes")),
            ),
            (
                "HOME only",
                None,
                Some("/home/me"),
                Some(PathBuf::from("/home/me/.config/rjq/themes")),
            ),
            ("neither", None, None, None),
        ];

        for (name, xdg, home, expected) in tests {
            assert_eq!(
                expected,
                themes_dir_from(xdg.map(OsString::from), home.map(OsString::from)),
                "{name}"
            );
        }
    }
}
