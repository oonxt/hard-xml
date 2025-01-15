use std::io::{Read, Seek};
use crate::{XmlReader, XmlResult};

pub trait XmlRead<'a>: Sized {
    fn from_reader(reader: &mut XmlReader<'a>) -> XmlResult<Self>;

    fn from_str(text: &'a str) -> XmlResult<Self> {
        let mut reader = XmlReader::new(text);
        Self::from_reader(&mut reader)
    }
}

pub trait XmlReadOwned: for<'s> XmlRead<'s> {
    #[cfg(feature = "zip")]
    fn from_archive_file<R: Read + Seek>(archive: &mut zip::ZipArchive<R>, path: &str) -> XmlResult<Self> {
        let mut file = archive.by_name(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        let mut reader = XmlReader::new(&buffer);
        Self::from_reader(&mut reader)
    }
}

impl<T> XmlReadOwned for T where T: for<'s> XmlRead<'s> {}
