<!--
Author: Jeff
Date: 2026-06-08
Description: Configurable UI and workflow model for Geist DAW.
Notes: Captures creator-owned workflows as a product requirement before Phase 8 implementation.
-->

# Geist DAW UI Configuration Model

## Creator-authored workflows

Workflow profiles are how creators make Geist fit their own process instead of adapting to a single fixed DAW layout.

## North star

Geist should bring Linux modularity to audio production: creators can shape the working environment around their own habits without turning the DAW into a fragile script collection or a spreadsheet of preferences.

The default UI is only one curated workflow. The product must also support user-authored workflow profiles that define how surfaces, commands, lenses, shortcuts, panels, templates, and add/search behavior fit together.

## Product requirement

UI/UX customization is a first-class feature, not a theme-only preference.

Creators should be able to configure:
- visible lenses and their order,
- default startup workspace,
- panel layout and pinned regions,
- command palette aliases,
- keyboard shortcuts and controller bindings,
- browser categories, favorites, and add-target rules,
- track, rack, graph, and project templates,
- context shelf action order,
- parameter pins and macro surfaces,
- modulation/routing display density,
- meter, clip, cable, and node visual density,
- per-workflow empty-state actions.

Examples:
- A modular sound designer can start in Build lens with graph-first add/search, visible modulators, and deep cable tracing.
- A songwriter can start in Arrange lens with track templates, clip recording actions, and simplified graph detail.
- A mixing engineer can start in Mix lens with meter-heavy panels, routing health, and pinned gain-stage commands.
- A live performer can start in a performance profile with scene/clip triggers, macro controls, and protected destructive actions.

## Configuration shape

Use human-readable versioned config files. Prefer TOML for global/user settings and project-local workflow overrides unless implementation proves another format materially better.

The implementation home is `crates/spectre-config`. That crate owns the workflow schema, TOML loading, validation diagnostics, safe fallback resolution, command alias schema, keybinding schema, and template references.

`crates/geist-ui` consumes the validated workflow profile through `UIState`. UI commands can switch configured lenses, apply a new workflow snapshot, execute declarative aliases, and queue non-UI command intents for the app/project layer instead of mutating DAW truth directly.

The renderer boundary derives a `RenderFrame` from `UIState`: visible lens tabs, active lens, panel placements, density, transport edge, main-view empty actions, and context-shelf actions all come from the active workflow profile. The current egui adapter is a deterministic scaffold around that frame plan so renderer tests can prove config changes affect the UI plan before concrete egui widgets are built.

The view layer now converts that frame plan into renderer-neutral `WorkspaceSurface` and `LensSurface` models. Lens tabs, active surface purpose, visible empty-state action chips, context-shelf action chips, and panel placement data are therefore already config-driven before concrete drawing code exists.

The widget layer now converts `WorkspaceSurface` into `WorkspaceWidgets`: tab widgets, panel widgets, main empty-action buttons, context-shelf buttons, and command-palette state. The egui scaffold exposes `render_widgets` so actual egui drawing can consume deterministic widget inputs that already reflect workflow config.

The app layer now has startup and reload paths. `App::from_workflow_candidates` and `App::from_workflow_files` resolve built-in, bundled, user, and project workflow candidates by precedence: later valid profiles win, invalid later profiles emit source-tagged diagnostics, and the last valid profile remains active. `App::load_workflow_file` parses and validates one workflow TOML file on the app/control side, applies it only when valid, and preserves the existing workflow when diagnostics are returned. `UICommand::LoadWorkflowFile` exposes that reload path to future profile picker or command-palette UI.

The `geist-daw` binary resolves startup workflows before opening the audio stream or GUI. It loads the built-in profile, bundled default profile, optional user profile, optional project override, then an explicit `--workflow <path>` / `--workflow=<path>` file. Diagnostics are printed as warnings; invalid profiles do not stop launch unless audio or window creation fails.

Workflow `templates` now feed the studio browser as searchable insert items. Double-clicking a template emits a typed `instantiate_template` intent with `name`, `kind`, and template args; project/audio mutation remains behind app command dispatch.

Suggested files:
- user/global: `~/.config/geist/workflows/*.toml`
- project-local: `.geist/workflow.toml`
- bundled defaults: `assets/workflows/default.toml`, `modular.toml`, `songwriting.toml`, `mixing.toml`, `performance.toml`

