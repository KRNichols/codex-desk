# Publish parts

Base64 (and gzip+base64) slices used only to land files that exceed
the GitHub MCP upload size. GitHub Actions concatenates them into:

- `docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md` (full KRNichols/IL5-Agent-Protocol snapshot)
- `package-lock.json`
- `src-tauri/Cargo.lock`
- `src-tauri/icons/*`

These parts are not secrets. Do not put PATs or `.env.local` here.
This is not an ATO package.
