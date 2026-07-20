# Leaf fork notes for Codex agents

Start with the upstream project documentation:

- `CONTRIBUTING.md` for contribution, formatting, and review expectations.
- `ARCHITECTURE.md` for the module layout and UI event flow.
- `TESTING.md` for fixtures and manual testing.

This file records **astral303 fork-specific** workflow.  Do not add its Mise,
local-install, or custom-version guidance to upstream-facing documentation.

## Remotes and branch strategy

- `origin` is the personal fork: `https://github.com/astral303/leaf.git`.
- `upstream` is the project repository: `https://github.com/RivoLink/leaf.git`.
- `main` is the usable personal-fork branch.  It includes local convenience
  changes and may contain merged upstream-ready feature commits.

For a change that might become a pull request, create its branch from
`upstream/main`, not from fork `main`.  Keep that branch limited to the
focused feature/fix and its necessary docs/tests.  In particular, exclude
Mise files, `AGENTS.md`, the fork version suffix, and other fork-only changes.
After the focused work is ready, merge or cherry-pick it into fork `main` for
local use.  Current examples are `viewer-keymap-overrides-upstream` and
`normalize-file-picker-paths-upstream`; `mise-build-environment` is
intentionally fork-only.

Fetch before deciding where to base work:

```powershell
git fetch origin upstream
```

## Fork-local build and installation

`mise.toml` is a personal convenience layer.  It pins Rust through
`mise.lock`; do not casually update either file.  On this fork's `main`:

```powershell
mise install
mise run build
mise run install
```

`mise run install` depends on `build` and replaces the locally installed
custom executable:

- Windows: `%LOCALAPPDATA%\Programs\leaf\leaf.exe`
- macOS: `~/.local/bin/leaf` (ensure that directory is on `PATH`)

Windows builds use `scripts/build-windows.cmd`, which loads the Visual Studio
Build Tools environment so Rust can find the MSVC linker.  It resolves Cargo
from `CARGO_HOME` when Mise sets it and otherwise uses `PATH`.  Keep arguments
out of that wrapper: it should forward `%*`, while `mise.toml` defines the
actual build arguments.

The fork's `Cargo.toml` version suffix (`-astral303-1`) is intentional and
makes `leaf --version` a quick check that the custom binary is in use.  Do not
put it on a branch intended for upstream.  `leaf --update` would overwrite the
custom Windows executable; leaf does not perform an automatic update check on
ordinary startup.

## Testing on Windows

The MSVC target needs the VS developer environment for direct Cargo commands.
The build task handles this; for direct tests, run Cargo after `VsDevCmd.bat`
has been called, for example:

```powershell
cmd.exe /d /c 'call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 -no_logo && "%USERPROFILE%\.cargo\bin\cargo.exe" test -- --quiet'
```

The current main branch's full suite passes on Windows.  The file-picker tests
compare normalized paths using `src/tests/mod.rs::normalize_path_separators`;
this is deliberately test-only.  The UI must keep displaying Windows-native
`\` separators.  Tests with `Vec<String>` use iterator comparisons such as
`labels.iter().any(|label| label == expected)` to avoid temporary allocation.

## UI changes: places that are easy to miss

- Keyboard behavior is mode-sensitive in `src/runtime/keyboard.rs`.  Modal
  controls must keep precedence over configurable viewer bindings.
- Viewer defaults and action descriptions live in `src/keymap/viewer.rs`.
  Help, status, and catalog labels derive from the effective keymap; do not
  add new handwritten viewer shortcuts in rendering code.
- Personal shortcuts such as Escape to quit and Space/Backspace page
  navigation belong in `[keymap.viewer]` in the local config, not fork code.

## Line endings

Fork `main` has `.gitattributes`: source, docs, TOML, and lock files use LF;
Windows command scripts use CRLF.  Do not make an unrelated line-ending-only
commit and do not change global `core.autocrlf` merely for this repository.
An upstream-based branch will not have these fork-only attributes, so review
the diff carefully before committing.
