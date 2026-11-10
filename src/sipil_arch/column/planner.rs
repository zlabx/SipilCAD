//! Column logic on a model of the drawing: where a column goes and what an
//! edit changes. The planner never touches the drawing; it returns an [`Edit`]
//! the host applies as one undo step.

use codec::Handle;

use super::model::ColumnDef;
use super::plan;
use crate::sipil_arch::steel::draw::Prim;
use crate::sipil_wall::engine::P;

#[derive(Clone, Debug)]
pub struct ColumnRec {
    /// The column as it is now in the drawing (a MOVE already applied).
    pub def: ColumnDef,
    pub handles: Vec<Handle>,
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub columns: Vec<ColumnRec>,
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub def: ColumnDef,
    pub prims: Vec<Prim>,
}

#[derive(Clone, Debug, Default)]
pub struct Edit {
    pub delete: Vec<Handle>,
    pub add: Vec<Piece>,
}

impl Model {
    pub fn next_id(&self) -> u32 {
        self.columns.iter().map(|c| c.def.id).max().unwrap_or(0) + 1
    }

    /// `template` standing at `p`, with the next free number.
    pub fn at(&self, template: &ColumnDef, p: P) -> ColumnDef {
        ColumnDef { id: self.next_id(), at: p, ..template.sanitized() }
    }

    /// Put a column of `template`'s settings at `p`.
    pub fn place(&self, template: &ColumnDef, p: P) -> (Edit, ColumnDef) {
        let def = self.at(template, p);
        let edit = Edit { delete: Vec::new(), add: vec![Piece { prims: plan::prims(&def), def: def.clone() }] };
        (edit, def)
    }

    /// Remember a column the command just placed, so the next one gets the next number.
    pub fn remember(&mut self, def: ColumnDef) {
        self.columns.push(ColumnRec { def, handles: Vec::new() });
    }

    /// Redraw a column that is in the drawing (its place and number stay).
    pub fn change(&self, new: &ColumnDef) -> Result<Edit, String> {
        let old = self
            .columns
            .iter()
            .find(|c| c.def.id == new.id)
            .ok_or("This column is no longer in the drawing")?;
        let def = ColumnDef { id: old.def.id, at: old.def.at, ..new.sanitized() };
        Ok(Edit { delete: old.handles.clone(), add: vec![Piece { prims: plan::prims(&def), def }] })
    }

    /// The column under the cursor while placing.
    pub fn preview(&self, template: &ColumnDef, p: P) -> Vec<Prim> {
        plan::prims(&ColumnDef { at: p, ..template.sanitized() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sipil_arch::steel::catalog;

    fn tpl() -> ColumnDef {
        ColumnDef::new(catalog::default_profile(), 1000.0)
    }

    #[test]
    fn placing_numbers_the_columns_and_adds_pieces() {
        let mut m = Model::default();
        let (edit, d1) = m.place(&tpl(), [1000.0, 2000.0]);
        assert_eq!((d1.id, d1.at), (1, [1000.0, 2000.0]));
        assert!(edit.delete.is_empty() && edit.add.len() == 1);
        m.remember(d1);
        let (_, d2) = m.place(&tpl(), [4000.0, 2000.0]);
        assert_eq!(d2.id, 2);
    }

    #[test]
    fn changing_keeps_place_and_number_and_replaces_the_entities() {
        let mut m = Model::default();
        let (_, d) = m.place(&tpl(), [1000.0, 2000.0]);
        m.columns.push(ColumnRec { def: d.clone(), handles: vec![Handle::new(7), Handle::new(8)] });
        let mut ch = d.clone();
        ch.at = [9.0, 9.0]; // ignored: the column stays where it is
        ch.profile = catalog::key(catalog::all().iter().find(|p| p.name == "H350X350X12X19").unwrap());
        ch.rot_deg = 450.0;
        let edit = m.change(&ch).unwrap();
        assert_eq!(edit.delete, vec![Handle::new(7), Handle::new(8)]);
        let new = &edit.add[0].def;
        assert_eq!((new.id, new.at, new.rot_deg), (1, [1000.0, 2000.0], 90.0));
        assert!(new.profile.ends_with("H350X350X12X19"));
        ch.id = 99;
        assert!(m.change(&ch).is_err());
    }

    #[test]
    fn the_preview_follows_the_cursor() {
        let m = Model::default();
        let p = m.preview(&tpl(), [10.0, 20.0]);
        assert!(!p.is_empty());
    }
}
