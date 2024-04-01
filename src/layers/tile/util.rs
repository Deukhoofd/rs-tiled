use std::borrow::Cow;
use std::ops::Deref;
use std::{convert::TryInto, io::Read};

use base64::Engine;
use quick_xml::events::Event;

use crate::{util::XmlEventResult, Error, LayerTileData, MapTilesetGid, Result};

pub(crate) fn parse_data_line<'a>(
    encoding: Option<String>,
    compression: Option<String>,
    parser: &mut impl Iterator<Item = XmlEventResult<'a>>,
    tilesets: &[MapTilesetGid],
    size: usize,
) -> Result<Vec<Option<LayerTileData>>> {
    match (encoding.as_deref(), compression.as_deref()) {
        (Some("csv"), None) => decode_csv(parser, tilesets, size),

        (Some("base64"), None) => parse_base64(parser).map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("zlib")) => parse_base64(parser)
            .and_then(|data| process_decoder(libflate::zlib::Decoder::new(&data[..])))
            .map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("gzip")) => parse_base64(parser)
            .and_then(|data| process_decoder(libflate::gzip::Decoder::new(&data[..])))
            .map(|v| convert_to_tiles(&v, tilesets)),
        #[cfg(feature = "zstd")]
        (Some("base64"), Some("zstd")) => parse_base64(parser)
            .and_then(|data| process_decoder(zstd::stream::read::Decoder::with_buffer(&data[..])))
            .map(|v| convert_to_tiles(&v, tilesets)),

        _ => Err(Error::InvalidEncodingFormat {
            encoding,
            compression,
        }),
    }
}

fn parse_base64<'a>(parser: &mut impl Iterator<Item = XmlEventResult<'a>>) -> Result<Vec<u8>> {
    for next in parser {
        match next.map_err(Error::XmlDecodingError)? {
            Event::Text(s) => {
                let s = String::from_utf8_lossy(s.deref());
                return base64::engine::GeneralPurpose::new(
                    &base64::alphabet::STANDARD,
                    base64::engine::general_purpose::PAD,
                )
                .decode(s.as_bytes())
                .map_err(Error::Base64DecodingError);
            }
            Event::End(e) if e.name().local_name().as_ref() == "data".as_bytes() => {
                return Ok(Vec::new());
            }
            _ => {}
        }
    }
    Err(Error::PrematureEnd("Ran out of XML data".to_owned()))
}

fn process_decoder(decoder: std::io::Result<impl Read>) -> Result<Vec<u8>> {
    decoder
        .and_then(|mut decoder| {
            let mut data = Vec::new();
            decoder.read_to_end(&mut data)?;
            Ok(data)
        })
        .map_err(Error::DecompressingError)
}

fn decode_csv<'a>(
    parser: &mut impl Iterator<Item = XmlEventResult<'a>>,
    tilesets: &[MapTilesetGid],
    size: usize,
) -> Result<Vec<Option<LayerTileData>>> {
    for next in parser {
        match next.map_err(Error::XmlDecodingError)? {
            Event::Text(s) => {
                let s = String::from_utf8_lossy(s.deref());
                let tiles = decode_csv_vec(s, size, tilesets);
                return Ok(tiles);
            }
            Event::End(e) if e.name().local_name().as_ref() == "data".as_bytes() => {
                return Ok(Vec::new());
            }
            _ => {}
        }
    }
    Err(Error::PrematureEnd("Ran out of XML data".to_owned()))
}

fn convert_to_tiles(data: &[u8], tilesets: &[MapTilesetGid]) -> Vec<Option<LayerTileData>> {
    data.chunks_exact(4)
        .map(|chunk| {
            let bits = u32::from_le_bytes(chunk.try_into().unwrap());
            LayerTileData::from_bits(bits, tilesets)
        })
        .collect()
}

fn decode_csv_vec(
    input: Cow<'_, str>,
    size: usize,
    tilesets: &[MapTilesetGid],
) -> Vec<Option<LayerTileData>> {
    let mut vec = Vec::with_capacity(size);
    let mut current = 0;
    for c in input.chars() {
        match c {
            ',' => {
                let tile = LayerTileData::from_bits(current, tilesets);
                vec.push(tile);
                current = 0;
            }
            '0'..='9' => {
                current *= 10;
                current += c as u32 - '0' as u32;
            }
            _ => {}
        }
    }
    vec.push(LayerTileData::from_bits(current, tilesets));
    vec
}
