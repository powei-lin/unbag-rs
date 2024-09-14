use byteorder::{LittleEndian, ReadBytesExt};
use serde::de;
use std;
use std::fmt::{self, Display};
use std::io;

pub struct Deserializer<R> {
    reader: R,
    length: u32,
}

impl<R> Deserializer<R>
where
    R: io::Read,
{
    pub fn new(reader: R, expected_length: u32) -> Self {
        Deserializer {
            reader,
            length: expected_length,
        }
    }

    pub fn into_inner(self) -> R {
        self.reader
    }

    pub fn is_fully_read(&self) -> bool {
        self.length == 0
    }

    #[inline]
    fn reserve_bytes(&mut self, size: u32) -> Result<()> {
        if size > self.length {
            return Err(Error::Message("Not enough bytes left.".to_string()));
        }
        self.length -= size;
        Ok(())
    }

    #[inline]
    fn pop_length(&mut self) -> Result<u32> {
        self.reserve_bytes(4).expect("msg");
        let l = self.reader.read_u32::<LittleEndian>().expect("msg");
        Ok(l)
    }

    #[inline]
    fn get_string(&mut self) -> Result<String> {
        let length = self.pop_length()?;
        self.reserve_bytes(length)?;
        let mut buffer = vec![0; length as usize];
        self.reader
            .read_exact(&mut buffer)
            .expect("read exact error");
        let s = String::from_utf8(buffer).expect("from utf8 err");
        Ok(s)
    }
}

macro_rules! impl_nums {
    ($ty:ty, $dser_method:ident, $visitor_method:ident, $reader_method:ident, $bytes:expr) => {
        #[inline]
        fn $dser_method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.reserve_bytes($bytes).expect("reserve bytes error");
            let value = self
                .reader
                .$reader_method::<LittleEndian>()
                .expect("read error");
            visitor.$visitor_method(value)
        }
    };
}

impl<'de, 'a, R: io::Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("a")
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("b")
    }

    impl_nums!(u16, deserialize_u16, visit_u16, read_u16, 2);
    impl_nums!(u32, deserialize_u32, visit_u32, read_u32, 4);
    impl_nums!(u64, deserialize_u64, visit_u64, read_u64, 8);
    impl_nums!(i16, deserialize_i16, visit_i16, read_i16, 2);
    impl_nums!(i32, deserialize_i32, visit_i32, read_i32, 4);
    impl_nums!(i64, deserialize_i64, visit_i64, read_i64, 8);
    impl_nums!(f32, deserialize_f32, visit_f32, read_f32, 4);
    impl_nums!(f64, deserialize_f64, visit_f64, read_f64, 8);

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.reserve_bytes(1)?;
        let value = self.reader.read_u8().expect("read error");
        visitor.visit_u8(value)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.reserve_bytes(1)?;
        let value = self.reader.read_i8().expect("read error");
        visitor.visit_i8(value)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("not support char")
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(&self.get_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.get_string()?)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("d")
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("e")
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("f")
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("g")
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("h")
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("aa")
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len = self.pop_length()? as usize;

        struct Access<'a, R: io::Read + 'a> {
            deserializer: &'a mut Deserializer<R>,
            len: usize,
        }

        impl<'de, 'a, 'b: 'a, R: io::Read + 'b> de::SeqAccess<'de> for Access<'a, R> {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
            where
                T: de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    Ok(Some(seed.deserialize(&mut *self.deserializer)?))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        struct Access<'a, R: io::Read + 'a> {
            deserializer: &'a mut Deserializer<R>,
            len: usize,
        }

        impl<'de, 'a, 'b: 'a, R: io::Read + 'b> de::SeqAccess<'de> for Access<'a, R> {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
            where
                T: de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    Ok(Some(seed.deserialize(&mut *self.deserializer)?))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("aaab")
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("acb")
    }

    #[inline]
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("qq")
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("iii")
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Eof,
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            /* and so forth */
        }
    }
}

impl std::error::Error for Error {}

pub fn from_reader<'de, R, T>(reader: R, length: u32) -> Result<T>
where
    R: io::Read,
    T: de::Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(reader, length);
    let value = T::deserialize(&mut deserializer)?;
    if !deserializer.is_fully_read() {
        return Err(Error::Message("Not fully read.".to_string()));
    }
    Ok(value)
}

pub fn from_slice<'de, T>(bytes: &[u8]) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    from_reader(io::Cursor::new(bytes), bytes.len() as u32)
}

pub fn from_str<'de, T>(value: &str) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    from_slice(value.as_bytes())
}
