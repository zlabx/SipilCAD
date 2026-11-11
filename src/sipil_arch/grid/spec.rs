//! A grid as text: coordinates and labels. Port of SipilBIM's `gridspec.py`
//! (the plugin in zlabx/FT-CES), so a grid typed in either program reads the
//! same. Pure: `std` only.
//!
//! **Coordinates** (millimetres) are numbers and `n*d`:
//! `0.00 2*3000.00` is lines at 0, 3000 and 6000; `0.00 3000 4500` is 0, 3000
//! and 7500. In X and Y the first number is where the first line is and every
//! other number is the distance from the line before it; `n*d` is `n` lines
//! each `d` after the one before. In Z (the levels) every number is an
//! elevation and `n*d` is `n` levels each `d` above the one before:
//! `0.00 6000.00 8000.00` is levels at 0, 6 and 8 m.
//!
//! **Labels** are one per line, separated by spaces (by commas when a label has
//! a space in it: `Level 1, Level 2`). Missing labels are made up: X lines get
//! letters (A B C...), Y lines numbers (1 2 3...), levels `Level 1`, `Level 2`.

/// Most lines in one list.
pub const MAX_LINES: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Lines at each X, running along Y (labelled A B C...).
    X,
    /// Lines at each Y, running along X (labelled 1 2 3...).
    Y,
    /// Levels: elevations.
    Z,
}

/// Where the bubble of a line is: at the low end, the high end, or both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Bubble {
    #[default]
    Start,
    End,
    Both,
}

impl Bubble {
    pub const ALL: [Bubble; 3] = [Bubble::Start, Bubble::End, Bubble::Both];

    pub fn code(self) -> i16 {
        match self {
            Bubble::Start => 0,
            Bubble::End => 1,
            Bubble::Both => 2,
        }
    }

    pub fn from_code(code: i16) -> Bubble {
        match code {
            1 => Bubble::End,
            2 => Bubble::Both,
            _ => Bubble::Start,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Bubble::Start => "Start",
            Bubble::End => "End",
            Bubble::Both => "Both",
        }
    }

    /// `(start, end)`: which ends of a line carry a bubble.
    pub fn sides(self) -> (bool, bool) {
        match self {
            Bubble::Start => (true, false),
            Bubble::End => (false, true),
            Bubble::Both => (true, true),
        }
    }
}

/// One line of a grid (or one level): its label and where it is, in mm.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: String,
    pub pos: f64,
}

fn tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| c.is_whitespace() || c == ';' || c == ',')
        .filter(|t| !t.is_empty())
        .collect()
}

