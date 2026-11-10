//! Door primitives <-> drawing entities, and reading a drawing back into a
//! [`Model`]. `codec` types only (no iced, no host types), so it is checked
//! against the public `acadrust` crate in the scratch test (`test/wallcheck`).

use std::collections::BTreeMap;

use codec::entities::Text;
use codec::types::Vector3;
use codec::xdata::ExtendedDataRecord;
use codec::{EntityType, Handle};

use super::model::{DPrim, DShape, DTag, XDATA_APP};
use super::planner::{DoorRec, Edit, Model, Piece, SheetRec, WallRec};
use crate::sipil_wall::engine::{ColumnBox, Role, WallDef, XVal};
use crate::sipil_wall::entities::{from_xdata, moved_by, read_tag as read_wall_tag, shape_entity, to_xdata, PatternFn};

/// The XDATA record for a tag.
pub fn record(tag: &DTag) -> ExtendedDataRecord {
    let mut rec = ExtendedDataRecord::new(XDATA_APP);
    for v in tag.encode() {
        rec.add_value(to_xdata(&v));
    }
    rec
}

/// The door or sheet tag of an entity, or `None` if it has none.
pub fn read_tag(e: &EntityType) -> Option<DTag> {
    let rec = e.common().extended_data.get_record(XDATA_APP)?;
    let vals: Vec<XVal> = rec.values.iter().filter_map(from_xdata).collect();
    if vals.len() != rec.values.len() {
        return None;
    }
    DTag::decode(&vals)
}

/// One primitive as an entity on its role's layer, without a tag.
fn plain_entity(p: &DPrim, pattern: PatternFn) -> EntityType {
    let mut e = match &p.shape {
        DShape::Geo(s) => shape_entity(s, pattern),
        DShape::Text { at, height, text, rot } => EntityType::Text(
            Text::with_value(text.clone(), Vector3::new(at[0], at[1], 0.0))
                .with_height(*height)
                .with_rotation(*rot),
        ),
    };
    e.common_mut().layer = p.role.layer().to_owned();
    e
}

fn prim_entity(p: &DPrim, tag: &DTag, pattern: PatternFn) -> EntityType {
    let mut e = plain_entity(p, pattern);
    e.common_mut().extended_data.upsert_record(record(tag));
    e
}

/// Untagged entities for the rubber band under the cursor.
pub fn preview_entities(prims: &[DPrim], pattern: PatternFn) -> Vec<EntityType> {
    prims.iter().map(|p| plain_entity(p, pattern)).collect()
}

/// The entities of one piece of an edit. The first entity of a door or a
/// sheet carries the full definition (the anchor); the others only the id.
pub fn to_entities(piece: &Piece, pattern: PatternFn) -> Vec<EntityType> {
    match piece {
        Piece::Wall { id, def, out } => crate::sipil_wall::entities::to_entities(out, *id, def, pattern),
        Piece::Door { def, prims } => prims
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let tag = DTag::Door { id: def.id, def: (i == 0).then(|| def.clone()) };
                prim_entity(p, &tag, pattern)
            })
            .collect(),
        Piece::Sheet { code, origin, prims } => prims
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let tag = DTag::Sheet { code: code.clone(), origin: (i == 0).then_some(*origin) };
                prim_entity(p, &tag, pattern)
            })
            .collect(),
    }
}

/// Every entity of an edit, in order.
pub fn edit_entities(edit: &Edit, pattern: PatternFn) -> Vec<EntityType> {
    edit.add.iter().flat_map(|p| to_entities(p, pattern)).collect()
}

/// Read the walls, doors and detail sheets out of a drawing's entities.
pub fn scan<'a>(entities: impl IntoIterator<Item = &'a EntityType>) -> Model {
    let mut wall_defs: BTreeMap<u32, WallDef> = BTreeMap::new();
    let mut wall_handles: BTreeMap<u32, Vec<Handle>> = BTreeMap::new();
    let mut columns: Vec<(u32, ColumnBox)> = Vec::new();
    let mut door_defs: BTreeMap<u32, super::model::DoorDef> = BTreeMap::new();
    let mut door_handles: BTreeMap<u32, Vec<Handle>> = BTreeMap::new();
    let mut sheet_origins: BTreeMap<String, [f64; 2]> = BTreeMap::new();
    let mut sheet_handles: BTreeMap<String, Vec<Handle>> = BTreeMap::new();

    for e in entities {
        let handle = e.common().handle;
        if let Some(tag) = read_wall_tag(e) {
            wall_handles.entry(tag.id).or_default().push(handle);
            if tag.role == Role::Column {
                if let Some(b) = tag.column {
                    columns.push((tag.id, b));
                }
            }
            if let Some(mut def) = tag.def.clone() {
                if let Some(m) = moved_by(e, &tag) {
                    for p in &mut def.pts {
                        p[0] += m[0];
                        p[1] += m[1];
                    }
                }
                wall_defs.insert(tag.id, def);
            }
        }
        match read_tag(e) {
            Some(DTag::Door { id, def }) => {
                door_handles.entry(id).or_default().push(handle);
                if let Some(d) = def {
                    door_defs.insert(id, d);
                }
            }
            Some(DTag::Sheet { code, origin }) => {
                sheet_handles.entry(code.clone()).or_default().push(handle);
                if let Some(o) = origin {
                    sheet_origins.insert(code, o);
                }
            }
            None => {}
        }
    }

    Model {
        walls: wall_defs
            .into_iter()
            .map(|(id, def)| WallRec { id, def, handles: wall_handles.remove(&id).unwrap_or_default() })
            .collect(),
        columns,
        doors: door_defs
            .into_iter()
            .map(|(id, def)| DoorRec { def, handles: door_handles.remove(&id).unwrap_or_default() })
            .collect(),
        sheets: sheet_origins
            .into_iter()
            .map(|(code, origin)| SheetRec { handles: sheet_handles.remove(&code).unwrap_or_default(), code, origin })
            .collect(),
    }
}
