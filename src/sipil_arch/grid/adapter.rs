//! Grid primitives <-> drawing entities, and reading a drawing back into the
//! grid it holds. `codec` types only (no iced, no host types), so it is checked
//! against the public `acadrust` crate in the scratch test (`test/wallcheck`).

use codec::entities::{Circle, Line, Text, TextHorizontalAlignment, TextVerticalAlignment};
use codec::types::Vector3;
use codec::xdata::ExtendedDataRecord;
use codec::{EntityType, Handle};

use super::model::{GTag, GridDef, XDATA_APP};
use super::plan::{Anchor, GPrim, GShape};
use crate::sipil_wall::engine::XVal;
use crate::sipil_wall::entities::{from_xdata, to_xdata};

/// The XDATA record for a tag.
pub fn record(tag: &GTag) -> ExtendedDataRecord {
    let mut rec = ExtendedDataRecord::new(XDATA_APP);
    for v in tag.encode() {
        rec.add_value(to_xdata(&v));
    }
    rec
}

/// The grid tag of an entity, or `None` if it has none.
pub fn read_tag(e: &EntityType) -> Option<GTag> {
    let rec = e.common().extended_data.get_record(XDATA_APP)?;
    let vals: Vec<XVal> = rec.values.iter().filter_map(from_xdata).collect();
    if vals.len() != rec.values.len() {
        return None;
    }
    GTag::decode(&vals)
}

fn plain_entity(p: &GPrim) -> EntityType {
    let mut e = match &p.shape {
        GShape::Line { a, b } => EntityType::Line(Line::from_points(
            Vector3::new(a[0], a[1], 0.0),
            Vector3::new(b[0], b[1], 0.0),
        )),
        GShape::Circle { c, r } => EntityType::Circle(Circle::from_center_radius(
            Vector3::new(c[0], c[1], 0.0),
            *r,
        )),
        GShape::Text {
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
    e.common_mut().layer = p.layer.name().to_owned();
    e
}

/// Entities of the primitives, each tagged with the view; the first also
/// carries the whole definition (the anchor the drawing is read back from).
pub fn to_entities(prims: &[GPrim], view: &str, scale: f64, def: &GridDef) -> Vec<EntityType> {
    prims
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let tag = GTag {
                view: view.to_owned(),
                scale,
                def: (i == 0).then(|| def.clone()),
            };
            let mut e = plain_entity(p);
            e.common_mut().extended_data.upsert_record(record(&tag));
            e
        })
        .collect()
}

/// The grid a drawing holds: its definition, its view and every entity of it.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub def: GridDef,
    pub view: String,
    pub scale: f64,
    pub handles: Vec<Handle>,
}

/// Read the grid out of a drawing's entities (the first one, if there are
/// several), or `None` if there is none.
pub fn scan<'a>(entities: impl IntoIterator<Item = &'a EntityType>) -> Option<Found> {
    let mut def: Option<(GridDef, String, f64)> = None;
    let mut handles = Vec::new();
    for e in entities {
        let Some(tag) = read_tag(e) else { continue };
        handles.push(e.common().handle);
        if def.is_none() {
            if let Some(d) = tag.def {
                def = Some((d, tag.view, tag.scale));
            }
        }
    }
    def.map(|(def, view, scale)| Found {
        def,
        view,
        scale,
        handles,
    })
}
