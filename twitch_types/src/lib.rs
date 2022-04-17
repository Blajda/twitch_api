extern crate serde;

use std::convert::AsRef;
use std::cmp::{Eq, PartialEq};
use std::marker::PhantomData;
use std::convert::Into;
use std::fmt;
use std::fmt::{Debug, Formatter};

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

pub type UserId = StringId<User>;
pub type BroadcasterId = UserId;
pub type VideoId = StringId<Video>;
pub type ClipId = StringId<Clip>;
pub type GameId = StringId<Game>;
pub type StreamId = StringId<Stream>;

#[derive(Clone)]
pub struct StringId<T> {
    id: String,
    marker: PhantomData<T>
}

impl<T> StringId<T> {
    pub fn new(id: String) -> StringId<T> {
        StringId { 
            id,
            marker: PhantomData,
        }
    }
}

impl<'a, T> StringId<T> {

    pub fn from_str(id: &'a str) 
        -> Result<StringId<T>, ()> 
    {
        Ok(StringId::new(id.to_owned()))
    }
}


impl<'a, T> AsRef<str> for StringId<T> {
    fn as_ref(&self) -> &str {
        &self.id
    }
}


impl<'a, T> Into<&'a str> for &'a StringId<T> {
    fn into(self) -> &'a str {
        &self.id
    }
}

impl<T> Debug for StringId<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{:?}", &self.id)
    }
}

impl<T> ToString for StringId<T> {
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

impl<T> PartialEq<StringId<T>> for StringId<T> {
    fn eq(&self, other: &StringId<T>) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for StringId<T> {}

impl<T> PartialEq<str> for StringId<T> {
    fn eq(&self, other: &str) -> bool {
        self.id.eq(other)
    }
}

use serde::{Deserialize, Deserializer}; 
impl<'de, T> Deserialize<'de> for StringId<T> {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let id = String::deserialize(deserializer)?;
        Ok(StringId::new(id))
    }
}

use serde::{Serialize, Serializer};
impl<'a, T> Serialize for StringId<T> {
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