Config is declarative. It describes desired layout, bindings, defaults, aliases, and templates. It does not run arbitrary code in the UI thread or audio callback.

## Precedence

Configuration resolves in this order:

1. Built-in safe defaults.
2. Bundled workflow profile.
3. User workflow profile.
4. Project-local workflow override.
5. Session-only transient UI state.

Project files may reference a workflow profile, but opening a project must remain safe if the profile is missing. Missing profiles fall back to defaults and report a non-blocking warning.

## Safety boundaries

Customization must never compromise real-time audio safety.

Rules:
- Config parsing happens at startup or on an app/control thread, never in the audio callback.
- Config reload produces validated immutable UI config snapshots.
- Mutating actions still emit typed `UICommand` values and pass through normal validation.
- Config can bind commands; it cannot bypass command validation.
- Config can define templates; template instantiation still uses normal project commands.
- Invalid config reports precise diagnostics and falls back to the last valid snapshot.
- Workflow reload is undo-safe: layout changes are UI state; project mutations remain command-based.

## Workflow profile concepts

### Lenses

A profile can choose which lenses are visible, their order, and startup lens.

Required built-in lenses:
- Arrange,
- Build,
- Shape,
- Mix,
- Browser,
- Modulation.

Profiles may hide or de-emphasize lenses, but no core project data should become unreachable. Hidden actions remain discoverable through command search.

### Panels

A profile can arrange persistent regions:
- transport strip,
- main canvas,
- context shelf,
- search/add palette,
- browser/sidebar,
- meters,
- macro/performance strip.

Panel layout should use named regions and weights, not absolute pixels as the primary model. Absolute sizing may be a renderer-level detail.

### Commands and aliases

A profile can alias commands in language that matches the creator's workflow.

Examples:
- `Patch LFO` aliases `Add Modulator: LFO`.
- `Print Stem` aliases an export/bounce command.
- `Make Bass Bus` creates a configured group/routing template.

Aliases are labels for typed command invocations. They are not shell commands.

### Templates

Profiles can define templates for:
- tracks,
- plugin chains,
- graph branches,
- modulation setups,
- project startup state.

Templates should be inspectable before insertion and created through the same undoable command path as manual edits.

### Visual density

Profiles can set UI density without hiding meaning.

Allowed density changes:
- compact, normal, spacious sizing,
- cable label display policy,
- meter detail level,
- visible modulation overlay detail,
- node parameter pin count.

Not allowed:
- unlabeled core controls,
- hiding disabled reasons,
- removing value/unit/range from primary parameter controls,
- making common actions menu-only.

## Minimal config sketch

```toml
version = 1
profile_id = "modular-builder"
display_name = "Modular Builder"
startup_lens = "build"

[lenses]
order = ["build", "shape", "arrange", "mix", "modulation", "browser"]
visible = ["build", "shape", "arrange", "mix", "modulation", "browser"]

[layout]
density = "normal"
left_panel = "browser"
right_panel = "context_shelf"
bottom_panel = "modulation_overview"
transport = "top"

[graph]
cable_labels = "on_hover"
show_latency = true
show_route_health = true
empty_actions = ["add_source", "add_processor", "add_modulator", "open_browser"]

[context_shelf.track]
actions = ["add_instrument", "add_effect", "add_send", "arm", "mute", "solo", "show_graph_branch"]

[commands.aliases]
"Patch LFO" = { command = "add_modulator", kind = "lfo" }
"Add Output" = { command = "add_node", node = "audio_output" }

[keybindings]
"Cmd+K" = "open_command_palette"
"G" = "switch_lens:build"
"A" = "switch_lens:arrange"
```

## Acceptance checklist

Before accepting UI/config work, verify:
- A user can select or author a workflow profile from a config file.
- The default workflow still works with no user config.
- Invalid config falls back safely and reports actionable diagnostics.
- Config does not execute arbitrary code.
- Config does not bypass typed commands or undo behavior.
- Layout customization does not remove labels, values, units, disabled reasons, or destination-visible modulation from primary controls.
- Project-local workflow preferences do not make projects unsafe or unopenable on another machine.
