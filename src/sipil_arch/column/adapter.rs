//! Column primitives <-> drawing entities, and reading a drawing back into a
//! [`Model`]. `codec` types only, so it is checked in the scratch crate.

use std::collections::BTreeMap;

use codec::xdata::ExtendedDataRecord;
use codec::{EntityType, Handle};

use super::model::{Tag, XDATA_APP};
use super::planner::{ColumnRec, Edit, Model, Piece};
use crate::sipil_arch::steel::draw::{self, Prim, SShape};
use crate::sipil_wall::engine::{Shape, XVal, P};
use crate::sipil_wall::entities::{from_xdata, to_xdata, PatternFn};

pub fn record(tag: &Tag) -> ExtendedDataRecord {
    let mut rec = ExtendedDataRecord::new(XDATA_APP);
    for v in tag.encode() {
        rec.add_value(to_xdata(&v));
    }
    rec
}

pub fn read_tag(e: &EntityType) -> Option<Tag> {
    let rec = e.common().extended_data.get_record(XDATA_APP)?;
    let vals: Vec<XVal> = rec.values.iter().filter_map(from_xdata).collect();
    if vals.len() != rec.values.len() {
        return None;
    }
    Tag::decode(&vals)
}

/// The first point of a section outline, to notice when it was moved.
pub fn first_point(prims: &[Prim]) -> Option<P> {
    prims.iter().find_map(|p| match &p.shape {
        SShape::Geo(Shape::Poly { pts, .. }) => pts.first().copied(),
        _ => None,
    })
}

/// The entities of a piece; the first (the outline) carries the definition.
pub fn to_entities(piece: &Piece, pattern: PatternFn) -> Vec<EntityType> {
    let origin = first_point(&piece.prims);
    piece
        .prims
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let tag = Tag { id: piece.def.id, def: (i == 0).then(|| piece.def.clone()), origin: if i == 0 { origin } else { None } };
            draw::entity(p, pattern, Some(record(&tag)))
        })
        .collect()
}

pub fn edit_entities(edit: &Edit, pattern: PatternFn) -> Vec<EntityType> {
    edit.add.iter().flat_map(|p| to_entities(p, pattern)).collect()
}

/// Untagged entities for the rubber band under the cursor.
pub fn preview_entities(prims: &[Prim], pattern: PatternFn) -> Vec<EntityType> {
    prims.iter().map(|p| draw::entity(p, pattern, None)).collect()
}

fn outline_start(e: &EntityType) -> Option<P> {
    match e {
        EntityType::LwPolyline(pl) => pl.vertices.first().map(|v| [v.location.x, v.location.y]),
        _ => None,
    }
}

/// Read the columns out of a drawing's entities. A column moved with MOVE is
/// found where it is now (its outline's first point against where it started).
pub fn scan<'a>(entities: impl IntoIterator<Item = &'a EntityType>) -> Model {
    let mut defs = BTreeMap::new();
    let mut handles: BTreeMap<u32, Vec<Handle>> = BTreeMap::new();
    for e in entities {
        let Some(tag) = read_tag(e) else { continue };
        handles.entry(tag.id).or_default().push(e.common().handle);
        if let Some(mut def) = tag.def.clone() {
            if let (Some(now), Some(was)) = (outline_start(e), tag.origin) {
                def.at = [def.at[0] + now[0] - was[0], def.at[1] + now[1] - was[1]];
            }
            defs.insert(tag.id, def);
        }
    }
    Model { columns: defs.into_iter().map(|(id, def)| ColumnRec { def, handles: handles.remove(&id).unwrap_or_default() }).collect() }
}
