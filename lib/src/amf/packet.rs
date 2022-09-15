use std::mem::{size_of, transmute};
use std::ops::{Not};

use crate::Result;
use crate::amf::{Value, Version};
use amf::{amf0, amf3};
use bytes::{BufMut, Bytes, BytesMut, Buf, };
use try_buf::{TryBuf,};

#[derive(Debug)]
pub struct Packet<'a> {
    pub version: super::Version,
    pub headers: Vec<Header<'a>>,
    pub bodies: Vec<Body<'a>>,
}

#[derive(Debug)]
pub struct Header<'a> {
    pub name: &'a str,
    pub must_understand: bool,
    pub data: Value,
}

#[derive(Debug)]
pub struct Body<'a> {
    pub target_uri: &'a str,
    pub response_uri: &'a str,
    pub data: Value,
}

trait VersionVal: Sized {
    fn val(&self) -> u16;
    fn parse(ver: u16) -> Option<Self>;
}

impl VersionVal for Version {
    fn val(&self) -> u16 {
        match self {
            Version::Amf0 => 0,
            Version::Amf3 => 3,
        }
    }

    fn parse(ver: u16) -> Option<Self> {
        match ver {
            0 => Some(Version::Amf0) ,
            3 => Some(Version::Amf3),
            _ => None,
        }
    }
}

impl<'a> Packet<'a> {
    #[inline]
    pub(crate) fn size_hint(&self) -> usize {
        size_of::<u16>()            // version
            + size_of::<u16>()      // number of headers
            + self.headers
                .iter()
                .map(Header::size_hint)
                .sum::<usize>()
            + size_of::<u16>()      // number of bodies
            + self.bodies
                .iter()
                .map(Body::size_hint)
                .sum::<usize>()
    }

    pub fn builder() -> PacketBuilder<'a> {
        PacketBuilder::new()
    }

    pub fn read_from<'s,T: Buf>(src: &'s mut T) -> Result<Packet<'s>> {
        src.read_as().map_err(|e: Box<dyn std::error::Error>| e.to_string().into())
    }

}

#[derive(Default)]
pub struct PacketBuilder<'a> {
    version: Option<super::Version>,
    headers: Vec<Header<'a>>,
    bodies: Vec<Body<'a>>,
}

impl<'a> PacketBuilder<'a> {

    /// return a new packet with `amf3` version
    #[inline]
    pub fn new() -> Self {
        PacketBuilder::default()
    }

    pub fn build(self) -> Result<Packet<'a>, &'static str> {
        Ok(Packet {
            version: self.version.ok_or("the packet version must be set.")?,
            headers: self.headers,
            bodies: self.bodies,
        })
    }

    pub fn with_default_version(mut self) -> Self {
        self.version = Some(Version::Amf3);
        self
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    pub fn header(mut self, name: &'a str, must_understand: bool, data: Value) -> Self {
        let header = Header {
            name,
            must_understand,
            data,
        };
        self.headers.push(header);
        self
    }

    pub fn body<V>(mut self, target_uri: &'a str, response_uri: &'a str, data: V) -> Self
    where
        V: Into<Value>
    {
        let data = data.into();
        let body = Body {
            target_uri,
            response_uri,
            data,
        };
        self.bodies.push(body);
        self
    }

}

pub trait IntoBytes {
    fn into_bytes(self) -> Bytes;
}

impl IntoBytes for Value {
    fn into_bytes(self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(self.size_hint()).writer();
        self.write_to(&mut bytes).unwrap();

        bytes.into_inner().freeze()
    }
}

impl IntoBytes for Packet<'_> {
    fn into_bytes(self) -> Bytes {
        self.into()
    }
}

impl Into<Bytes> for Packet<'_> {
    fn into(self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(self.size_hint());
        bytes.put_u16(self.version.val());

        bytes.put_u16(self.headers.len() as u16);

        // put headers
        for h in self.headers {
            let _header_bytes: Bytes = h.into();
            bytes.put(_header_bytes);
        }

        bytes.put_u16(self.bodies.len() as u16);

        // put bodies
        for b in self.bodies {
            let _body_bytes: Bytes = b.into();
            bytes.put(_body_bytes);
        }

        bytes.freeze()
    }
}

impl Into<Bytes> for Header<'_> {
    fn into(self) -> Bytes {
        let mut bytes = BytesMut::new();

        // put name' length before name itself
        bytes.put_u16(self.name.len() as u16);
        bytes.put(self.name.as_bytes());

        bytes.put_u8(self.must_understand as u8);

        // put data' length before the data itself
        let _header_data: Bytes = self.data.into_bytes();
        bytes.put_u32(_header_data.len() as u32);
        bytes.put(_header_data);

        bytes.freeze()
    }
}

impl Into<Bytes> for Body<'_> {
    fn into(self) -> Bytes {
        let mut bytes = BytesMut::new();

        bytes.put_u16(self.target_uri.len() as u16);
        bytes.put(self.target_uri.as_bytes());

        bytes.put_u16(self.response_uri.len() as u16);
        bytes.put(self.response_uri.as_bytes());

        let _data: Bytes = self.data.into_bytes();
        bytes.put_u32(_data.len() as u32);
        bytes.put(_data);

        bytes.freeze()
    }
}

impl SizeHint for Header<'_> {
    fn size_hint(&self) -> usize {
        (16 + 8 + 32) / 8 + self.name.len() + self.data.size_hint()
    }
}

impl SizeHint for Body<'_> {
    fn size_hint(&self) -> usize {
        (16 + 16 + 32) / 8 + self.target_uri.len() + self.response_uri.len() + self.data.size_hint()
    }
}

