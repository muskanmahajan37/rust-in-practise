use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Write;
use std::str;
use std::sync::Arc;

use bytes::{Buf, BufMut, Bytes, BytesMut};

pub struct TlvBox {
    m_objects: HashMap<i32, Bytes>,
    m_total_bytes: usize,
}

impl Clone for TlvBox {
    fn clone(&self) -> Self {
        TlvBox {
            m_objects: self.m_objects.clone(),
            m_total_bytes: self.m_total_bytes.clone(),
        }
    }
}


impl TlvBox {
    pub fn new() -> TlvBox {
        TlvBox {
            m_objects: HashMap::new(),
            m_total_bytes: 0,
        }
    }

    pub fn parse(buffer: Bytes, offset: i32, length: usize) -> TlvBox {
        let mut tlv_box = TlvBox::new();
        let mut parsed = 0;
        while parsed < length {
            let buffer_vec = buffer.to_vec();

            let type_start = offset as usize + parsed;
            let mut type_mut = BytesMut::with_capacity(4);
            type_mut.put(&buffer_vec[type_start..type_start + 4]);
            let typ = type_mut.get_i32();
            parsed += 4;

            let length_start = offset as usize + parsed;
            let mut size_mut = BytesMut::with_capacity(4);
            size_mut.put(&buffer_vec[length_start..length_start + 4]);
            let size = size_mut.get_i32();
            parsed += 4;

            let value_start = offset as usize + parsed;
            let mut value_mut = BytesMut::with_capacity(4);
            value_mut.put(&buffer_vec[value_start..value_start + 4]);

            tlv_box.putBytesValue(typ as i32, value_mut.freeze());

            parsed += size as usize;
        }

        return tlv_box;
    }

    pub fn putShortValue(&mut self, typ: i32, value: i16) {
        let mut buf = Vec::with_capacity(2);
        buf.put_i16(value);
        self.putBytesValue(typ, Bytes::from(buf))
    }

    pub fn putIntValue(&mut self, typ: i32, value: i32) {
        let mut buf = Vec::with_capacity(4);
        buf.put_i32(value);
        self.putBytesValue(typ, Bytes::from(buf))
    }

    pub fn putLongValue(&mut self, typ: i32, value: i64) {
        let mut buf = Vec::with_capacity(8);
        buf.put_i64(value);
        self.putBytesValue(typ, Bytes::from(buf))
    }

    pub fn putFloatValue(&mut self, typ: i32, value: f32) {
        let mut buf = Vec::with_capacity(4);
        buf.put_f32(value);
        self.putBytesValue(typ, Bytes::from(buf))
    }

    pub fn putDoubleValue(&mut self, typ: i32, value: f64) {
        let mut buf = Vec::with_capacity(8);
        buf.put_f64(value);
        self.putBytesValue(typ, Bytes::from(buf))
    }

    pub fn putStringValue(&mut self, typ: i32, value: String) {
        let len = value.clone().len();
        let mut byts = BytesMut::with_capacity(len);
        write!(byts, "{}", &value).unwrap();

        self.putBytesValue(typ, byts.freeze());
    }

    pub fn putObjectValue(&mut self, typ: i32, value: TlvBox) {
        self.putBytesValue(typ, value.clone().serialize());
    }

    pub fn putBytesValue(&mut self, typ: i32, value: Bytes) {
        self.m_objects.insert(typ, value.clone());
        self.m_total_bytes += value.len() + 8;
    }

    pub fn serialize(&mut self) -> Bytes {
        let mut offset = 0;
        let mut result = (0..u8::from(self.m_total_bytes as u8)).collect::<Vec<_>>();
        let keys = self.m_objects.keys();
        for key in keys.clone().into_iter() {
            let bytes = self.m_objects.get(&key.clone()).unwrap();
            let mut typ = BytesMut::with_capacity(4);
            typ.put_i32(key.clone());
            let mut length = BytesMut::with_capacity(4);
            length.put_i32(bytes.clone().len() as i32);

            result[offset..typ.len()].clone_from_slice(&typ.clone().to_vec());
            offset += 4;
            result[offset..offset + length.len()].clone_from_slice(&length.clone().to_vec());
            offset += 4;
            result[offset..offset + bytes.len()].clone_from_slice(&bytes.clone().to_vec());
            offset += bytes.len();
        }
        Bytes::from(result.clone())
    }

    pub fn getBytesValue(&self, typ: i32) -> Option<Bytes> {
        let bytes = self.m_objects.get(typ.clone().borrow());
        match bytes {
            None => None,
            Some(byts) => Some(byts.clone()),
        }
    }

