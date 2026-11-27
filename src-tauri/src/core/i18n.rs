//! 本地化支持。
//!
//! Swift 版通过 `NSLocalizedString(key)` 运行时查找 `*.lproj/Localizable.strings`。
//! Rust 版在启动时用同样的 `.strings` 文件建立查表，缺失 key 时回退到 key 本身，
//! 行为与 Swift 的 `NSLocalizedString` 一致。

use std::collections::HashMap;

/// 简体中文文案（直接复用 Swift 版的 .strings 文件）。
const ZH_HANS: &str = include_str!("../../../locales/zh-Hans.lproj/Localizable.strings");
/// 英文文案（直接复用 Swift 版的 .strings 文件）。
const EN: &str = include_str!("../../../locales/en.lproj/Localizable.strings");

/// 支持的语言。
///
/// 规范写法是 kebab-case（`zh-hans` / `en`），但历史上出现过三种写法：
/// - `zh-Hans`：Swift 版 `@AppStorage` 用的 BCP-47 风格；
/// - `ZhHans`：egui 版 Rust 枚举的默认变体名；
/// - `zh-hans`：当前规范写法。
///
/// 三者都要能反序列化 —— `Settings::load()` 解析失败会**整份配置退回默认值**，
/// 旧用户换个版本就丢掉全部设置；`update_settings` 解析失败则会让整个设置补丁
/// 被丢弃，语言切回中文直接失效。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Locale {
    /// 简体中文
    #[default]
    #[serde(alias = "zh-Hans", alias = "ZhHans")]
    ZhHans,
    /// 英文
    #[serde(alias = "En")]
    En,
}

impl Locale {
    /// 根据系统语言环境变量推断语言，未识别时回退到简体中文。
    pub fn detect() -> Self {
        let raw = std::env::var("LANG")
            .ok()
            .or_else(|| std::env::var("LC_ALL").ok())
            .unwrap_or_default()
            .to_lowercase();
        if raw.starts_with("zh") {
            Locale::ZhHans
        } else if raw.starts_with("en") {
            Locale::En
        } else {
            Locale::default()
        }
    }

    /// 语言标识，用于界面展示与持久化。
    pub fn code(self) -> &'static str {
        match self {
            Locale::ZhHans => "zh-Hans",
            Locale::En => "en",
        }
    }
}

/// 文案表。
pub struct I18n {
    locale: Locale,
    table: HashMap<String, String>,
}

impl I18n {
    /// 按语言加载文案表。
    pub fn new(locale: Locale) -> Self {
        let source = match locale {
            Locale::ZhHans => ZH_HANS,
            Locale::En => EN,
        };
        Self {
            locale,
            table: parse_strings(source),
        }
    }

    /// 当前语言。
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// 切换语言并重新加载文案。
    pub fn set_locale(&mut self, locale: Locale) {
        *self = I18n::new(locale);
    }

    /// 取文案，缺失时返回 key 本身。
    pub fn t(&self, key: &str) -> String {
        self.table
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    /// 取文案并按顺序填充 `%@` 占位符。
    ///
    /// Swift 版使用 `String(format:)` 配合 `%@`；这里等价地按出现顺序替换。
    pub fn tf(&self, key: &str, args: &[&str]) -> String {
        format_placeholders(&self.t(key), args)
    }

    /// 导出整张文案表（供前端 `get_strings` 使用）。
    pub fn table(&self) -> HashMap<String, String> {
        self.table.clone()
    }
}

/// 依次把模板中的 `%@` 替换为给定参数。
pub fn format_placeholders(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    let mut index = 0usize;

    while let Some(pos) = rest.find("%@") {
        out.push_str(&rest[..pos]);
        match args.get(index) {
            Some(value) => out.push_str(value),
            None => out.push_str("%@"),
        }
        index += 1;
        rest = &rest[pos + 2..];
    }
    out.push_str(rest);
    out
}

/// 解析 Apple `.strings` 文件。
///
/// 文件语法为 `"key" = "value";`，允许 `//` 行注释与 `/* */` 块注释。
/// 重复 key 以后出现的为准（与 Foundation 的行为一致）。
fn parse_strings(source: &str) -> HashMap<String, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut table = HashMap::new();
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i] != '"' {
            i += 1;
            continue;
        }

        let Some((key, next)) = read_quoted(&chars, i) else {
            break;
        };

        let mut j = next;
        while j < chars.len() && chars[j].is_whitespace() {
            j += 1;
        }
        if j >= chars.len() || chars[j] != '=' {
            i = next;
            continue;
        }
        j += 1;
        while j < chars.len() && chars[j].is_whitespace() {
            j += 1;
        }
        if j >= chars.len() || chars[j] != '"' {
            i = next;
            continue;
        }

        let Some((value, after)) = read_quoted(&chars, j) else {
            break;
        };
        table.insert(key, value);
        i = after;
    }

    table
}