pub(crate) trait SizeHint {
    fn size_hint(&self) -> usize;
}

impl SizeHint for Value {
    fn size_hint(&self) -> usize {
        match self {
            Value::Amf0(val) => val.size_hint(),
            Value::Amf3(val) => val.size_hint(),
        }
    }
}

impl SizeHint for amf0::Value {
    fn size_hint(&self) -> usize {
        return size_of::<u8>() + 128;

        // use amf::Amf0Value::*;
        //
        // fn size_of_str(s: &str) -> usize {
        //     // string length & string
        //     size_of::<u16>() + s.len()
        // }
        //
        // the first byte indicates AMF type (type marker)
        // size_of::<u8>() + match self {
        //     Number(val) => size_of_val(val),
        //     Boolean(val) => size_of_val(val),
        //     String(val) => size_of_str(val),
        //     Object { class_name, entries } =>
        //             todo!(),
        //     Null | Undefined=> 0,
        //     EcmaArray { entries } => todo!(),
        //     Array { entries } =>
        //             entries.iter()
        //                 .map(SizeHint::size_hint)
        //                 .sum::<usize>(),
        //     Date { unix_time: _, time_zone: _ } =>
        //             size_of::<f64>()
        //             + size_of::<i16>(),
        //     XmlDocument(val) => val.len(),
        //     AvmPlus(val) => val.size_hint(),
        // }
    }
}

impl SizeHint for amf3::Value {
    fn size_hint(&self) -> usize {
        return size_of::<u8>() + 128;

        // use amf::Amf3Value::*;
        //
        // // the first byte indicates AMF type (type marker)
        // size_of::<u8>() + match self {
        //     Undefined | Null => 0,
        //     Boolean(val) => size_of_val(val),
        //     Integer(val) => todo!(),
        //     Double(val) => todo!(),
        //     String(_) => todo!(),
        //     XmlDocument(val) => todo!(),
        //     Date { unix_time } => todo!(),
        //     Array { assoc_entries, dense_entries } => todo!(),
        //     Object { class_name, sealed_count, entries } => todo!(),
        //     Xml(val) => val.len(),
        //     ByteArray(val) => val.len(),
        //     IntVector { is_fixed, entries } => todo!(),
        //     UintVector { is_fixed, entries } => todo!(),
        //     DoubleVector { is_fixed, entries } => todo!(),
        //     ObjectVector { class_name, is_fixed, entries } => todo!(),
        //     Dictionary { is_weak, entries } => todo!(),
        // }
    }
}

impl Into<reqwest::Body> for Packet<'_> {
    fn into(self) -> reqwest::Body {
        self.into_bytes().into()
    }
}

pub trait ReadAs<Target> {
    type Error;
    fn read_as(&mut self) -> Result<Target, Self::Error>;
}

impl<'a, Src: Buf> ReadAs<Packet<'a>> for Src {
    type Error = Box<dyn std::error::Error>;

    fn read_as(&mut self) -> Result<Packet<'a>, Self::Error> {
        let ver = self.try_get_u16()?;
        let version = Version::parse(ver)
            .ok_or("unknow amf version")?;

        let header_len = self.try_get_u16()? as usize;
        let mut headers = Vec::with_capacity(header_len);
        for _ in 0..header_len {
            headers.push(self.read_as()
                .map_err(|e| {
                    format!("fail to parse amf packet header: {}", e)
                })?
            );
        }

        let body_len = self.try_get_u16()? as usize;
        let mut bodies = Vec::with_capacity(body_len);
        for _ in 0..body_len {
            bodies.push(self.read_as()
                .map_err(|e| {
                    format!("fail to parse amf packet body: {}", e)
                })?
            );
        }

        Ok(Packet {
            version,
            headers,
            bodies
        })
    }
}

impl<'a, Src: Buf> ReadAs<Header<'a>> for Src {
    type Error = Box<dyn std::error::Error>;

    fn read_as(&mut self) -> Result<Header<'a>, Self::Error> {
        let len = self.try_get_u16()?;
        let name = self.try_copy_to_bytes(len as usize)
            .map_err(|e| format!("fail to parse name: {}",e))?;

        let must_understand = self.try_get_u8()
            .map_err(|e| format!("fail to parse must_understand: {}",e))?
            .eq(&0)
            .not();

        let len = self.try_get_u32()?;
        let data = self.try_copy_to_bytes(len as usize)?;
        let data = Value::read_from(data.reader(), Version::Amf0)
            .map_err(|e| format!("fail to parse data: {}",e))?;

        unsafe {
            let name = transmute(name.as_ref());
            Ok(Header {
                name,
                must_understand,
                data,
            })
        }
    }
}

impl<'a,Src: Buf> ReadAs<Body<'a>> for Src {
    type Error = Box<dyn std::error::Error>;

    fn read_as(&mut self) -> Result<Body<'a>, Self::Error> {
        let len = self.try_get_u16()?;
        let target_uri = self.try_copy_to_bytes(len as usize)
            .map_err(|e| format!("fail to parse target_uri: {}",e))?;

        let len = self.try_get_u16()?;
        let response_uri = self.try_copy_to_bytes(len as usize)
            .map_err(|e| format!("fail to parse response_uri: {}",e))?;

        let len = self.try_get_u32()?;
        let data = self.try_copy_to_bytes(len as usize)?;
        let data = Value::read_from(data.reader(), Version::Amf0)
            .map_err(|e| format!("fail to parse data: {}",e))?;

        unsafe {
            let target_uri = transmute(target_uri.as_ref());
            let response_uri = transmute(response_uri.as_ref());
            Ok(Body {
                target_uri,
                response_uri,
                data,
            })
        }    
    }
}
