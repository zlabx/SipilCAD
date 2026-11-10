//! Wall primitives <-> drawing entities.
//!
//! Only `codec` types here (no iced, no `crate::` host types), so this file is
//! checked against the public `acadrust` crate in the kit's scratch test
//! (`test/wallcheck`). The one host-specific piece, how a catalog hatch pattern
//! becomes prebaked pattern lines, is passed in as a function.

use super::engine::{centroid, Prim, Role, Shape, Tag, WallDef, XVal, XDATA_APP};
use super::engine::{Output, P};
use codec::entities::{
    BoundaryEdge, BoundaryPath, Hatch, HatchPattern, HatchPatternType, Line, LwPolyline,
    PolylineEdge,
};
use codec::types::{Vector2, Vector3};
use codec::xdata::{ExtendedDataRecord, XDataValue};
use codec::EntityType;

/// Resolve a catalog pattern name and scale to the pattern lines a hatch
/// stores (final world geometry). `None` if the catalog lacks the name.
pub type PatternFn<'a> = &'a dyn Fn(&str, f64) -> Option<HatchPattern>;

pub fn to_xdata(v: &XVal) -> XDataValue {
    match v {
        XVal::I16(i) => XDataValue::Integer16(*i),
        XVal::I32(i) => XDataValue::Integer32(*i),
        XVal::Real(r) => XDataValue::Real(*r),
        XVal::Str(s) => XDataValue::String(s.clone()),
        XVal::Pt(p) => XDataValue::Point3D(Vector3::new(p[0], p[1], 0.0)),
    }
}

pub fn from_xdata(v: &XDataValue) -> Option<XVal> {
    Some(match v {
        XDataValue::Integer16(i) => XVal::I16(*i),
        XDataValue::Integer32(i) => XVal::I32(*i),
        XDataValue::Real(r) => XVal::Real(*r),
        XDataValue::String(s) => XVal::Str(s.clone()),
        XDataValue::Point3D(p) => XVal::Pt([p.x, p.y]),
        _ => return None,
    })
}

/// The XDATA record for a tag.
pub fn record(tag: &Tag) -> ExtendedDataRecord {
    let mut rec = ExtendedDataRecord::new(XDATA_APP);
    for v in tag.encode() {
        rec.add_value(to_xdata(&v));
    }
    rec
}

fn v2(p: &P) -> Vector2 {
    Vector2::new(p[0], p[1])
}

pub fn shape_entity(shape: &Shape, pattern: PatternFn) -> EntityType {
    match shape {
        Shape::Poly { pts, closed } => {
            let mut pl = LwPolyline::new();
            for p in pts {
                pl.add_point(v2(p));
            }
            if *closed {
                pl.close();
            }
            EntityType::LwPolyline(pl)
        }
        Shape::Line { a, b } => EntityType::Line(Line::from_points(
            Vector3::new(a[0], a[1], 0.0),
            Vector3::new(b[0], b[1], 0.0),
        )),
        Shape::Hatch {
            name,
            solid,
            scale,
            boundary,
        } => {
            let mut h = Hatch::new();
            h.is_solid = *solid;
            if !*solid {
                h.pattern = pattern(name, *scale).unwrap_or_else(|| HatchPattern::new(*name));
                h.pattern_scale = *scale;
                h.pattern_angle = 0.0;
                h.pattern_type = HatchPatternType::Predefined;
            }
            let verts: Vec<Vector2> = boundary.iter().map(v2).collect();
            let mut path = BoundaryPath::external();
            path.add_edge(BoundaryEdge::Polyline(PolylineEdge::new(verts, true)));
            h.paths.push(path);
            EntityType::Hatch(Box::new(h))
        }
    }
}

/// One primitive as an entity on `layer`, tagged with the wall.
pub fn to_entity(prim: &Prim, layer: &str, tag: &Tag, pattern: PatternFn) -> EntityType {
    let mut e = shape_entity(&prim.shape, pattern);
    let c = e.common_mut();
    c.layer = layer.to_owned();
    c.extended_data.upsert_record(record(tag));
    e
}

/// All entities of a build. The first carries the full definition (the
/// anchor); the others only the wall id, so a long wall is not stored N times.
pub fn to_entities(out: &Output, id: u32, def: &WallDef, pattern: PatternFn) -> Vec<EntityType> {
    out.prims
        .iter()
        .enumerate()
        .map(|(i, prim)| {
            let tag = Tag {
                id,
                role: prim.role,
                column: prim.column,
                def: (i == 0).then(|| def.clone()),
                origin: (i == 0).then(|| outline_centre(prim)).flatten(),
            };
            to_entity(prim, &def.params.layer(prim.role).name, &tag, pattern)
        })
        .collect()
}

/// Centre of a primitive's outline (the anchor is always a column or brick outline).
fn outline_centre(prim: &Prim) -> Option<P> {
    match &prim.shape {
        Shape::Poly { pts, .. } => centroid(pts),
        _ => None,
    }
}

/// How far an anchor entity has moved since its wall was built (a MOVE or
/// drag). Rotation and scaling are not tracked.
pub fn moved_by(anchor: &EntityType, tag: &Tag) -> Option<P> {
    let origin = tag.origin?;
    let EntityType::LwPolyline(pl) = anchor else {
        return None;
    };
    let pts: Vec<P> = pl
        .vertices
        .iter()
        .map(|v| [v.location.x, v.location.y])
        .collect();
    let now = centroid(&pts)?;
    Some([now[0] - origin[0], now[1] - origin[1]])
}

/// The wall tag of an entity, or `None` if it is not part of a wall. For a
/// column the centre is taken from its outline.
pub fn read_tag(e: &EntityType) -> Option<Tag> {
    let rec = e.common().extended_data.get_record(XDATA_APP)?;
    let vals: Vec<XVal> = rec.values.iter().filter_map(from_xdata).collect();
    if vals.len() != rec.values.len() {
        return None;
    }
    let mut tag = Tag::decode(&vals)?;
    if tag.role == Role::Column {
        if let (Some(b), EntityType::LwPolyline(pl)) = (tag.column.as_mut(), e) {
            let pts: Vec<P> = pl
                .vertices
                .iter()
                .map(|v| [v.location.x, v.location.y])
                .collect();
            b.c = centroid(&pts)?;
        }
    }
    Some(tag)
}
