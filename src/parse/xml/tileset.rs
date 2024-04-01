use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

use crate::parse::xml::ReaderIterator;
use crate::{Error, ResourceCache, ResourceReader, Result, Tileset};

pub fn parse_tileset(
    path: &Path,
    reader: &mut impl ResourceReader,
    cache: &mut impl ResourceCache,
) -> Result<Tileset> {
    let mut tileset_parser = Reader::from_file(path).map_err(|_| Error::PathIsNotFile)?;
    let mut buf = Vec::new();
    let mut tileset_parser = ReaderIterator::new(&mut tileset_parser, &mut buf);
    loop {
        match tileset_parser
            .next()
            .unwrap()
            .map_err(Error::XmlDecodingError)?
        {
            Event::Start(e) if e.name().local_name().as_ref() == b"tileset" => {
                return Tileset::parse_external_tileset(
                    &mut tileset_parser.into_iter(),
                    e.attributes(),
                    path,
                    reader,
                    cache,
                );
            }
            Event::Eof => {
                return Err(Error::PrematureEnd(
                    "Tileset Document ended before map was parsed".to_string(),
                ))
            }
            _ => {}
        }
    }
}
