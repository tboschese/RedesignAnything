# Packaging — Redesign Anything

This fork reuses Craft Anything packaging under `packaging/macos`, `packaging/linux`, and `packaging/windows`.

## Display vs machine names

| | |
|---|---|
| **Display name** | Redesign Anything |
| **Machine / crate / CLI** | `craft-anything*` (upstream heritage; rebrand of binaries is a follow-up) |
| **Bundle id** | `ai.storyteller.craft-anything` until a dedicated bundle id is registered |

## Build installers (same scripts as Craft)

```bash
# macOS (requires signing material — do not invent certs)
./packaging/macos/package.sh

# Linux AppImage / nfpm
./packaging/linux/package.sh
```

Set optional env for release notes / DMG labels when scripts support it:

```bash
export PRODUCT_DISPLAY_NAME="Redesign Anything"
export PRODUCT_CREDITS="Engine: Craft Anything (MIT OR Apache-2.0). Product: Redesign Anything."
```

## First public release packaging bar

- [x] SBOM: `cargo xtask sbom`
- [x] Notices: `THIRD_PARTY_NOTICES.md`
- [ ] Signed installers on clean OS images (operator signing keys required)
- [ ] Binary rename `redesign-anything-cli` (optional follow-up; CLI commands already use `redesign-*`)

Never push private design systems into installer payloads.
