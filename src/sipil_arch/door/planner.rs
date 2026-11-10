//! Door logic on a model of the drawing: where a door goes, how the wall is
//! rebuilt around it, and which detail sheets must change.
//!
//! The planner never touches the drawing. It reads a [`Model`] (the walls,
//! doors and sheets found by the adapter's scan) and returns an [`Edit`]: the
//! entity handles to delete and the pieces to add, which the host applies as a
//! single undo step.

use codec::Handle;

use super::catalog::{self, DoorType};
use super::model::{DPrim, DoorDef};
use super::plan::{self, Placement};
use super::sheet;
use crate::sipil_wall::engine::{self, ColumnBox, Opening, Output, WallDef, P};

#[derive(Clone, Debug)]
pub struct WallRec {
    pub id: u32,
    /// The wall as it is now in the drawing (moves already applied).
    pub def: WallDef,
    pub handles: Vec<Handle>,
}

#[derive(Clone, Debug)]
pub struct DoorRec {
    pub def: DoorDef,
    pub handles: Vec<Handle>,
}

#[derive(Clone, Debug)]
pub struct SheetRec {
    pub code: String,
    pub origin: P,
    pub handles: Vec<Handle>,
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub walls: Vec<WallRec>,
    /// Every column in the drawing with the id of its wall.
    pub columns: Vec<(u32, ColumnBox)>,
    pub doors: Vec<DoorRec>,
    pub sheets: Vec<SheetRec>,
}

#[derive(Clone, Debug)]
pub enum Piece {
    Wall { id: u32, def: WallDef, out: Output },
    Door { def: DoorDef, prims: Vec<DPrim> },
    Sheet { code: String, origin: P, prims: Vec<DPrim> },
}

#[derive(Clone, Debug, Default)]
pub struct Edit {
    pub delete: Vec<Handle>,
    pub add: Vec<Piece>,
}

/// The closest wall run to a point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub wall: u32,
    pub seg: usize,
    /// Where the point projects on the run, from its start, drawing units
    /// (not clamped to the run).
    pub pos_du: f64,
    pub len: f64,
    /// The point is to the left of the run's direction.
    pub left: bool,
}

fn type_of(def: &DoorDef) -> &'static DoorType {
    catalog::find(&def.code).unwrap_or_else(catalog::fallback)
}

/// Wall thickness used when no wall is at hand: 10 cm brick, 13 cm with plaster.
const DEFAULT_T: f64 = 0.10;
const DEFAULT_TOT: f64 = 0.13;

impl Model {
    fn wall(&self, id: u32) -> Option<&WallRec> {
        self.walls.iter().find(|w| w.id == id)
    }

    fn next_door_id(&self) -> u32 {
        self.doors.iter().map(|d| d.def.id).max().unwrap_or(0) + 1
    }

    fn door_defs(&self) -> Vec<DoorDef> {
        self.doors.iter().map(|d| d.def.clone()).collect()
    }

    /// How many doors of each type; used for the sheets' counts.
    pub fn count_of(&self, code: &str) -> usize {
        self.doors.iter().filter(|d| d.def.code == code).count()
    }

    /// The wall run nearest to `p`, if the point is on or just beside a wall.
    pub fn locate(&self, p: P) -> Option<Hit> {
        let mut best: Option<(f64, Hit)> = None;
        for w in &self.walls {
            let Ok((pts, closed)) = engine::axis(&w.def) else { continue };
            let upm = if w.def.upm > 0.0 { w.def.upm } else { 1000.0 };
            let reach = w.def.params.sanitized().total_m() * upm / 2.0 + 0.3 * upm;
            for (seg, (a, b, ang)) in engine::runs(&pts, closed).into_iter().enumerate() {
                let (u, n) = ((ang.cos(), ang.sin()), (-ang.sin(), ang.cos()));
                let (dx, dy) = (p[0] - a[0], p[1] - a[1]);
                let (s, d) = (dx * u.0 + dy * u.1, dx * n.0 + dy * n.1);
                let len = (b[0] - a[0]).hypot(b[1] - a[1]);
                let off = (s - s.clamp(0.0, len)).hypot(d);
                if off <= reach && best.as_ref().map_or(true, |(o, _)| off < *o) {
                    best = Some((off, Hit { wall: w.id, seg, pos_du: s, len, left: d >= 0.0 }));
                }
            }
        }
        best.map(|(_, h)| h)
    }

