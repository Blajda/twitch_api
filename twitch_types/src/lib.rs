extern crate serde;

use std::borrow::Cow;
use std::cmp::{Eq, PartialEq};
use std::convert::AsRef;
use std::convert::Into;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/* Used for Id's that can be interpreted as integers but aren't returned as
 * an int by Twitch's API. (Maybe to allow a quick switch to a different representation
 * without breaking the json schema?)
 *
 * Don't Implement Display for StringID since it would allow comparisons between
 * different StringId types
 */

pub struct User {}
pub struct Video {}
pub struct Game {}
pub struct Clip {}
pub struct Stream {}

pub type UserId<'a> = StringId<'a, User>;
pub type BroadcasterId<'a> = UserId<'a>;
pub type VideoId<'a> = StringId<'a, Video>;
pub type ClipId<'a> = StringId<'a, Clip>;
pub type GameId<'a> = StringId<'a, Game>;
pub type StreamId<'a> = StringId<'a, Stream>;

#[derive(Clone)]
pub struct StringId<'a, T> {
    id: Cow<'a, str>,
    marker: PhantomData<T>,
}

impl<'a, T> StringId<'a, T> {
    pub fn new<S: Into<Cow<'a, str>>>(id: S) -> StringId<'a, T> {
        StringId {
            id: id.into(),
            marker: PhantomData,
        }
    }
}

impl<'a, T> From<&'a str> for StringId<'a, T> {
    fn from(id: &'a str) -> Self {
        StringId::new(id)
    }
}

impl<T> From<String> for StringId<'static, T> {
    fn from(id: String) -> Self {
        StringId::new(id)
    }
}

impl<'a, T> Into<String> for StringId<'a, T> {
    fn into(self) -> String {
        self.id.into()
    }
}

impl<'a, T> StringId<'_, T> {
    pub fn from_str(id: &'a str) -> Result<StringId<T>, ()> {
        Ok(StringId::new(id.to_owned()))
    }
}

impl<'a, T> AsRef<str> for StringId<'_, T> {
    fn as_ref(&self) -> &str {
        &self.id
    }
}

impl<'a, T> Debug for StringId<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.id)
    }
}

impl<'a, T> ToString for StringId<'a, T> {
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

impl<'a, T> PartialEq<StringId<'a, T>> for StringId<'a, T> {
    fn eq(&self, other: &StringId<'a, T>) -> bool {
        self.id == other.id
    }
}
impl<'a, T> Eq for StringId<'a, T> {}

impl<'a, T> PartialEq<str> for StringId<'a, T> {
    fn eq(&self, other: &str) -> bool {
        self.id.eq(other)
    }
}

use serde::{Deserialize, Deserializer};
impl<'de, T> Deserialize<'de> for StringId<'static, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        Ok(StringId::new(id))
    }
}

use serde::{Serialize, Serializer};
impl<'a, T> Serialize for StringId<'static, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id)
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
        assert_eq!(&u1, "1234");

        let u2 = UserId::from_str("1235").unwrap();
        assert_ne!(u1, u2);
        assert_ne!(&u1, "1235");

        /* This must give a compile error */
        //let v1 = VideoId::from_str("1234").unwrap();
        //assert_ne!(v1, u1);
    }
}
