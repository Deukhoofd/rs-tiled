use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::util::XmlEventResult;
use crate::{Error, Map, ResourceCache, ResourceReader, Result};

pub fn parse_map(
    path: &Path,
    reader: &mut impl ResourceReader,
    cache: &mut impl ResourceCache,
) -> Result<Map> {
    let mut parser = Reader::from_file(path).map_err(|_| Error::PathIsNotFile)?;
    let mut buf = Vec::new();
    let mut iter = ReaderIterator::new(&mut parser, &mut buf);
    loop {
        match iter.next().unwrap().map_err(Error::XmlDecodingError)? {
            Event::Start(e) => {
                if e.name().local_name().as_ref() == b"map" {
                    return Map::parse_xml(&mut iter, e.attributes(), path, reader, cache);
                }
            }
            Event::End(_) => {
                return Err(Error::PrematureEnd(
                    "Document ended before map was parsed".to_string(),
                ))
            }
            _ => {}
        }
    }
}

pub struct ReaderIterator<'a> {
    reader: &'a mut Reader<BufReader<File>>,
    buf: &'a mut Vec<u8>,
}

impl<'own> ReaderIterator<'own> {
    pub fn new<'a>(
        reader: &'own mut Reader<BufReader<File>>,
        buf: &'own mut Vec<u8>,
    ) -> ReaderIterator<'own> {
        reader.trim_text(true);
        reader.expand_empty_elements(true);

        ReaderIterator { reader, buf }
    }
}

impl<'own> Iterator for ReaderIterator<'own> {
    type Item = XmlEventResult<'static>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(
            self.reader
                .read_event_into(&mut self.buf)
                .map(|v| v.into_owned()),
        )
    }
}
