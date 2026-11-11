//! Title block primitives <-> drawing entities, and reading a sheet back into
//! its definition. `codec` types only (no iced, no host types), so it is
//! checked against the public `acadrust` crate in the scratch test.

use codec::entities::{Line, Text, TextHorizontalAlignment, TextVerticalAlignment};
use codec::types::{LineWeight, Vector3};
use codec::xdata::ExtendedDataRecord;
use codec::{EntityType, Handle};

use super::geometry::{Anchor, TPrim, TShape};
use super::model::{TTag, TitleBlockDef, LAYER, XDATA_APP};
use crate::sipil_wall::engine::XVal;
use crate::sipil_wall::entities::{from_xdata, to_xdata};

pub fn record(tag: &TTag) -> ExtendedDataRecord {
    let mut rec = ExtendedDataRecord::new(XDATA_APP);
    for v in tag.encode() {
        rec.add_value(to_xdata(&v));
    }
    rec
}

pub fn read_tag(e: &EntityType) -> Option<TTag> {
    let rec = e.common().extended_data.get_record(XDATA_APP)?;
    let vals: Vec<XVal> = rec.values.iter().filter_map(from_xdata).collect();
    if vals.len() != rec.values.len() {
        return None;
    }
    TTag::decode(&vals)
}

fn plain_entity(p: &TPrim) -> EntityType {
    let mut e = match &p.shape {
        TShape::Line { a, b } => EntityType::Line(Line::from_points(
            Vector3::new(a[0], a[1], 0.0),
            Vector3::new(b[0], b[1], 0.0),
        )),
        TShape::Text {
            at,
            height,
            text,
            anchor,
        } => {
            let pt = Vector3::new(at[0], at[1], 0.0);
            let mut t = Text::with_value(text.clone(), pt).with_height(*height);
            t.horizontal_alignment = match anchor {
                Anchor::Center => TextHorizontalAlignment::Center,
                Anchor::Left => TextHorizontalAlignment::Left,
            };
            t.vertical_alignment = TextVerticalAlignment::Middle;
            t.alignment_point = Some(pt);
            EntityType::Text(t)
        }
    };
    e.common_mut().layer = LAYER.to_owned();
    if p.weight > 0 {
        e.common_mut().line_weight = LineWeight::Value(p.weight);
    }
    e
}

/// Entities of the primitives, all tagged; the first carries the definition.
pub fn to_entities(prims: &[TPrim], def: &TitleBlockDef) -> Vec<EntityType> {
    prims
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut e = plain_entity(p);
            e.common_mut().extended_data.upsert_record(record(&TTag {
                def: (i == 0).then(|| def.clone()),
            }));
            e
        })
        .collect()
}

/// The title block a set of entities holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub def: TitleBlockDef,
    pub handles: Vec<Handle>,
}

pub fn scan<'a>(entities: impl IntoIterator<Item = &'a EntityType>) -> Option<Found> {
    let mut def = None;
    let mut handles = Vec::new();
    for e in entities {
        let Some(tag) = read_tag(e) else { continue };
        handles.push(e.common().handle);
        if def.is_none() {
            def = tag.def;
        }
    }
    def.map(|def| Found { def, handles })
}
