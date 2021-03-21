//!
//! The compiler lexer.
//!

use regex::Regex;

///
/// Provides vector of tokens for a given source.
///
pub fn get_lexemes(src: &mut String) -> Vec<String> {
    let mut result = Vec::new();
    let mut index = 0;
    // TODO: Check if we can rely on regexp to guarantee that in case of ':=' it will always be
    // ':=' rather than [':', '='].
    let delimeters = Regex::new(r"(:=)|(\s+)|[{}(),:]|(/\*)|(\*/)").expect("invalid regex");
    let mut matched = delimeters.find(&src[index..]);
    while matched != None {
        let the_match = matched.unwrap();
        if the_match.start() != 0 {
            result.push(String::from(&src[index..index + the_match.start()]));
        }
        result.push(String::from(the_match.as_str()));
        index += the_match.end();
        matched = delimeters.find(&src[index..]);
    }
    if index < src.len() {
        result.push(String::from(&src[index..]));
    }
    result
        .into_iter()
        .filter(|x| !Regex::new(r"^\s+$").unwrap().is_match(x))
        .collect()
}

///
/// Removes comments from the given source code.
///
pub fn remove_comments(src: &mut String) {
    let mut comment = src.find("//");
    while comment != None {
        let pos = comment.unwrap();
        let eol = src[pos..].find('\n').unwrap_or(src.len() - pos) + pos;
        src.replace_range(pos..eol, "");
        comment = src.find("//");
    }
}
