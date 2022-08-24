//! amf请求相关实现

use std::collections::HashMap;

pub use amf::{amf0, amf3, Amf0Value, Amf3Value, Value, Version};

pub mod packet;

pub trait TryIntoAmf0Object {
    fn try_into_amf0_object(self) -> Option<HashMap<String, Amf0Value>>;
}

pub trait TryAsAmf0Object {
    fn try_as_amf0_object(&self) -> Option<HashMap<&str, &Amf0Value>>;
}

pub trait TryAsNumber {
    fn try_as_number(&self) -> Option<f64>;
}

impl TryIntoAmf0Object for Value {
    fn try_into_amf0_object(self) -> Option<HashMap<String, Amf0Value>> {
        if let Value::Amf0(Amf0Value::Object{entries,..}) = self {
            let entries = entries.into_iter()
                .map(|p| (p.key, p.value));
            return Some(HashMap::from_iter(entries));
        }
        None
    }
}

impl TryAsAmf0Object for Amf0Value {
    fn try_as_amf0_object(&self) -> Option<HashMap<&str, &Amf0Value>> {
        if let Amf0Value::Object{entries,..} = self {
            let entries = entries.into_iter()
                .map(|p| (p.key.as_str(), &p.value));
            return Some(HashMap::from_iter(entries));
        }
        None
    }
}

impl TryAsAmf0Object for Value {
    fn try_as_amf0_object(&self) -> Option<HashMap<&str, &Amf0Value>> {
        if let Value::Amf0(Amf0Value::Object{entries,..}) = self {
            let entries = entries.into_iter()
                .map(|p| (p.key.as_str(), &p.value));
            return Some(HashMap::from_iter(entries));
        }
        None
    }
}

impl TryAsNumber for Amf0Value {
    fn try_as_number(&self) -> Option<f64> {
        if let Amf0Value::Number(num)= self {
            return Some(*num);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    use super::packet::*;

    use amf0::{array, number, object, string};
    use bytes::Bytes;

    #[test]
    fn test_serialize() -> Result<(), Box<dyn std::error::Error>>{
        let p = Packet::builder()
            .with_default_version()
            .body(
                "amfService.dispatchAMF",
                "/64", 
                Value::from(object([
                    ("name", string("Mike")),
                    ("age", number(16)),
                    ("numbers", array(vec![
                        number(999),
                        number(1001)
                    ]))
                ].into_iter())),
            )
            .build()?;
        let bytes = p.into_bytes();

        // display
        for (i, byte) in bytes.iter().enumerate() {
            print!("{:02x?} ", byte);
            if (i+1)%8 == 0 {
                if (i+1)%16 == 0 {
                    print!("\n");
                } else {
                    print!("  ");
                }
            }
        }
        println!("");

        Ok(())
    }

    #[test]
    fn test_deser() -> crate::Result<()>{
        let mut req = Bytes::from_static(include_bytes!("../test_req.amf"));
        let packet: Packet = req.read_as()?;
        println!("Request Packet:\n {:?}\n\n", packet);

        let resp = include_bytes!("../test_resp.amf");
        let mut resp = Bytes::from_static(resp);
        let packet: Packet = resp.read_as()?;
        println!("Response Packet:\n {:?}\n\n", packet);

        Ok(())
    }
}