    /// The frame-to-frame half width of the opening and the margin kept to
    /// the columns at the ends of the run, drawing units.
    fn reach(def: &DoorDef, wall: &WallRec) -> (f64, f64, f64) {
        let upm = if wall.def.upm > 0.0 { wall.def.upm } else { 1000.0 };
        let half = plan::opening_half(def, type_of(def), upm);
        let tot = wall.def.params.sanitized().total_m() * upm;
        (half, tot / 2.0, upm)
    }

    /// The door's definition at the wall under `p`: position clamped so the
    /// opening stays clear of the columns at the ends of the run.
    fn at(&self, template: &DoorDef, p: P) -> Result<DoorDef, String> {
        let hit = self.locate(p).ok_or("Click on a wall")?;
        let wall = self.wall(hit.wall).ok_or("Click on a wall")?;
        let mut def = template.sanitized();
        let (half, margin, upm) = Self::reach(&def, wall);
        let (lo, hi) = (half + margin, hit.len - half - margin);
        if hi < lo {
            return Err("This stretch of wall is too short for the door".into());
        }
        def.wall = hit.wall;
        def.seg = hit.seg as u32;
        def.pos_m = hit.pos_du.clamp(lo, hi) / upm;
        def.swing_left = hit.left;
        Ok(def)
    }

    /// Check that `def` fits its run and does not overlap another door.
    pub fn fit(&self, def: &DoorDef) -> Result<(), String> {
        let wall = self.wall(def.wall).ok_or("The wall of this door is no longer in the drawing")?;
        let (pts, closed) = engine::axis(&wall.def)?;
        let run = engine::runs(&pts, closed)
            .get(def.seg as usize)
            .copied()
            .ok_or("The wall no longer has this stretch")?;
        let (half, margin, upm) = Self::reach(def, wall);
        let len = (run.1[0] - run.0[0]).hypot(run.1[1] - run.0[1]);
        let c = def.pos_m * upm;
        if c - half < margin - 1e-6 || c + half > len - margin + 1e-6 {
            return Err("The door does not fit in this stretch of wall".into());
        }
        for o in self.doors.iter().filter(|o| o.def.id != def.id && o.def.wall == def.wall && o.def.seg == def.seg) {
            let (oh, _, _) = Self::reach(&o.def, wall);
            if (o.def.pos_m * upm - c).abs() < half + oh - 1e-6 {
                return Err("The door overlaps another door".into());
            }
        }
        Ok(())
    }

    /// The plan symbol of a door under the cursor, or `None` while the cursor
    /// is not on a wall where the door fits.
    pub fn preview(&self, template: &DoorDef, p: P) -> Option<Vec<DPrim>> {
        let mut def = self.at(template, p).ok()?;
        def.id = self.next_door_id();
        self.fit(&def).ok()?;
        let (pl, _) = self.placement(&def)?;
        Some(plan::plan(&def, type_of(&def), &pl))
    }

    /// Where `def` sits on its wall.
    fn placement(&self, def: &DoorDef) -> Option<(Placement, &WallRec)> {
        let wall = self.wall(def.wall)?;
        let (pts, closed) = engine::axis(&wall.def).ok()?;
        let (a, _, ang) = *engine::runs(&pts, closed).get(def.seg as usize)?;
        let upm = if wall.def.upm > 0.0 { wall.def.upm } else { 1000.0 };
        let s = def.pos_m * upm;
        let c = [a[0] + s * ang.cos(), a[1] + s * ang.sin()];
        let tot = wall.def.params.sanitized().total_m() * upm;
        Some((Placement { c, ang, tot, upm }, wall))
    }

    /// Put a door of `template`'s type at the wall under `p`.
    pub fn place(&self, template: &DoorDef, p: P) -> Result<(Edit, DoorDef), String> {
        let mut def = self.at(template, p)?;
        def.id = self.next_door_id();
        self.fit(&def)?;
        let mut all = self.door_defs();
        all.push(def.clone());
        let edit = self.compose(def.wall, None, &all, &[def.code.clone()])?;
        Ok((edit, def))
    }

    /// Change a door that is in the drawing (its position stays).
    pub fn change_door(&self, new: &DoorDef) -> Result<Edit, String> {
        let old = self
            .doors
            .iter()
            .find(|d| d.def.id == new.id)
            .ok_or("This door is no longer in the drawing")?;
        let mut def = new.sanitized();
        def.wall = old.def.wall;
        def.seg = old.def.seg;
        def.pos_m = old.def.pos_m;
        self.fit(&def)?;
        let all: Vec<DoorDef> = self
            .door_defs()
            .into_iter()
            .map(|d| if d.id == def.id { def.clone() } else { d })
            .collect();
        self.compose(def.wall, None, &all, &[old.def.code.clone(), def.code.clone()])
    }

