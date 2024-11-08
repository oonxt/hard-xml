use crate::{XmlReader, XmlResult};
use std::io::{BufReader, Read};

pub trait XmlRead<'a>: Sized {
    fn from_reader(reader: &mut XmlReader<'a>) -> XmlResult<Self>;

    fn from_str(text: &'a str) -> XmlResult<Self> {
        let mut reader = XmlReader::new(text);
        Self::from_reader(&mut reader)
    }

    fn from_buffer<R: std::io::BufRead + 'a>(mut buf: R, buffer: &'a mut String) -> XmlResult<Self> {
        buf.read_to_string(buffer)?;
        let mut reader = XmlReader::new(buffer);
        Self::from_reader(&mut reader)
    }
}

pub trait XmlReadOwned: for<'s> XmlRead<'s> {}

impl<T> XmlReadOwned for T where T: for<'s> XmlRead<'s> {}
