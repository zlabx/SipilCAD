# Modifications

SipilCAD is derived from Open CAD Studio (https://github.com/HakanSeven12/OpenCADStudio),
GPL-3.0. Changes made by the rebrand overlay (GPL-3.0 section 5(a)):

- Product name and identity: the tab title, loading screen, About dialog, copied info text, the web app manifest and
  service worker texts, and the locale titles in `web/locale-labels.json` use the name SipilCAD; the logo is replaced by
  the SipilStock icon (the favicon uses the label-free variant) and the loading screen is set in PT Sans (OFL, unmodified
  files in `web/pt-sans/`).
- About dialog: shows SipilCAD's own version and a short tagline (`APP_VERSION` and `ABOUT_TAGLINE` in `src/sipil.rs`) and,
  with `COMPACT_ABOUT`, only the top card and the button row; the detail cards stay available in the copied info text.
- Links to the upstream project are hidden or blocked (website, Patreon, GitHub, Reddit, open-aec, YouTube): start page
  buttons and panels, the donation prompt at start-up, the Plugins row in Options, the commands that open upstream sites
  (DONATE, HELP/F1, REPORT, CHANGELOG, PLUGINS, ...), and an `open_url` safety net for new upstream links. All of it sits
  behind one flag, `HIDE_UPSTREAM_LINKS` in `src/sipil.rs`; every changed spot is marked `// SIPIL:` (`git grep "SIPIL:"`).
- Attribution and licence: the About dialog links License, Font (the PT Sans licence) and Source (`LICENSE.txt`,
  `OFL-PTSans.txt` and `SOURCE.txt`, served next to the app). Upstream's `LICENSE` and copyright headers are unchanged.
- Upstream data emptied: `web/videos.json` and `web/discussions.json` hold empty lists; `supporters.json` is emptied at
  build time (`scripts/sipil-postbuild.py`).
- Service worker cache name: `CACHE_VERSION` in `sw.js` is stamped with a hash of the finished bundle at build time
  (`scripts/sipil-postbuild.py`), so every build gets its own cache and the previous one is deleted.
- Guard: `tests/upstream_links.rs` fails when upstream adds a host or an `open_url` call site (its limit counts the
  About "Font" button).
- Added for the build: `about.toml` and `about.hbs` (THIRD_PARTY_LICENSES.txt), `scripts/sipil-*`, `SOURCE.txt.in`,
  `BUILD.md` and a new `README.md`.
- Upstream's CI and release workflows (`.github/`) are not carried in this repository; SipilCAD is built with its
  own pipeline (see `BUILD.md`).
- Based on HakanSeven12/OpenCADStudio commit `b6fbac39ffc74398ba5d283ce2bc1dc07a8fd8f0`.

Additional changes applied by the build after those substitutions:

- modified: `src/app/commands/view.rs`
- modified: `src/app/startup.rs`
- modified: `src/app/view/mod.rs`
- modified: `src/lib.rs`
- modified: `src/sys.rs`
- modified: `src/ui/window/about.rs`
- modified: `src/ui/window/options.rs`
- modified: `src/ui/window/xref_help.rs`
- modified: `web-app.html`
- modified: `web/app.webmanifest`
- modified: `web/locale-labels.json`
- modified: `web/sw.js`
- added or replaced: `BUILD.md`
- added or replaced: `README.md`
- added or replaced: `SOURCE.txt.in`
- added or replaced: `about.hbs`
- added or replaced: `about.toml`
- added or replaced: `assets/LOGO.md`
- added or replaced: `assets/logo-small.svg`
- added or replaced: `assets/logo.svg`
- added or replaced: `scripts/sipil-licenses.sh`
- added or replaced: `scripts/sipil-postbuild.py`
- added or replaced: `scripts/sipil-tools.sh`
- added or replaced: `src/sipil.rs`
- added or replaced: `tests/upstream_links.rs`
- added or replaced: `web/discussions.json`
- added or replaced: `web/pt-sans/OFL-PTSans.txt`
- added or replaced: `web/pt-sans/PT_Sans-Web-Bold.ttf`
- added or replaced: `web/pt-sans/PT_Sans-Web-Regular.ttf`
- added or replaced: `web/videos.json`

The date of these changes is the date of this build's commit in the git history of this repository.
