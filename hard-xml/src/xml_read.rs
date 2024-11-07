use crate::{XmlReader, XmlResult};

pub trait XmlRead<'a>: Sized {
    fn from_reader(reader: &mut XmlReader<'a>) -> XmlResult<Self>;

    fn from_str(text: &'a str) -> XmlResult<Self> {
        let mut reader = XmlReader::new(text);
        Self::from_reader(&mut reader)
    }
    fn from_file(path: impl AsRef<std::path::Path>) -> XmlResult<Self> {
        let text = std::fs::read_to_string(path)?;
        let mut reader = XmlReader::new(&text);
        Self::from_reader(&mut reader)
    }

    fn from_buffer<R: std::io::BufRead>(mut buf: R) -> XmlResult<Self> {
        let mut buffer = String::new();
        buf.read_to_string(&mut buffer)?;
        let mut reader = XmlReader::new(&buffer);
        Self::from_reader(&mut reader)
    }
}

pub trait XmlReadOwned: for<'s> XmlRead<'s> {}

impl<T> XmlReadOwned for T where T: for<'s> XmlRead<'s> {}
