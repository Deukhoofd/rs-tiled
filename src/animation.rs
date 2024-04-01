//! Structures related to tile animations.

use crate::{
    error::{Error, Result},
    util::{get_attrs, parse_cow, parse_tag, XmlEventResult},
};
use quick_xml::events::attributes::Attributes;

/// A structure describing a [frame] of a [TMX tile animation].
///
/// [frame]: https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tmx-frame
/// [TMX tile animation]: https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#animation
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Frame {
    /// The local ID of a tile within the parent tileset.
    pub tile_id: u32,
    /// How long (in milliseconds) this frame should be displayed before advancing to the next frame.
    pub duration: u32,
}

impl Frame {
    pub(crate) fn new(attrs: Attributes) -> Result<Frame> {
        let (tile_id, duration) = get_attrs!(
            for v in attrs {
                "tileid" => tile_id ?= parse_cow(&v),
                "duration" => duration ?= parse_cow(&v),
            }
            (tile_id, duration)
        );
        Ok(Frame { tile_id, duration })
    }
}

pub(crate) fn parse_animation<'a>(
    parser: &mut impl Iterator<Item = XmlEventResult<'a>>,
) -> Result<Vec<Frame>> {
    let mut animation = Vec::new();
    parse_tag!(parser, "animation", {
        "frame" => |attrs: Attributes| {
            animation.push(Frame::new(attrs.clone())?);
            Ok(())
        },
    });
    Ok(animation)
}
