use std::convert::AsRef;
use std::str::FromStr;
use std::sync::Arc;
use std::cmp::{Eq, PartialEq};
use std::marker::PhantomData;

/* Used for Id's that can be interpreted as integers but aren't returned as 
 * an int by Twitch's API. (Maybe to allow a quick switch to a different representation 
 * without breaking the json schema?)
 */

pub struct User {}
pub struct Video {}

pub type UserId = IntegerId<User>;
pub type ChannelId = UserId;
pub type VideoId = IntegerId<Video>;
pub type ClipId = Id;

#[derive(Clone)]
pub struct IntegerId<T> {
    inner: Arc<IntegerIdRef<T>>
}

struct IntegerIdRef<T> {
    id: String,
    int: u32,
    _type: PhantomData<T>
}

impl<T> IntegerId<T> {
    fn new(id: u32) -> IntegerId<T> {
        IntegerId { 
            inner: Arc::new(IntegerIdRef {
                id: id.to_string(),
                int: id,
                _type: PhantomData,
            })
        }
    }
}

impl<T> AsRef<u32> for IntegerId<T> {
    fn as_ref(&self) -> &u32 {
        &self.inner.int
    }
}

impl<T> AsRef<str> for IntegerId<T> {
    fn as_ref(&self) -> &str {
        &self.inner.id
    }
}

use std::fmt;
use std::fmt::{Display, Debug, Formatter};

impl<T> Display for IntegerId<T> { 

    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{}", &self.inner.id)
    }
}

impl<T> Debug for IntegerId<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{:?}", &self.inner.id)
    }
}

impl<T> PartialEq<IntegerId<T>> for IntegerId<T> {
    fn eq(&self, other: &IntegerId<T>) -> bool {
        self.inner.int == other.inner.int
    }
}
impl<T> Eq for IntegerId<T> {}

impl<T> PartialEq<&str> for IntegerId<T> {
    fn eq(&self, other: &&str) -> bool {
        self.inner.id == *other
    }
}

impl<T> PartialEq<u32> for IntegerId<T> {
    fn eq(&self, other: &u32) -> bool {
        self.inner.int == *other
    }
}


impl<T> FromStr for IntegerId<T> {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let int = u32::from_str(s)?;
        Ok(IntegerId::new(int))
    }
}

use serde::{Deserialize, Deserializer}; 
impl<'de, T> Deserialize<'de> for IntegerId<T> {

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

        /* This must give a compile error */
        /*
        let v1 = VideoId::from_str("1234").unwrap();
        assert_ne!(v1, u1);
        */
    }

}