/// 从 `start`（必须是起始引号）读取一个带转义的字符串，返回内容与结束位置。
fn read_quoted(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut i = start + 1;

    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            let escaped = chars[i + 1];
            value.push(match escaped {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                other => other,
            });
            i += 2;
            continue;
        }
        if c == '"' {
            return Some((value, i + 1));
        }
        value.push(c);
        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 三种历史写法都必须能反序列化。
    ///
    /// 写错的后果不只是这一个字段失效：
    /// - `Settings::load()` 解析失败 → **整份配置退回默认值**（旧用户设置全丢）；
    /// - `update_settings` 解析失败 → **整个设置补丁被丢弃**，切语言直接不生效，
    ///   而且前端 `void patch(...)` 把异常吞掉，界面毫无反馈。
    #[test]
    fn accepts_every_historical_locale_spelling() {
        for raw in ["zh-hans", "zh-Hans", "ZhHans"] {
            let parsed: Locale = serde_json::from_str(&format!("\"{raw}\""))
                .unwrap_or_else(|err| panic!("`{raw}` 应可反序列化: {err}"));
            assert_eq!(parsed, Locale::ZhHans);
        }

        for raw in ["en", "En"] {
            let parsed: Locale = serde_json::from_str(&format!("\"{raw}\""))
                .unwrap_or_else(|err| panic!("`{raw}` 应可反序列化: {err}"));
            assert_eq!(parsed, Locale::En);
        }
    }

    /// 规范写法必须保持不变：前端按这个值比对选中态，落盘也按这个值写。
    #[test]
    fn serializes_to_the_canonical_kebab_case_form() {
        assert_eq!(
            serde_json::to_string(&Locale::ZhHans).unwrap(),
            "\"zh-hans\""
        );
        assert_eq!(serde_json::to_string(&Locale::En).unwrap(), "\"en\"");
    }

    #[test]
    fn parses_keys_and_values() {
        let table = parse_strings("\"a.b\" = \"hello\";\n// comment\n\"a.c\" = \"world\";");
        assert_eq!(table.get("a.b").map(String::as_str), Some("hello"));
        assert_eq!(table.get("a.c").map(String::as_str), Some("world"));
    }

    #[test]
    fn duplicate_key_keeps_last() {
        let table = parse_strings("\"k\" = \"first\";\n\"k\" = \"second\";");
        assert_eq!(table.get("k").map(String::as_str), Some("second"));
    }

    #[test]
    fn fills_placeholders_in_order() {
        assert_eq!(
            format_placeholders("kill %@ (%@)?", &["Finder", "42"]),
            "kill Finder (42)?"
        );
    }

    #[test]
    fn missing_placeholder_is_kept() {
        assert_eq!(format_placeholders("kill %@", &[]), "kill %@");
    }

    #[test]
    fn shipped_locales_contain_tab_titles() {
        for locale in [Locale::ZhHans, Locale::En] {
            let i18n = I18n::new(locale);
            assert!(!i18n.t("tab.process").is_empty());
            assert!(!i18n.t("settings.about").is_empty());
        }
    }
}
