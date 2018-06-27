use std::borrow::Cow;

type CowString<'a> = Cow<'a, str>;
pub type Report<'a> = Vec<(CowString<'a>, bool)>;

pub trait Reporter {
    fn report<'a, M: Into<CowString<'a>>>(&self, key: M, report: &mut Report<'a>);
}

impl<T, E> Reporter for Result<T, E>
where
    T: Reporter,
{
    fn report<'a, M: Into<CowString<'a>>>(&self, key: M, report: &mut Report<'a>) {
        match self {
            Err(_) => {
                report.push((key.into(), false));
            }
            Ok(item) => {
                item.report(key, report);
            }
        }
    }
}

impl<T> Reporter for Option<T>
where
    T: Reporter,
{
    fn report<'a, M: Into<CowString<'a>>>(&self, key: M, report: &mut Report<'a>) {
        if let Some(item) = self {
            item.report(key, report);
        }
    }
}

impl Reporter for bool {
    fn report<'a, M: Into<CowString<'a>>>(&self, key: M, report: &mut Report<'a>) {
        report.push((key.into(), *self));
    }
}

impl Reporter for () {
    fn report<'a, M: Into<CowString<'a>>>(&self, key: M, report: &mut Report<'a>) {
        report.push((key.into(), true));
    }
}