    pub fn getShortValue(&self, typ: i32) -> Option<i16> {
        let mut bytes = self.m_objects.get(typ.clone().borrow());
        match bytes {
            Some(x) => Some(x.clone().get_i16()),
            None => None,
        }
    }

    pub fn getIntValue(&self, typ: i32) -> Option<i32> {
        let mut bytes = self.m_objects.get(typ.clone().borrow());
        match bytes {
            Some(x) => Some(x.clone().get_i32()),
            None => None,
        }
    }

    pub fn getLongValue(&self, typ: i32) -> Option<i64> {
        let mut bytes = self.m_objects.get(typ.clone().borrow());
        match bytes {
            Some(x) => Some(x.clone().get_i64()),
            None => None,
        }
    }
    pub fn getFloatValue(&self, typ: i32) -> Option<f32> {
        let mut bytes = self.m_objects.get(typ.clone().borrow());
        match bytes {
            Some(x) => Some(x.clone().get_f32()),
            None => None,
        }
    }
    pub fn getDoubleValue(&self, typ: i32) -> Option<f64> {
        let mut bytes = self.m_objects.get(typ.clone().borrow());
        match bytes {
            Some(x) => Some(x.clone().get_f64()),
            None => None,
        }
    }
    pub fn getStringValue(&self, typ: i32) -> String {
        let value = self.getBytesValue(typ);
        let buf = value.clone().unwrap().to_vec();
        let s = match str::from_utf8(&buf) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        String::from(s)
    }
    pub fn getObjectValue(&self, typ: i32) -> TlvBox {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use bytes::{BufMut, Bytes, BytesMut};

    use crate::tlv::tlvbox::TlvBox;

    #[test]
    fn test_parse_bytes() {
        let mut tlv_box = TlvBox::new();

        let mut buf = BytesMut::with_capacity(1024);
        buf.put(&b"hello world"[..]);
        tlv_box.putBytesValue(01, buf.clone().freeze());
        let value = tlv_box.getBytesValue(01);

        assert_eq!(11, value.clone().unwrap().len());
        assert_eq!(&b"hello world"[..], value.clone().unwrap());
    }

    #[test]
    fn test_parse_strings() {
        let mut tlv_box = TlvBox::new();
        tlv_box.putStringValue(01, String::from("hello, world"));
        let value = tlv_box.getBytesValue(01);

        assert_eq!(12, value.clone().unwrap().len());

        let a = format!("{:?}", Bytes::from(value.clone().unwrap()));

        assert_eq!("b\"hello, world\"", a);
        assert_eq!(1, tlv_box.m_objects.len());
        assert_eq!(20, tlv_box.m_total_bytes);

        assert_eq!("hello, world", tlv_box.getStringValue(01));
    }

    #[test]
    fn test_convert_short() {
        let mut tlv_box = TlvBox::new();
        tlv_box.putShortValue(01, 12);
        assert_eq!(12, tlv_box.getShortValue(01).unwrap());
    }

    #[test]
    fn test_convert_double() {
        let mut tlv_box = TlvBox::new();
        let value = -179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368.0000000000000000;;
        tlv_box.putDoubleValue(01, value);
        assert_eq!(value, tlv_box.getDoubleValue(01).unwrap());
    }

    #[test]
    fn test_convert_int() {
        let mut tlv_box = TlvBox::new();
        let value = 2332;
        tlv_box.putIntValue(01, value);
        assert_eq!(value, tlv_box.getIntValue(01).unwrap());
    }

    #[test]
    fn test_convert_long() {
        let mut tlv_box = TlvBox::new();
        let value = 2332;
        tlv_box.putLongValue(01, value);
        assert_eq!(value, tlv_box.getLongValue(01).unwrap());
    }

    #[test]
    fn test_convert_float() {
        let mut tlv_box = TlvBox::new();
        let value = 340282346638528859811704183484516925440.0000000000000000;
        tlv_box.putFloatValue(01, value);
        assert_eq!(value, tlv_box.getFloatValue(01).unwrap());
    }

    #[test]
    fn test_convert_object() {
        let mut tlv_test_obj = TlvBox::new();
        let value = 340282346638528859811704183484516925440.0000000000000000;
        tlv_test_obj.putFloatValue(02, value);
        assert_eq!(12, tlv_test_obj.m_total_bytes);

        let mut tlv_box = TlvBox::new();
        tlv_box.putObjectValue(01, tlv_test_obj);

        assert_eq!(1, tlv_box.m_objects.len());
        assert_eq!(20, tlv_box.m_total_bytes);

        let serialized = tlv_box.serialize();
        let result_box = TlvBox::parse(serialized.clone(), 0, serialized.clone().len());

        assert_eq!(1, result_box.m_objects.len());
        assert_eq!(12, result_box.m_total_bytes);
    }
}