    /// Rebuild a wall with new parameters, keeping its doors in their openings.
    pub fn rebuild_wall(&self, wall: u32, new_def: WallDef) -> Result<Edit, String> {
        let all = self.door_defs();
        let codes: Vec<String> = all.iter().filter(|d| d.wall == wall).map(|d| d.code.clone()).collect();
        self.compose(wall, Some(new_def), &all, &codes)
    }

    /// Bring the model up to date after `edit` was applied to the drawing, so
    /// a command can go on placing doors without reading the drawing again.
    /// `counts[i]` is the number of entities of `edit.add[i]` and
    /// `new_handles` the handles they were given, in order.
    pub fn apply(&mut self, edit: &Edit, counts: &[usize], new_handles: &[Handle]) {
        let gone = |hs: &[Handle]| hs.iter().any(|h| edit.delete.contains(h));
        let walls_gone: Vec<u32> = self.walls.iter().filter(|w| gone(&w.handles)).map(|w| w.id).collect();
        self.walls.retain(|w| !walls_gone.contains(&w.id));
        self.columns.retain(|(w, _)| !walls_gone.contains(w));
        self.doors.retain(|d| !gone(&d.handles));
        self.sheets.retain(|s| !gone(&s.handles));
        let mut at = 0;
        for (piece, n) in edit.add.iter().zip(counts) {
            let end = (at + n).min(new_handles.len());
            let handles = new_handles[at.min(end)..end].to_vec();
            at += n;
            match piece {
                Piece::Wall { id, def, out } => {
                    self.walls.retain(|w| w.id != *id);
                    self.columns.retain(|(w, _)| w != id);
                    self.columns.extend(out.new_columns.iter().map(|b| (*id, *b)));
                    self.walls.push(WallRec { id: *id, def: def.clone(), handles });
                }
                Piece::Door { def, .. } => {
                    self.doors.retain(|d| d.def.id != def.id);
                    self.doors.push(DoorRec { def: def.clone(), handles });
                }
                Piece::Sheet { code, origin, .. } => {
                    self.sheets.retain(|s| &s.code != code);
                    self.sheets.push(SheetRec { code: code.clone(), origin: *origin, handles });
                }
            }
        }
    }

    /// Wall + its doors + the sheets of `codes`, given every door as it will be.
    fn compose(&self, wall_id: u32, new_def: Option<WallDef>, all: &[DoorDef], codes: &[String]) -> Result<Edit, String> {
        let rec = self.wall(wall_id).ok_or("The wall is no longer in the drawing")?;
        let def = new_def.unwrap_or_else(|| rec.def.clone());
        let upm = if def.upm > 0.0 { def.upm } else { 1000.0 };
        let here: Vec<&DoorDef> = all.iter().filter(|d| d.wall == wall_id).collect();

        let openings: Vec<Opening> = here
            .iter()
            .map(|d| {
                let half = plan::opening_half(d, type_of(d), upm);
                let c = d.pos_m * upm;
                Opening { seg: d.seg as usize, from: c - half, to: c + half }
            })
            .collect();
        let others: Vec<ColumnBox> = self.columns.iter().filter(|(w, _)| *w != wall_id).map(|(_, b)| *b).collect();
        let out = engine::build_with(&def, &others, &openings)?;

        let mut edit = Edit::default();
        edit.delete.extend(rec.handles.iter().copied());
        for d in self.doors.iter().filter(|d| d.def.wall == wall_id) {
            edit.delete.extend(d.handles.iter().copied());
        }
        edit.add.push(Piece::Wall { id: wall_id, def: def.clone(), out });

        let (pts, closed) = engine::axis(&def)?;
        let runs = engine::runs(&pts, closed);
        let tot = def.params.sanitized().total_m() * upm;
        for d in here {
            let (a, _, ang) = *runs.get(d.seg as usize).ok_or("The wall no longer has this stretch")?;
            let s = d.pos_m * upm;
            let c = [a[0] + s * ang.cos(), a[1] + s * ang.sin()];
            let prims = plan::plan(d, type_of(d), &Placement { c, ang, tot, upm });
            edit.add.push(Piece::Door { def: d.clone(), prims });
        }

        // Sheets: from the wall as it will be (a rebuilt wall may be thicker).
        let mut model = self.clone();
        if let Some(w) = model.walls.iter_mut().find(|w| w.id == wall_id) {
            w.def = def;
        }
        let (del, add) = model.sheets_for(all, codes);
        edit.delete.extend(del);
        edit.add.extend(add);
        Ok(edit)
    }

