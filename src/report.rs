use std::borrow::Cow;

type CowString<'a> = Cow<'a, str>;
pub type Report<'a> = Vec<(CowString<'a>, bool)>;

pub fn report<'a, M: Into<CowString<'a>>>(report: &mut Report<'a>, result: Option<(M, bool)>) {
    if let Some((key, success)) = result {
        let key = key.into();

        debug_assert!(!report.iter().any(|(k, _)| k == &key), "{} already reported", key);
        report.push((key, success));
    }
}
