use std::{fmt::Formatter, hash::Hasher, io::Read};

use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine as _;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use twox_hash::XxHash32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Content {
    String(String),
    #[serde(with = "Base64Serde")]
    Binary(Vec<u8>),
}

impl AsRef<[u8]> for Content {
    fn as_ref(&self) -> &[u8] {
        match self {
            Content::String(s) => s.as_bytes(),
            Content::Binary(bs) => bs.as_slice(),
        }
    }
}

pub enum Base64Serde {}

impl Base64Serde {
    pub fn serialize<S, Input>(
        bytes: Input,
        serializer: S,
    ) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
        Input: AsRef<[u8]>,
    {
        serializer.serialize_str(&BASE64_STANDARD.encode(bytes.as_ref()))
    }

    pub fn deserialize<'de, D, Output>(deserializer: D) -> ::std::result::Result<Output, D::Error>
    where
        D: Deserializer<'de>,
        Output: From<Vec<u8>>,
    {
        struct Base64Visitor;

        impl<'de> de::Visitor<'de> for Base64Visitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "base64 ASCII text")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                BASE64_STANDARD.decode(v).map_err(de::Error::custom)
            }
        }

        deserializer
            .deserialize_str(Base64Visitor)
            .map(|vec| Output::from(vec))
    }
}

pub(crate) fn calc_fast_digest<R: Read>(mut r: R) -> Result<i32> {
    let mut buf = [0u8; 8192];
    let mut fast_hasher = XxHash32::default();
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        Hasher::write(&mut fast_hasher, &buf[0..n]);
    }
    let value = Hasher::finish(&fast_hasher);
    Ok(value as i32)
}