    /// Thickness of the wall the first door of `code` is in (brick, total), metres.
    fn wall_thickness(&self, code: &str, all: &[DoorDef]) -> (f64, f64, f64) {
        all.iter()
            .find(|d| d.code == code)
            .and_then(|d| self.wall(d.wall))
            .map(|w| {
                let p = w.def.params.sanitized();
                (p.thickness_m, p.total_m(), if w.def.upm > 0.0 { w.def.upm } else { 1000.0 })
            })
            .unwrap_or((DEFAULT_T, DEFAULT_TOT, self.walls.first().map_or(1000.0, |w| w.def.upm.max(1e-9))))
    }

    /// Lower-left corner for a new sheet: under the plan, right of the sheets there.
    fn new_origin(&self, placed: &[(String, P)], upm: f64, h_cm: f64) -> P {
        let (mut min_x, mut min_y) = (f64::MAX, f64::MAX);
        for w in &self.walls {
            let reach = w.def.params.sanitized().total_m() * w.def.upm.max(1e-9);
            for p in &w.def.pts {
                min_x = min_x.min(p[0] - reach);
                min_y = min_y.min(p[1] - reach);
            }
        }
        if !min_x.is_finite() {
            (min_x, min_y) = (0.0, 0.0);
        }
        let k = upm / 100.0;
        let mut x = min_x;
        let all_sheets = self.sheets.iter().map(|s| (s.code.clone(), s.origin)).chain(placed.iter().cloned());
        for (code, o) in all_sheets {
            let ty = catalog::find(&code).unwrap_or_else(catalog::fallback);
            let (w_cm, _) = sheet::size_cm(ty, DEFAULT_T, DEFAULT_TOT);
            x = x.max(o[0] + (w_cm + sheet::GAP_CM) * k);
        }
        [x, min_y - (h_cm + sheet::GAP_CM) * k]
    }

