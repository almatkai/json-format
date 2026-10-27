# Format JSON for Tezbar

Format JSON/JS Objects in Tezbar on Windows or macOS with selectable indentation.

Standalone Rust process implementation (schema 3): [v2/README.md](v2/README.md).

## Commands

- **Format Clipboard JSON** — formats JSON/JS object from clipboard and copies/pastes back.
- **Format Selected JSON** — formats selected text in the foremost editor and copies/pastes back.
- **Format Selected JSON and Show** — formats selection and previews in a viewer.
- **Format JSON** — form with text area; choose Format (copy/paste) or View Result (detail preview). Handles escaped and stringified payloads and JS object literals via safe eval.
- **Format Array of JSON to JSONLines** — converts a JSON array string to newline-delimited JSONLines.
- **Format Text into Valid JSON Value** — wraps text as a JSON string value (`JSON.stringify`).

All formatting respects preferences: `Auto Paste` (paste vs copy), `Indentation` (`tab`, `2`, `4`, `8`), and `Max Initial Lines` for viewer truncation.

## Compatibility

- Windows: built and exercised in Tezbar on Windows 11.
- macOS: uses only cross-platform `@raycast/api`, `@raycast/utils`, and `js-beautify` (pure JS beautifier) and retains the upstream macOS-compatible design.
- Linux: not currently declared.

## Local verification

```powershell
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

## Credits

Adapted for Tezbar from Raycast's MIT-licensed `json-format` extension at upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author `destiner` and contributors remain credited in `package.json`.
