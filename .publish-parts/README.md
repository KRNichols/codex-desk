# Publish parts

Small verified icon PNGs (base64) used by GitHub Actions together with:

- the pinned `KRNichols/IL5-Agent-Protocol` snapshot
  (`da6bf2880aad20aa757b894976304271a3a50511`)
- `npm install --package-lock-only` and `cargo generate-lockfile`

Larger gzip+base64 slices here were an earlier MCP workaround and are
not used for lockfiles or the IL5 standard (several were truncated in
transit). These parts are not secrets. Do not put PATs or `.env.local`
here. This is not an ATO package.
