use std::borrow::Cow;

type CowString<'a> = Cow<'a, str>;
pub struct Report<'a> {
    data: Vec<(CowString<'a>, bool)>,
}

impl<'a> Report<'a> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push_result<M>(&mut self, result: Option<(M, bool)>)
    where
        M: Into<CowString<'a>>,
    {
        if let Some((key, success)) = result {
            let key = key.into();

            debug_assert!(!self.data.iter().any(|(k, _)| k == &key), "{} already reported", key);
            self.data.push((key, success));
        }
    }

    pub fn data(&self) -> &Vec<(CowString<'a>, bool)> {
        &self.data
    }
}
