# Codex Desk SBOM note

Lockfile-derived component list for GRC prep. **Not** a signed SLSA/provenance attestation.
Not a FedRAMP authorization artifact. Regenerated from committed lockfiles.

## npm (direct + lock)

- lockfileVersion: 3
### dependencies
- `@radix-ui/react-dialog` ^1.1.23
- `@radix-ui/react-scroll-area` ^1.2.18
- `@radix-ui/react-separator` ^1.1.15
- `@radix-ui/react-slot` ^1.3.3
- `@radix-ui/react-tooltip` ^1.2.16
- `@tauri-apps/api` ^2
- `@tauri-apps/plugin-opener` ^2
- `class-variance-authority` ^0.7.1
- `clsx` ^2.1.1
- `lucide-react` ^0.544.0
- `react` ^19.1.0
- `react-dom` ^19.1.0
- `tailwind-merge` ^3.6.0

### devDependencies
- `@tailwindcss/vite` ^4.3.3
- `@tauri-apps/cli` ^2
- `@types/node` ^26.4.1
- `@types/react` ^19.1.8
- `@types/react-dom` ^19.1.6
- `@vitejs/plugin-react` ^4.6.0
- `tailwindcss` ^4.3.3
- `typescript` ~5.8.3
- `vite` ^7.0.4

## cargo (direct crate pins)

- `tauri-build` 2
- `tauri` 2
- `serde` 1
- `rusqlite` 0.32
- `uuid` 1
- `chrono` 0.4
- `windows-sys` 0.59
- `tauri-plugin-opener` 2
- `serde_json` 1
- `which` 7
- `toml` 0.8
- `aes-gcm` 0.10
- `rand` 0.8
- `sha2` 0.10
- `hkdf` 0.12
- `hex` 0.4
- `keyring` 3
- `zeroize` 1
- `tempfile` 3

Full graphs: `package-lock.json`, `src-tauri/Cargo.lock`.
