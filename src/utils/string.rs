use std::str::FromStr;

#[inline]
pub fn parse_key_value(raw_str: &str) -> (&str, &str) {
    let colon_pos = raw_str.find(':').unwrap();
    (raw_str[..colon_pos].trim(), raw_str[colon_pos + 1..].trim())
}

#[inline(always)]
pub fn trim_split_iter<'a, I>(split_iter: I, remove_whitespace: bool) -> Vec<&'a str>
where
    I: Iterator<Item = &'a str>,
{
    if remove_whitespace {
        split_iter.map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
    } else {
        split_iter.map(|s| s.trim()).collect()
    }
}

pub fn remove_comments(string: &str, comment_begin: &str) -> String {
    let mut result = String::with_capacity(string.len());
    for line in string.lines() {
        let (content, _) = line.split_once(comment_begin).unwrap_or((line, ""));
        if content.chars().any(|c| !c.is_whitespace()) {
            result.push_str(content);
            result.push('\n');
        }
    }
    result.pop();
    result
}

#[inline]
pub fn add_key_value_template(template: &mut String, key: &str, sep: &str, value: &str, end: &str) {
    template.reserve(key.len() + value.len() + 3);
    template.push_str(key);
    template.push_str(sep);
    template.push_str(value);
    template.push_str(end);
}

pub trait StrDefaultExtension {
    fn or_default_empty(&self, default: &str) -> String;
}

impl StrDefaultExtension for str {
    fn or_default_empty(&self, default: &str) -> String {
        let trimmed = self.trim();
        if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed.to_string()
        }
    }
}

pub trait StrNumericDefaultExtension {
    fn or_default_empty_as<D: Default + ToString + FromStr>(&self, default: D) -> D;
}

impl StrNumericDefaultExtension for str {
    fn or_default_empty_as<D: Default + ToString + FromStr>(&self, default: D) -> D {
        let s = self.trim();
        if s.is_empty() {
            return default;
        }

        s.parse().unwrap_or_else(|_| {
            default.to_string().parse().unwrap_or_else(|_| {
                panic!(
                    "Failed to parse '{}' or default '{}' as requested numeric type",
                    s,
                    default.to_string()
                )
            })
        })
    }
}