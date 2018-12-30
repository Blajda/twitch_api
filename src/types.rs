use std::convert::AsRef;
use std::str::FromStr;
use std::cmp::{Eq, PartialEq};
use std::marker::PhantomData;

/* Used for Id's that can be interpreted as integers but aren't returned as 
 * an int by Twitch's API. (Maybe to allow a quick switch to a different representation 
 * without breaking the json schema?)
 */

pub struct User {}
pub struct Video {}
pub struct Game {}

pub type UserId<'a> = IntegerId<'a, User>;
pub type ChannelId<'a> = UserId<'a>;
pub type VideoId<'a> = IntegerId<'a, Video>;
pub type ClipId = Id;
pub type GameId<'a> = IntegerId<'a, Game>;


use std::borrow::Cow;

#[derive(Clone)]
pub struct IntegerId<'a, T> {
    pub int: u32,
    id: Cow<'a, str>,
    marker: PhantomData<T>
}

impl<T> IntegerId<'static, T> {
    pub fn new(id: u32) -> IntegerId<'static, T> {
        IntegerId { 
            id: Cow::Owned(id.to_string()),
            int: id,
            marker: PhantomData,
        }
    }
}

impl<'a, T> IntegerId<'a, T> {

    pub fn from_str(id: &'a str) 
        -> Result<IntegerId<'a, T>, std::num::ParseIntError> 
    {
        let int = id.parse::<u32>()?;
        Ok(IntegerId {
            id: Cow::Borrowed(id),
            int,
            marker: PhantomData,
        })
    }
}

impl<'a, T> AsRef<str> for IntegerId<'a, T> {
    fn as_ref(&self) -> &str {
        &self.id
    }
}

use std::convert::Into;

impl<'a, T> Into<u32> for &'a IntegerId<'a, T> {
    fn into(self) -> u32 {
        self.int
    }
}

impl<'a, T> Into<&'a str> for &'a IntegerId<'a, T> {
    fn into(self) -> &'a str {
        &self.id
    }
}

use std::fmt;
use std::fmt::{Display, Debug, Formatter};

impl<'a, T> Display for IntegerId<'a, T> { 

    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{}", &self.id)
    }
}

impl<'a, T> Debug for IntegerId<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{:?}", &self.id)
    }
}

impl<'a, 'b, T> PartialEq<IntegerId<'a, T>> for IntegerId<'b, T> {
    fn eq(&self, other: &IntegerId<T>) -> bool {
        self.int == other.int
    }
}
impl<'a, T> Eq for IntegerId<'a, T> {}

impl<'a, T> PartialEq<str> for IntegerId<'a, T> {
    fn eq(&self, other: &str) -> bool {
        self.id == *other
    }
}

impl<'a, T> PartialEq<u32> for IntegerId<'a, T> {
    fn eq(&self, other: &u32) -> bool {
        self.int == *other
    }
}

use serde::{Deserialize, Deserializer}; 
impl<'de, T> Deserialize<'de> for IntegerId<'static, T> {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let id = String::deserialize(deserializer)?;
        let int = (&id).parse::<u32>().map_err(serde::de::Error::custom)?;
        Ok(IntegerId::new(int))
    }
}


pub struct Id {
    inner: String
}

impl Id {
    pub fn new(id: &str) -> Id {
        Id {
            inner: id.to_owned(),
        }
    }
}

impl AsRef<str> for Id {

    fn as_ref(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::{UserId, VideoId};
    use std::str::FromStr;
    #[test]
    fn test_comparison() {
        let u1 = UserId::from_str("1234");
        let u2 = UserId::from_str("1234");

        assert_eq!(u1.is_ok(), true);
        assert_eq!(u2.is_ok(), true);

        let u1 = u1.unwrap();
        let u2 = u2.unwrap();

        assert_eq!(u1, u2);
        assert_eq!(u1, 1234);
        assert_eq!(u1, "1234");

        let u2 = UserId::from_str("1235").unwrap();
        assert_ne!(u1, u2);
        assert_ne!(u1, 1235);
        assert_ne!(u1, "1235");

        /* This must give a compile error */
        /*
        let v1 = VideoId::from_str("1234").unwrap();
        assert_ne!(v1, u1);
        */
    }

}
