use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;
use std::sync::Arc;

use crate::parse::xml::ReaderIterator;
use crate::{
    util::*, EmbeddedParseResultType, Error, MapTilesetGid, ObjectData, ResourceCache,
    ResourceReader, Result, Tileset,
};

/// A template, consisting of an object and a tileset
///
/// Templates define a tileset and object data to use for an object that can be shared between multiple objects and
/// maps.
#[derive(Clone, Debug)]
pub struct Template {
    /// The tileset this template contains a reference to
    pub tileset: Option<Arc<Tileset>>,
    /// The object data for this template
    pub object: ObjectData,
}

impl Template {
    pub(crate) fn parse_template(
        path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Arc<Template>> {
        // Open the template file
        let mut template_parser = Reader::from_file(path).map_err(|_| Error::PathIsNotFile)?;
        let mut buf = Vec::new();
        let mut iterator = ReaderIterator::new(&mut template_parser, &mut buf);
        loop {
            match iterator.next().unwrap().map_err(Error::XmlDecodingError)? {
                Event::Start(e) if e.name().local_name().as_ref() == b"template" => {
                    let template =
                        Self::parse_external_template(&mut iterator, path, reader, cache)?;
                    return Ok(template);
                }
                Event::Eof => {
                    return Err(Error::PrematureEnd(
                        "Template Document ended before template element was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }

    fn parse_external_template<'a>(
        parser: &mut impl Iterator<Item = XmlEventResult<'a>>,
        template_path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Arc<Template>> {
        let mut object = Option::None;
        let mut tileset = None;
        let mut tileset_gid: Vec<MapTilesetGid> = vec![];

        parse_tag!(parser, "template", {
            "object" => |attrs| {
                object = Some(ObjectData::new(parser, attrs, Some(&tileset_gid), tileset.clone(), template_path.parent().ok_or(Error::PathIsNotFile)?, reader, cache)?);
                Ok(())
            },
            "tileset" => |attrs: Attributes| {
                let res = Tileset::parse_xml_in_map(parser, attrs, template_path, reader, cache)?;
                match res.result_type {
                    EmbeddedParseResultType::ExternalReference { tileset_path } => {
                        tileset = Some(if let Some(ts) = cache.get_tileset(&tileset_path) {
                            ts
                        } else {
                            let tileset = Arc::new(crate::parse::xml::parse_tileset(&tileset_path, reader, cache)?);
                            cache.insert_tileset(tileset_path.clone(), tileset.clone());
                            tileset
                        });
                    }
                    EmbeddedParseResultType::Embedded { tileset: embedded_tileset } => {
                        tileset = Some(Arc::new(embedded_tileset));
                    },
                };
                tileset_gid.push(MapTilesetGid {
                    tileset: tileset.clone().unwrap(),
                    first_gid: res.first_gid,
                });
                Ok(())
            },
        });

        let object = object.ok_or(Error::TemplateHasNoObject)?;

        Ok(Arc::new(Template { tileset, object }))
    }
}
