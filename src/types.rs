use std::convert::{AsRef, Into};
use std::sync::Arc;

/* Used for Id's that can be interpreted as integers but aren't returned as 
 * an int by Twitch's API. (Maybe to allow a quick switch to a different representation 
 * without breaking the json schema?)
 */

#[derive(Clone)]
pub struct IntegerId {
    inner: Arc<IntegerIdRef>
}

struct IntegerIdRef {
    id: String,
    int: u32,
}

impl AsRef<u32> for IntegerId {
    fn as_ref(&self) -> &u32 {
        &self.inner.int
    }
}

impl AsRef<str> for IntegerId {

    fn as_ref(&self) -> &str {
        &self.inner.id
    }
}

use std::fmt;
use std::fmt::{Display, Debug, Formatter};

impl Display for IntegerId { 

    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{}", &self.inner.id)
    }
}

impl Debug for IntegerId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        write!(f, "{:?}", &self.inner.id)
    }
}

use serde::{Deserialize, Deserializer}; 
impl<'de> Deserialize<'de> for IntegerId {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let id = String::deserialize(deserializer)?;
        let int = (&id).parse::<u32>().map_err(serde::de::Error::custom)?;

        Ok(IntegerId { 
            inner: Arc::new(IntegerIdRef {
                id: id,
                int: int,
            })
        })
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

pub type UserId = IntegerId;
pub type ChannelId = UserId;
pub type VideoId = IntegerId;


pub type ClipId = Id;
