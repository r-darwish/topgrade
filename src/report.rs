use std::borrow::Cow;
use std::collections::HashMap;
use std::process::ExitStatus;

pub type Report = HashMap<String, bool>;

pub trait Reporter {
    fn report<'a, M: Into<Cow<'a, str>>>(&self, key: M, report: &mut Report);
}

impl Reporter for ExitStatus {
    fn report<'a, M: Into<Cow<'a, str>>>(&self, key: M, report: &mut Report) {
        report.insert(key.into().into_owned(), self.success());
    }
}

impl Reporter for bool {
    fn report<'a, M: Into<Cow<'a, str>>>(&self, key: M, report: &mut Report) {
        report.insert(key.into().into_owned(), *self);
    }
}
