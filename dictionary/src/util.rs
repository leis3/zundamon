use std::collections::HashMap;
use kanaria::utils::{AsciiUtils, WidthUtils};

// 全角文字を半角文字に変換する
pub fn to_narrow(s: &str) -> String {
    s.chars().map(|c| {
        if AsciiUtils::is_wide_ascii(c) {
            WidthUtils::convert_to_narrow(c).0
        } else {c}
    }).collect()
}

// 文字列Tが文字列集合Sに含まれる文字列を組み合わせて作ることができるか判定する
// 作ることができる場合はその文字列の組み合わせを返す
pub fn can_construct(s: &HashMap<String, String>, t: &str) -> Option<Vec<String>> {
    let mut memo = HashMap::new();
    can_construct_inner(s, t, &mut memo)
        .map(|v| v.into_iter().map(|s| s.to_owned()).collect())
}

fn can_construct_inner<'a>(s: &HashMap<String, String>, t: &'a str, memo: &mut HashMap<&'a str, Vec<&'a str>>) -> Option<Vec<&'a str>> {
    if let Some(v) = memo.get(t) {
        return Some(v.clone());
    }
    if s.contains_key(t) {
        memo.insert(t, vec![t]);
        return Some(vec![t]);
    }
    for i in 1..t.len() {
        let (prefix, suffix) = t.split_at(i);
        if s.contains_key(prefix) {
            if let Some(suffix_comb) = can_construct_inner(s, suffix, memo) {
                let mut result = vec![prefix];
                result.extend(suffix_comb);
                memo.insert(t, result.clone());
                return Some(result);
            }
        }
    }
    memo.insert(t, Vec::new());
    None
}

#[test]
fn test_can_construct() {
    let s = HashMap::from([
        ("apple".to_owned(), String::new()),
        ("banana".to_owned(), String::new()),
        ("orange".to_owned(), String::new())
    ]);
    let t = "appleorangebanana";
    assert_eq!(can_construct(&s, t), Some(vec!["apple".to_owned(), "orange".to_owned(), "banana".to_owned()]));
}