    /// Delete and re-add the sheets of `codes` for the doors in `all`.
    fn sheets_for(&self, all: &[DoorDef], codes: &[String]) -> (Vec<Handle>, Vec<Piece>) {
        let mut seen: Vec<&String> = Vec::new();
        let (mut del, mut add) = (Vec::new(), Vec::new());
        let mut placed: Vec<(String, P)> = Vec::new();
        for code in codes {
            if seen.contains(&code) {
                continue;
            }
            seen.push(code);
            let existing = self.sheets.iter().find(|s| &s.code == code);
            if let Some(s) = existing {
                del.extend(s.handles.iter().copied());
            }
            let count = all.iter().filter(|d| &d.code == code).count();
            if count == 0 {
                continue;
            }
            let ty = catalog::find(code).unwrap_or_else(catalog::fallback);
            let (t, tot, upm) = self.wall_thickness(code, all);
            let origin = match existing {
                Some(s) => s.origin,
                None => {
                    let (_, h) = sheet::size_cm(ty, t, tot);
                    self.new_origin(&placed, upm, h)
                }
            };
            placed.push((code.clone(), origin));
            let prims = sheet::sheet(ty, count, origin, upm, t, tot);
            add.push(Piece::Sheet { code: code.clone(), origin, prims });
        }
        (del, add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sipil_wall::engine::{build, WallDef, WallParams};

    /// One straight wall 6 m long along x, with handles 1..=n.
    fn model() -> Model {
        let def = WallDef { pts: vec![[0.0, 0.0], [6000.0, 0.0]], closed: false, params: WallParams::default(), upm: 1000.0 };
        let out = build(&def, &[]).unwrap();
        let handles: Vec<Handle> = (1..=out.prims.len() as u64).map(Handle::new).collect();
        Model {
            walls: vec![WallRec { id: 1, def, handles }],
            columns: out.new_columns.iter().map(|b| (1, *b)).collect(),
            ..Model::default()
        }
    }

    fn tpl(code: &str) -> DoorDef {
        DoorDef::of_type(catalog::find(code).unwrap())
    }

    #[test]
    fn locate_finds_the_run_and_the_side() {
        let m = model();
        let h = m.locate([2000.0, 90.0]).unwrap();
        assert_eq!((h.wall, h.seg, h.left), (1, 0, true));
        assert!((h.pos_du - 2000.0).abs() < 1e-9 && (h.len - 6000.0).abs() < 1e-9);
        assert!(!m.locate([2000.0, -90.0]).unwrap().left);
        assert!(m.locate([2000.0, 2000.0]).is_none());
    }

    #[test]
    fn placing_cuts_the_wall_adds_the_door_and_a_sheet() {
        let m = model();
        let (edit, def) = m.place(&tpl("P2"), [3000.0, 90.0]).unwrap();
        assert_eq!((def.id, def.wall, def.seg), (1, 1, 0));
        assert!((def.pos_m - 3.0).abs() < 1e-9 && def.swing_left);
        // The old wall goes, a new wall (two pieces), the door and the sheet come.
        assert_eq!(edit.delete.len(), m.walls[0].handles.len());
        let kinds: Vec<&str> = edit
            .add
            .iter()
            .map(|p| match p {
                Piece::Wall { .. } => "wall",
                Piece::Door { .. } => "door",
                Piece::Sheet { .. } => "sheet",
            })
            .collect();
        assert_eq!(kinds, ["wall", "door", "sheet"]);
        match &edit.add[0] {
            Piece::Wall { out, .. } => assert_eq!(out.n_pieces, 2),
            _ => unreachable!(),
        }
    }

    #[test]
    fn the_sheet_goes_below_the_plan_and_new_types_go_beside_it() {
        let mut m = model();
        let (edit, _) = m.place(&tpl("P2"), [2000.0, 90.0]).unwrap();
        let (code, origin) = match edit.add.last().unwrap() {
            Piece::Sheet { code, origin, .. } => (code.clone(), *origin),
            _ => panic!(),
        };
        assert_eq!(code, "P2");
        assert!(origin[1] < -65.0 * 1000.0 / 100.0 * 0.0 - 1.0, "below the wall: {origin:?}");
        assert!(origin[0] <= 0.0 + 1e-9);
        // A second type is placed to the right of the first sheet.
        m.sheets.push(SheetRec { code, origin, handles: vec![Handle::new(900)] });
        let (edit, _) = m.place(&tpl("P1"), [4500.0, 90.0]).unwrap();
        let o2 = match edit.add.last().unwrap() {
            Piece::Sheet { origin, .. } => *origin,
            _ => panic!(),
        };
        let (w_cm, _) = sheet::size_cm(catalog::find("P2").unwrap(), 0.10, 0.13);
        assert!(o2[0] >= origin[0] + w_cm * 10.0, "{o2:?} vs {origin:?}");
        assert!((o2[1] - origin[1]).abs() < 1e-6 || o2[1] < 0.0);
    }

    #[test]
    fn existing_sheet_is_replaced_in_place_with_the_new_count() {
        let mut m = model();
        let (edit, def) = m.place(&tpl("P2"), [1500.0, 90.0]).unwrap();
        let origin = match edit.add.last().unwrap() {
            Piece::Sheet { origin, .. } => *origin,
            _ => panic!(),
        };
        m.doors.push(DoorRec { def, handles: vec![Handle::new(500)] });
        m.sheets.push(SheetRec { code: "P2".into(), origin, handles: vec![Handle::new(700), Handle::new(701)] });
        let (edit, def2) = m.place(&tpl("P2"), [4500.0, 90.0]).unwrap();
        assert_eq!(def2.id, 2);
        assert!(edit.delete.contains(&Handle::new(700)) && edit.delete.contains(&Handle::new(500)));
        match edit.add.last().unwrap() {
            Piece::Sheet { origin: o, prims, .. } => {
                assert_eq!(*o, origin);
                let has = prims.iter().any(|p| matches!(&p.shape, super::super::model::DShape::Text { text, .. } if text == "P2  =  2  buah"));
                assert!(has, "count 2 on the sheet");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn doors_that_do_not_fit_are_refused() {
        let mut m = model();
        // Too close to the end column.
        let (_, d) = m.place(&tpl("P2"), [10.0, 90.0]).unwrap();
        assert!(d.pos_m * 1000.0 > 450.0 + 65.0 - 1e-6, "clamped away from the column: {}", d.pos_m);
        // Overlap.
        let (_, d) = m.place(&tpl("P2"), [3000.0, 90.0]).unwrap();
        m.doors.push(DoorRec { def: d, handles: vec![] });
        assert!(m.place(&tpl("P1"), [3300.0, 90.0]).unwrap_err().contains("overlaps"));
        // Not on a wall.
        assert!(m.place(&tpl("P1"), [3000.0, 5000.0]).unwrap_err().contains("wall"));
        // A double door in a short stub.
        let short = WallDef { pts: vec![[0.0, 2000.0], [1200.0, 2000.0]], closed: false, params: WallParams::default(), upm: 1000.0 };
        m.walls.push(WallRec { id: 2, def: short, handles: vec![] });
        assert!(m.place(&tpl("P3"), [600.0, 2090.0]).unwrap_err().contains("too short"));
    }

    #[test]
    fn changing_a_door_updates_both_sheets() {
        let mut m = model();
        let (_, d) = m.place(&tpl("P2"), [3000.0, 90.0]).unwrap();
        m.doors.push(DoorRec { def: d.clone(), handles: vec![Handle::new(500)] });
        m.sheets.push(SheetRec { code: "P2".into(), origin: [0.0, -9000.0], handles: vec![Handle::new(700)] });
        let mut ch = d.clone();
        ch.code = "P1".into();
        ch.open = false;
        let edit = m.change_door(&ch).unwrap();
        assert!(edit.delete.contains(&Handle::new(700)), "old P2 sheet goes (no doors left)");
        let sheets: Vec<&str> = edit
            .add
            .iter()
            .filter_map(|p| match p {
                Piece::Sheet { code, .. } => Some(code.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(sheets, ["P1"]);
        // Position is kept whatever the form said.
        let mut moved = ch.clone();
        moved.pos_m = 99.0;
        assert!(m.change_door(&moved).is_ok());
    }

    #[test]
    fn rebuilding_a_wall_keeps_its_doors_in_their_openings() {
        let mut m = model();
        let (_, d) = m.place(&tpl("P2"), [3000.0, 90.0]).unwrap();
        m.doors.push(DoorRec { def: d, handles: vec![Handle::new(500)] });
        let mut thick = m.walls[0].def.clone();
        thick.params.thickness_m = 0.2;
        let edit = m.rebuild_wall(1, thick).unwrap();
        assert!(edit.delete.contains(&Handle::new(500)));
        let n_doors = edit.add.iter().filter(|p| matches!(p, Piece::Door { .. })).count();
        assert_eq!(n_doors, 1);
        match &edit.add[0] {
            Piece::Wall { out, .. } => assert_eq!(out.n_pieces, 2),
            _ => panic!(),
        }
    }

    #[test]
    fn apply_keeps_the_model_in_step_with_the_drawing() {
        let mut m = model();
        let mut next = 1000u64;
        let mut put = |m: &mut Model, code: &str, at: f64| {
            let (edit, _) = m.place(&tpl(code), [at, 90.0]).unwrap();
            // Pretend each piece became one entity per primitive-ish count.
            let counts: Vec<usize> = edit
                .add
                .iter()
                .map(|p| match p {
                    Piece::Wall { out, .. } => out.prims.len(),
                    Piece::Door { prims, .. } | Piece::Sheet { prims, .. } => prims.len(),
                })
                .collect();
            let total: usize = counts.iter().sum();
            let handles: Vec<Handle> = (0..total as u64).map(|i| Handle::new(next + i)).collect();
            next += total as u64;
            m.apply(&edit, &counts, &handles);
        };
        put(&mut m, "P2", 1500.0);
        assert_eq!((m.walls.len(), m.doors.len(), m.sheets.len(), m.count_of("P2")), (1, 1, 1, 1));
        // The second door sees the first one's opening and sheet.
        put(&mut m, "P2", 4500.0);
        assert_eq!((m.walls.len(), m.doors.len(), m.sheets.len(), m.count_of("P2")), (1, 2, 1, 2));
        assert_eq!(m.doors[0].def.id, 1);
        assert_eq!(m.doors[1].def.id, 2);
        // Handles are the new ones, and the columns are still the wall's own.
        assert!(m.walls[0].handles.iter().all(|h| h.value() >= 1000));
        assert_eq!(m.columns.len(), 2);
        // The third overlaps nothing and is refused only where a door is.
        assert!(m.place(&tpl("P1"), [1600.0, 90.0]).is_err());
        put(&mut m, "P1", 3000.0);
        assert_eq!((m.doors.len(), m.sheets.len()), (3, 2));
    }

    #[test]
    fn preview_follows_the_cursor() {
        let m = model();
        assert!(m.preview(&tpl("P2"), [3000.0, 90.0]).is_some());
        assert!(m.preview(&tpl("P2"), [3000.0, 4000.0]).is_none());
    }
}
