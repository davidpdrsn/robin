pub struct JobName(String);

impl ToString for JobName {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl<T> From<T> for JobName
where
    T: Into<String>,
{
    fn from(s: T) -> JobName {
        JobName(s.into())
    }
}
