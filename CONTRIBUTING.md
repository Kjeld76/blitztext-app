**English** · [Deutsch](CONTRIBUTING.de.md)

# Contributing

Thanks for taking a look at Blitztext (Windows).

This repository is deliberately a **preview/learning project**. Contributions should make it easier to understand, build, fork, or safely extend the project.

> Note: This repository contains the **Windows app**. The original macOS project (by cmagnussen) lives in the [original repository](https://github.com/cmagnussen/blitztext-app) — see the origin section in the [README](README.md).

## Good first contributions

- Improve the build instructions
- Fix confusing UI text
- Better error messages
- Tests around prompt assembly and quality filters
- Simplify the setup
- A custom app icon / branding

## Before a pull request

Please describe:

- what changed
- why
- how you tested it

Keep changes small and focused; no unrelated cleanups in the same PR.

## Local build (Windows)

```powershell
pnpm install
pnpm tauri dev          # run (development)
pnpm tauri build        # installers (MSI/NSIS)
```

For requirements and the libclang note, see [README.md](README.md).
Always build via the **Tauri CLI**, not via a bare `cargo build`.

## Security & privacy

- **Never** commit API keys, tokens, private audio, or confidential transcripts.
- No telemetry, hosted services, or external dependencies without prior discussion (an issue).
- Clearly call out privacy-relevant changes in the PR.
- Stay honest: don't describe a cloud path (e.g. AI rewriting via a gateway) as "offline/local".

## Project scope

The focus is the Windows app in this repository. Please discuss larger directional changes in an issue first.
