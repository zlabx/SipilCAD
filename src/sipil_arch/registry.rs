//! The component registry: one row per building component of the Architecture tab.
//!
//! A component is a self-contained folder (`src/sipil_arch/<id>/`) that a team
//! member can build alone. It becomes part of the app through exactly three
//! one-line entries, so two people adding two components touch different
//! lines of the same three files and do not collide:
//!
//! 1. `mod <id>;` in `sipil_arch/mod.rs` (and its pure files in the scratch
//!    crate's `lib.rs`, see `rebrand/opencadstudio/COMPONENTS.md`),
//! 2. one `<id>::INFO` row in [`COMPONENTS`] below (this file: the ribbon,
//!    the command names and the XDATA application names come from it),
//! 3. one `Box::new(<id>::host::Component)` row in `app/sipil_arch/mod.rs`.
//!
//! This file is pure (no iced, no host types) so the checks run in the
//! scratch crate with `cargo test`.

/// What the rest of the app needs to know about a component without loading it.
#[derive(Clone, Copy, Debug)]
pub struct ComponentInfo {
    /// Lower-case identifier, also the folder name (`door`).
    pub id: &'static str,
    /// Ribbon label of the component's button (`Door`).
    pub label: &'static str,
    /// Command the button runs (`DOOR`); upper case.
    pub command: &'static str,
    /// SVG of the ribbon button (24x24, the kit's two-colour icon style).
    pub icon: &'static [u8],
    /// XDATA application name its entities carry (`SIPILDOOR`).
    pub xdata_app: &'static str,
    /// One line for the component guide and error messages.
    pub summary: &'static str,
}

/// Every component, in ribbon order. Add yours here.
pub const COMPONENTS: &[ComponentInfo] = &[super::door::INFO, super::column::INFO, super::beam::INFO];

/// Commands and XDATA names the wall tool already owns.
const RESERVED_COMMANDS: &[&str] = &["WALL", "BW", "WALLPOPUP"];
const RESERVED_XDATA: &[&str] = &["SIPILWALL"];

/// Find the component that runs `command` (case-insensitive, `_`/`.` prefix allowed).
pub fn by_command(command: &str) -> Option<&'static ComponentInfo> {
    let c = command.trim().trim_start_matches(['_', '.']);
    COMPONENTS.iter().find(|i| i.command.eq_ignore_ascii_case(c))
}

/// Check a registry for the mistakes two people adding components at the same
/// time can make: a repeated id, command or XDATA name, or a malformed one.
pub fn validate(list: &[ComponentInfo]) -> Result<(), String> {
    let ident = |s: &str, upper: bool| {
        !s.is_empty()
            && s.chars().all(|c| {
                c.is_ascii_digit() || (if upper { c.is_ascii_uppercase() } else { c.is_ascii_lowercase() })
            })
            && !s.chars().next().is_some_and(|c| c.is_ascii_digit())
    };
    for (i, c) in list.iter().enumerate() {
        if !ident(c.id, false) {
            return Err(format!("component id '{}' must be lower-case letters and digits", c.id));
        }
        if !ident(c.command, true) {
            return Err(format!("{}: command '{}' must be upper-case letters and digits", c.id, c.command));
        }
        if !c.xdata_app.starts_with("SIPIL") || !ident(c.xdata_app, true) {
            return Err(format!("{}: XDATA name '{}' must be SIPIL plus upper-case letters", c.id, c.xdata_app));
        }
        if c.label.trim().is_empty() || c.summary.trim().is_empty() || c.icon.is_empty() {
            return Err(format!("{}: label, summary and icon are required", c.id));
        }
        if RESERVED_COMMANDS.contains(&c.command) || RESERVED_XDATA.contains(&c.xdata_app) {
            return Err(format!("{}: '{}' / '{}' belong to the wall tool", c.id, c.command, c.xdata_app));
        }
        for o in &list[..i] {
            if o.id == c.id || o.command == c.command || o.xdata_app == c.xdata_app {
                return Err(format!("{} and {} share an id, command or XDATA name", o.id, c.id));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_is_valid() {
        validate(COMPONENTS).unwrap();
        assert!(by_command("_door").is_some() && by_command("DOOR").is_some());
        assert!(by_command("WALL").is_none());
    }

    fn info(id: &'static str, command: &'static str, xdata: &'static str) -> ComponentInfo {
        ComponentInfo {
            id,
            label: "X",
            command,
            icon: b"<svg/>",
            xdata_app: xdata,
            summary: "x",
        }
    }

    #[test]
    fn clashes_and_bad_names_are_caught() {
        let a = info("window", "WINDOW", "SIPILWINDOW");
        assert!(validate(&[a]).is_ok());
        assert!(validate(&[a, a]).unwrap_err().contains("share"));
        assert!(validate(&[a, info("win2", "WINDOW", "SIPILWIN2")]).is_err());
        assert!(validate(&[a, info("win2", "WIN2", "SIPILWINDOW")]).is_err());
        assert!(validate(&[info("Window", "WINDOW", "SIPILWINDOW")]).is_err());
        assert!(validate(&[info("window", "window", "SIPILWINDOW")]).is_err());
        assert!(validate(&[info("window", "WINDOW", "WINDOW")]).is_err());
        assert!(validate(&[info("window", "WALL", "SIPILWINDOW")]).unwrap_err().contains("wall tool"));
        assert!(validate(&[info("window", "WINDOW", "SIPILWALL")]).is_err());
    }
}
