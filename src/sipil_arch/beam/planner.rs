//! Beam logic on a model of the drawing: placing a beam between two points
//! and redrawing one that is there. The planner never touches the drawing; it
//! returns an [`Edit`] the host applies as one undo step.

use codec::Handle;

use super::model::BeamDef;
use super::plan;
use crate::sipil_arch::steel::draw::Prim;
use crate::sipil_wall::engine::P;

#[derive(Clone, Debug)]
pub struct BeamRec {
    /// The beam as it is now in the drawing (a MOVE already applied).
    pub def: BeamDef,
    pub handles: Vec<Handle>,
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub beams: Vec<BeamRec>,
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub def: BeamDef,
    pub prims: Vec<Prim>,
}

#[derive(Clone, Debug, Default)]
pub struct Edit {
    pub delete: Vec<Handle>,
    pub add: Vec<Piece>,
}

impl Model {
    pub fn next_id(&self) -> u32 {
        self.beams.iter().map(|b| b.def.id).max().unwrap_or(0) + 1
    }

    /// Put a beam of `template`'s settings from `a` to `b`.
    pub fn place(&self, template: &BeamDef, a: P, b: P) -> Result<(Edit, BeamDef), String> {
        let def = BeamDef { id: self.next_id(), a, b, ..template.sanitized() };
        if def.length() < 1e-6 {
            return Err("The beam needs two different points".into());
        }
        Ok((Edit { delete: Vec::new(), add: vec![Piece { prims: plan::prims(&def), def: def.clone() }] }, def))
    }

    /// Remember a beam the command just placed, so the next one gets the next number.
    pub fn remember(&mut self, def: BeamDef) {
        self.beams.push(BeamRec { def, handles: Vec::new() });
    }

    /// Redraw a beam that is in the drawing (its ends and number stay).
    pub fn change(&self, new: &BeamDef) -> Result<Edit, String> {
        let old = self.beams.iter().find(|b| b.def.id == new.id).ok_or("This beam is no longer in the drawing")?;
        let def = BeamDef { id: old.def.id, a: old.def.a, b: old.def.b, ..new.sanitized() };
        Ok(Edit { delete: old.handles.clone(), add: vec![Piece { prims: plan::prims(&def), def }] })
    }

    /// The beam from `a` to the cursor while placing.
    pub fn preview(&self, template: &BeamDef, a: P, b: P) -> Vec<Prim> {
        plan::prims(&BeamDef { a, b, ..template.sanitized() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sipil_arch::steel::catalog;

    fn tpl() -> BeamDef {
        BeamDef::new(catalog::default_profile(), 1000.0)
    }

    #[test]
    fn placing_numbers_the_beams() {
        let mut m = Model::default();
        let (edit, d) = m.place(&tpl(), [0.0, 0.0], [3000.0, 0.0]).unwrap();
        assert_eq!((d.id, edit.delete.len(), edit.add.len()), (1, 0, 1));
        m.remember(d);
        assert_eq!(m.place(&tpl(), [0.0, 0.0], [0.0, 3000.0]).unwrap().1.id, 2);
        assert!(m.place(&tpl(), [1.0, 1.0], [1.0, 1.0]).unwrap_err().contains("two different"));
    }

    #[test]
    fn changing_keeps_the_ends_and_the_number() {
        let mut m = Model::default();
        let (_, d) = m.place(&tpl(), [0.0, 0.0], [3000.0, 0.0]).unwrap();
        m.beams.push(BeamRec { def: d.clone(), handles: vec![Handle::new(4)] });
        let mut ch = d.clone();
        ch.a = [9.0, 9.0];
        ch.mark = "B7".into();
        let edit = m.change(&ch).unwrap();
        assert_eq!(edit.delete, vec![Handle::new(4)]);
        let new = &edit.add[0].def;
        assert_eq!((new.id, new.a, new.b, new.mark.as_str()), (1, [0.0, 0.0], [3000.0, 0.0], "B7"));
        ch.id = 5;
        assert!(m.change(&ch).is_err());
    }
}