/// `-?digits(.digits)?`, nothing else (no `+`, no exponent, no `.5`).
fn number(tok: &str) -> Option<f64> {
    let body = tok.strip_prefix('-').unwrap_or(tok);
    let (int, frac) = match body.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (body, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(int) || frac.is_some_and(|f| !digits(f)) {
        return None;
    }
    tok.parse().ok()
}

/// `n*d` with `n` a whole number and `d` a number.
fn steps(tok: &str) -> Option<(u32, f64)> {
    let (n, d) = tok.split_once('*')?;
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((n.parse().ok()?, number(d)?))
}

/// The positions (mm, increasing) the text says.
pub fn parse_coordinates(text: &str, axis: Axis) -> Result<Vec<f64>, String> {
    let absolute = axis == Axis::Z;
    let mut out: Vec<f64> = Vec::new();
    for tok in tokens(text) {
        if let Some((n, d)) = steps(tok) {
            if n < 1 {
                return Err(format!("\u{ab}{tok}\u{bb}: the count must be 1 or more."));
            }
            if out.is_empty() {
                out.push(0.0); // `3*3000` alone starts at 0
            }
            for _ in 0..n {
                let last = out[out.len() - 1];
                out.push(last + d);
                if out.len() > MAX_LINES {
                    return Err(format!("At most {MAX_LINES} lines."));
                }
            }
        } else if let Some(v) = number(tok) {
            match out.last() {
                Some(last) if !absolute => out.push(last + v),
                _ => out.push(v),
            }
        } else {
            return Err(format!("\u{ab}{tok}\u{bb} is not a number or n*distance."));
        }
        if out.len() > MAX_LINES {
            return Err(format!("At most {MAX_LINES} lines."));
        }
    }
    for w in out.windows(2) {
        if w[1] <= w[0] + 1e-9 {
            return Err(if absolute {
                format!(
                    "The levels must go up: \u{ab}{:.2}\u{bb} is not above \u{ab}{:.2}\u{bb}.",
                    w[1], w[0]
                )
            } else {
                "The lines must go up: a distance of 0 or less puts a line on or before the one it follows."
                    .to_string()
            });
        }
    }
    Ok(out)
}

/// The shortest text that gives `positions` back (equal steps become `n*d`).
pub fn format_coordinates(positions: &[f64], axis: Axis) -> String {
    let Some(first) = positions.first() else {
        return String::new();
    };
    let mut out = vec![format!("{first:.2}")];
    let steps: Vec<f64> = positions.windows(2).map(|w| w[1] - w[0]).collect();
    let mut i = 0;
    while i < steps.len() {
        let mut n = 1;
        while i + n < steps.len() && (steps[i + n] - steps[i]).abs() < 1e-6 {
            n += 1;
        }
        if n >= 2 {
            out.push(format!("{n}*{:.2}", steps[i]));
        } else if axis == Axis::Z {
            out.push(format!("{:.2}", positions[i + 1]));
        } else {
            out.push(format!("{:.2}", steps[i]));
        }
        i += n;
    }
    out.join(" ")
}

/// 0 -> A, 25 -> Z, 26 -> AA, 27 -> AB...
pub fn letters(index: usize) -> String {
    let mut out = Vec::new();
    let mut n = index + 1;
    while n > 0 {
        let r = (n - 1) % 26;
        out.push(b'A' + r as u8);
        n = (n - 1) / 26;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

pub fn auto_label(axis: Axis, index: usize) -> String {
    match axis {
        Axis::X => letters(index),
        Axis::Y => (index + 1).to_string(),
        Axis::Z => format!("Level {}", index + 1),
    }
}

/// `count` labels: those in the text, then made-up ones. Two equal labels are
/// an error.
pub fn parse_labels(text: &str, count: usize, axis: Axis) -> Result<Vec<String>, String> {
    let raw = text.trim();
    let given: Vec<String> = if raw.contains(',') {
        raw.split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect()
    } else {
        tokens(raw).into_iter().map(str::to_string).collect()
    };
    let mut labels: Vec<String> = given.into_iter().take(count).collect();
    for (i, a) in labels.iter().enumerate() {
        if labels[..i].contains(a) {
            return Err(format!("The label \u{ab}{a}\u{bb} is used twice."));
        }
    }
    let mut i = labels.len();
    while labels.len() < count {
        let candidate = auto_label(axis, i);
        i += 1;
        if !labels.contains(&candidate) {
            labels.push(candidate);
        }
    }
    Ok(labels)
}

pub fn format_labels(labels: &[String]) -> String {
    let sep = if labels.iter().any(|l| l.contains(' ')) {
        ", "
    } else {
        " "
    };
    labels.join(sep)
}

/// Rows from the two texts.
pub fn rows(coordinates: &str, labels: &str, axis: Axis) -> Result<Vec<Row>, String> {
    let positions = parse_coordinates(coordinates, axis)?;
    let names = parse_labels(labels, positions.len(), axis)?;
    Ok(names
        .into_iter()
        .zip(positions)
        .map(|(id, pos)| Row { id, pos })
        .collect())
}

/// `(coordinates text, labels text)` of the rows.
pub fn texts_from_rows(rows: &[Row], axis: Axis) -> (String, String) {
    let positions: Vec<f64> = rows.iter().map(|r| r.pos).collect();
    let labels: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
    (format_coordinates(&positions, axis), format_labels(&labels))
}
