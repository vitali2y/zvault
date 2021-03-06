use rmp_serde;
use serde::{Serialize, Deserialize};

use std::io::{Write, Read, Cursor};

pub use serde_bytes::ByteBuf as Bytes;
pub use rmp_serde::decode::Error as DecodeError;
pub use rmp_serde::encode::Error as EncodeError;


#[inline]
pub fn encode<T: Serialize>(t: &T) -> Result<Vec<u8>, EncodeError> {
    let mut data = Vec::new();
    {
        let mut writer = rmp_serde::Serializer::new(&mut data);
        try!(t.serialize(&mut writer));
    }
    Ok(data)
}

#[inline]
pub fn encode_to_stream<T: Serialize>(t: &T, w: &mut Write) -> Result<(), EncodeError> {
    let mut writer = rmp_serde::Serializer::new(w);
    t.serialize(&mut writer)
}

#[inline]
pub fn decode<'a, T: Deserialize<'a>>(data: &[u8]) -> Result<T, DecodeError> {
    let data = Cursor::new(data);
    let mut reader = rmp_serde::Deserializer::new(data);
    T::deserialize(&mut reader)
}

#[inline]
pub fn decode_from_stream<'a, T: Deserialize<'a>>(r: &mut Read) -> Result<T, DecodeError> {
    let mut reader = rmp_serde::Deserializer::new(r);
    T::deserialize(&mut reader)
}
