use quick_xml::events::attributes::Attributes;
use std::str::FromStr;

use crate::util::parse_cow;
use crate::{
    error::Error,
    util::{get_attrs, XmlEventResult},
    Result, TileId,
};

/// The Wang ID, stored as an array of 8 u8 values.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WangId(pub [u8; 8]);

impl FromStr for WangId {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<WangId, Error> {
        let mut ret = [0u8; 8];
        let values: Vec<&str> = s
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .collect();
        if values.len() != 8 {
            return Err(Error::InvalidWangIdEncoding {
                read_string: s.to_string(),
            });
        }
        for i in 0..8 {
            ret[i] = values[i].parse::<u8>().unwrap_or(0);
        }

        Ok(WangId(ret))
    }
}

/// Stores the Wang ID.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WangTile {
    #[allow(missing_docs)]
    pub wang_id: WangId,
}

impl WangTile {
    /// Reads data from XML parser to create a WangTile.
    pub(crate) fn new<'a>(
        _parser: &mut impl Iterator<Item = XmlEventResult<'a>>,
        attrs: Attributes,
    ) -> Result<(TileId, WangTile)> {
        // Get common data
        let (tile_id, wang_id) = get_attrs!(
            for v in attrs {
                "tileid" => tile_id ?= parse_cow::<u32>(&v),
                "wangid" => wang_id ?= parse_cow(&v),
            }
            (tile_id, wang_id)
        );

        Ok((tile_id, WangTile { wang_id }))
    }
}
