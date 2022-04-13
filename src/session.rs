use std::fmt::{Display, Formatter, Result};

#[non_exhaustive]
pub struct Session_<T: Display + PartialEq> {
    pub id: T,
}

impl<T> Display for Session_<T>
where
    T: Display + PartialEq,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.id)
    }
}

impl<T> PartialEq for Session_<T>
where
    T: Display + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Session_<T>
where
    T: Display + PartialEq,
{
    pub fn new(id: T) -> Self {
        Session_ { id }
    }
}
