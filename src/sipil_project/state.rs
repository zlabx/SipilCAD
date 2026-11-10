//! The New Project form and the naming rules. Pure (std only), tested alone in
//! test/projectcheck.

use std::path::Path;

use super::templates::{TemplateId, DEFAULT_TEMPLATE};

/// Characters a file or folder name cannot hold on Windows (the strictest of
/// the systems this runs on). A project name becomes a folder and a file name.
const INVALID: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// The longest project name the form accepts.
pub const MAX_NAME: usize = 80;

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectForm {
    pub name: String,
    pub template: TemplateId,
    /// Folder the project folder goes in (desktop only).
    pub folder: String,
    pub error: Option<String>,
}

impl ProjectForm {
    /// A form with the next free "Project N" name and the default template.
    pub fn new(existing: &[String], folder: String) -> Self {
        Self {
            name: next_project_name(existing),
            template: DEFAULT_TEMPLATE,
            folder,
            error: None,
        }
    }
}

/// The New Project form while its popup is open.
#[derive(Debug, Default, Clone)]
pub struct State {
    pub form: Option<ProjectForm>,
}

/// "Project 1", or the first "Project N" no open project already has.
pub fn next_project_name(existing: &[String]) -> String {
    (1..)
        .map(|n| format!("Project {n}"))
        .find(|name| !existing.iter().any(|e| e.eq_ignore_ascii_case(name)))
        .unwrap_or_else(|| "Project".to_string())
}

/// The trimmed name, or why it cannot be a project name.
pub fn validate_name(raw: &str) -> Result<String, String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err("Enter a project name.".to_string());
    }
    if name.chars().count() > MAX_NAME {
        return Err(format!(
            "A project name can have at most {MAX_NAME} characters."
        ));
    }
    if name.chars().any(|c| c.is_control() || INVALID.contains(&c)) {
        return Err("A project name cannot contain \\ / : * ? \" < > |".to_string());
    }
    Ok(name.to_string())
}

/// A name made safe to use as a file or folder name: invalid characters become
/// `-`, trailing dots and spaces go, a reserved Windows device name gets a `_`.
pub fn file_stem(raw: &str) -> String {
    let mapped: String = raw
        .chars()
        .map(|c| {
            if c.is_control() || INVALID.contains(&c) {
                '-'
            } else {
                c
            }
        })
        .collect();
    let cleaned = mapped.trim().trim_end_matches('.').trim_end().to_string();
    if cleaned.is_empty() {
        return "Untitled".to_string();
    }
    let upper = cleaned.to_ascii_uppercase();
    let base = upper.split('.').next().unwrap_or("");
    let numbered = (base.starts_with("COM") || base.starts_with("LPT"))
        && base.len() == 4
        && base.as_bytes()[3].is_ascii_digit();
    if matches!(base, "CON" | "PRN" | "AUX" | "NUL") || numbered {
        format!("{cleaned}_")
    } else {
        cleaned
    }
}

/// Where the project file of `name` goes inside `folder`:
/// `<folder>/<name>/<name>.dst`.
pub fn project_file(folder: &str, name: &str) -> String {
    let stem = file_stem(name);
    Path::new(folder.trim())
        .join(&stem)
        .join(format!("{stem}.dst"))
        .to_string_lossy()
        .to_string()
}

/// The default parent folder of new projects: the user's Documents folder,
/// else the home folder, else nothing (a browser has neither).
pub fn default_folder() -> String {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    if home.is_empty() {
        return String::new();
    }
    let documents = Path::new(&home).join("Documents");
    if documents.is_dir() {
        documents.to_string_lossy().to_string()
    } else {
        home
    }
}
