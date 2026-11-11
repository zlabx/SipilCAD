# Modifications

SipilCAD is derived from Open CAD Studio (https://github.com/HakanSeven12/OpenCADStudio),
GPL-3.0. Changes made by the rebrand overlay (GPL-3.0 section 5(a)):

- Product name and identity: the tab title (just "SipilCAD", in every language), loading screen, About dialog, copied info text, the web app manifest and
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
- Default theme: the colour scheme a fresh install starts with is Light instead of Oxocarbon (`src/app/config.rs`,
  `src/app/mod.rs`); saved choices are read as before.
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
- Based on HakanSeven12/OpenCADStudio commit `dbd50b20091cf6ce9cf61967ae0563df04014f5a`.

Additional changes applied by the build after those substitutions:

- modified: `build.rs`
- modified: `src/app/commands/mod.rs`
- modified: `src/app/commands/sheet_set.rs`
- modified: `src/app/commands/view.rs`
- modified: `src/app/config.rs`
- modified: `src/app/mod.rs`
- modified: `src/app/properties.rs`
- modified: `src/app/startup.rs`
- modified: `src/app/update/mod.rs`
- modified: `src/app/update/viewport.rs`
- modified: `src/app/view/mod.rs`
- modified: `src/app/view/modal.rs`
- modified: `src/cli.rs`
- modified: `src/i18n.rs`
- modified: `src/input/spacemouse/navlib.rs`
- modified: `src/io/edit_lock.rs`
- modified: `src/io/file_association.rs`
- modified: `src/io/pdf_export.rs`
- modified: `src/io/recovery.rs`
- modified: `src/io/step.rs`
- modified: `src/io/stl.rs`
- modified: `src/io/svg_export.rs`
- modified: `src/lib.rs`
- modified: `src/modules/registry.rs`
- modified: `src/plugin/marketplace.rs`
- modified: `src/sys.rs`
- modified: `src/ui/ribbon/mod.rs`
- modified: `src/ui/ribbon/widgets.rs`
- modified: `src/ui/window/about.rs`
- modified: `src/ui/window/options.rs`
- modified: `src/ui/window/sheet_set.rs`
- modified: `src/ui/window/xref_help.rs`
- modified: `web-app.html`
- modified: `web/app.webmanifest`
- modified: `web/locale-labels.json`
- modified: `web/sw.js`
- added or replaced: `BUILD.md`
- added or replaced: `README.md`
- added or replaced: `SOURCE.txt.in`
- added or replaced: `about-windows.toml`
- added or replaced: `about.hbs`
- added or replaced: `about.toml`
- added or replaced: `assets/LOGO.md`
- added or replaced: `assets/icons/sipil/beam.svg`
- added or replaced: `assets/icons/sipil/column.svg`
- added or replaced: `assets/icons/sipil/door.svg`
- added or replaced: `assets/icons/sipil/door/p1.svg`
- added or replaced: `assets/icons/sipil/door/p2.svg`
- added or replaced: `assets/icons/sipil/door/p3.svg`
- added or replaced: `assets/icons/sipil/door/plan_closed.svg`
- added or replaced: `assets/icons/sipil/door/plan_open.svg`
- added or replaced: `assets/icons/sipil/grid.svg`
- added or replaced: `assets/icons/sipil/steel/cell-form.svg`
- added or replaced: `assets/icons/sipil/steel/channel.svg`
- added or replaced: `assets/icons/sipil/steel/equal-angle.svg`
- added or replaced: `assets/icons/sipil/steel/h-beam.svg`
- added or replaced: `assets/icons/sipil/steel/honeycomb.svg`
- added or replaced: `assets/icons/sipil/steel/i-beam.svg`
- added or replaced: `assets/icons/sipil/steel/king.svg`
- added or replaced: `assets/icons/sipil/steel/queen.svg`
- added or replaced: `assets/icons/sipil/steel/t-beam.svg`
- added or replaced: `assets/icons/sipil/steel/unequal-angle.svg`
- added or replaced: `assets/icons/sipil/steel/wide-flange.svg`
- added or replaced: `assets/icons/sipil/titleblock.svg`
- added or replaced: `assets/icons/sipil/wall.svg`
- added or replaced: `assets/icons/sipil/wall_settings.svg`
- added or replaced: `assets/logo-small.svg`
- added or replaced: `assets/logo.svg`
- added or replaced: `packaging/windows/main.wxs`
- added or replaced: `packaging/windows/sipilcad.iss`
- added or replaced: `packaging/windows/strings.en-US.wxl`
- added or replaced: `packaging/windows/ui.wxs`
- added or replaced: `scripts/sipil-licenses.sh`
- added or replaced: `scripts/sipil-postbuild.py`
- added or replaced: `scripts/sipil-tools.sh`
- added or replaced: `src/app/commands/sheet_set/sipil_project.rs`
- added or replaced: `src/app/sipil_arch/beam.rs`
- added or replaced: `src/app/sipil_arch/column.rs`
- added or replaced: `src/app/sipil_arch/door.rs`
- added or replaced: `src/app/sipil_arch/grid.rs`
- added or replaced: `src/app/sipil_arch/mod.rs`
- added or replaced: `src/app/sipil_arch/titleblock.rs`
- added or replaced: `src/app/sipil_wall.rs`
- added or replaced: `src/sipil.rs`
- added or replaced: `src/sipil_arch/beam/adapter.rs`
- added or replaced: `src/sipil_arch/beam/command.rs`
- added or replaced: `src/sipil_arch/beam/form.rs`
- added or replaced: `src/sipil_arch/beam/info.rs`
- added or replaced: `src/sipil_arch/beam/mod.rs`
- added or replaced: `src/sipil_arch/beam/model.rs`
- added or replaced: `src/sipil_arch/beam/plan.rs`
- added or replaced: `src/sipil_arch/beam/planner.rs`
- added or replaced: `src/sipil_arch/beam/ui.rs`
- added or replaced: `src/sipil_arch/column/adapter.rs`
- added or replaced: `src/sipil_arch/column/command.rs`
- added or replaced: `src/sipil_arch/column/form.rs`
- added or replaced: `src/sipil_arch/column/info.rs`
- added or replaced: `src/sipil_arch/column/mod.rs`
- added or replaced: `src/sipil_arch/column/model.rs`
- added or replaced: `src/sipil_arch/column/plan.rs`
- added or replaced: `src/sipil_arch/column/planner.rs`
- added or replaced: `src/sipil_arch/column/ui.rs`
- added or replaced: `src/sipil_arch/door/adapter.rs`
- added or replaced: `src/sipil_arch/door/catalog.rs`
- added or replaced: `src/sipil_arch/door/command.rs`
- added or replaced: `src/sipil_arch/door/form.rs`
- added or replaced: `src/sipil_arch/door/info.rs`
- added or replaced: `src/sipil_arch/door/mod.rs`
- added or replaced: `src/sipil_arch/door/model.rs`
- added or replaced: `src/sipil_arch/door/plan.rs`
- added or replaced: `src/sipil_arch/door/planner.rs`
- added or replaced: `src/sipil_arch/door/sheet.rs`
- added or replaced: `src/sipil_arch/door/ui.rs`
- added or replaced: `src/sipil_arch/fields.rs`
- added or replaced: `src/sipil_arch/grid/adapter.rs`
- added or replaced: `src/sipil_arch/grid/bubble.rs`
- added or replaced: `src/sipil_arch/grid/form.rs`
- added or replaced: `src/sipil_arch/grid/info.rs`
- added or replaced: `src/sipil_arch/grid/mod.rs`
- added or replaced: `src/sipil_arch/grid/model.rs`
- added or replaced: `src/sipil_arch/grid/plan.rs`
- added or replaced: `src/sipil_arch/grid/spec.rs`
- added or replaced: `src/sipil_arch/grid/templates.rs`
- added or replaced: `src/sipil_arch/grid/ui.rs`
- added or replaced: `src/sipil_arch/grid/view.rs`
- added or replaced: `src/sipil_arch/host.rs`
- added or replaced: `src/sipil_arch/mod.rs`
- added or replaced: `src/sipil_arch/registry.rs`
- added or replaced: `src/sipil_arch/steel/catalog.rs`
- added or replaced: `src/sipil_arch/steel/data.rs`
- added or replaced: `src/sipil_arch/steel/draw.rs`
- added or replaced: `src/sipil_arch/steel/mod.rs`
- added or replaced: `src/sipil_arch/steel/pick.rs`
- added or replaced: `src/sipil_arch/steel/shape.rs`
- added or replaced: `src/sipil_arch/steel/ui.rs`
- added or replaced: `src/sipil_arch/titleblock/adapter.rs`
- added or replaced: `src/sipil_arch/titleblock/form.rs`
- added or replaced: `src/sipil_arch/titleblock/geometry.rs`
- added or replaced: `src/sipil_arch/titleblock/info.rs`
- added or replaced: `src/sipil_arch/titleblock/mod.rs`
- added or replaced: `src/sipil_arch/titleblock/model.rs`
- added or replaced: `src/sipil_arch/titleblock/sheet.rs`
- added or replaced: `src/sipil_arch/titleblock/ui.rs`
- added or replaced: `src/sipil_project/db.rs`
- added or replaced: `src/sipil_project/mod.rs`
- added or replaced: `src/sipil_project/state.rs`
- added or replaced: `src/sipil_project/templates.rs`
- added or replaced: `src/sipil_project/ui.rs`
- added or replaced: `src/sipil_wall/command.rs`
- added or replaced: `src/sipil_wall/engine.rs`
- added or replaced: `src/sipil_wall/entities.rs`
- added or replaced: `src/sipil_wall/mod.rs`
- added or replaced: `src/sipil_wall/module.rs`
- added or replaced: `src/sipil_wall/state.rs`
- added or replaced: `src/sipil_wall/ui.rs`
- added or replaced: `tests/upstream_links.rs`
- added or replaced: `web/discussions.json`
- added or replaced: `web/pt-sans/OFL-PTSans.txt`
- added or replaced: `web/pt-sans/PT_Sans-Web-Bold.ttf`
- added or replaced: `web/pt-sans/PT_Sans-Web-Regular.ttf`
- added or replaced: `web/videos.json`

The date of these changes is the date of this build's commit in the git history of this repository.
