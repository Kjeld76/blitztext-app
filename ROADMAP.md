**English** · [Deutsch](ROADMAP.de.md)

# Roadmap

A preview roadmap, not a promise.

> Concerns the Windows app in this repository; the macOS original lives in the [original repository](https://github.com/cmagnussen/blitztext-app).

## Current state (done)

- Native **Windows 11 tray app** (Tauri, Rust + SvelteKit)
- **Local transcription** with Whisper `large-v3` (whisper.cpp)
- AI workflows (improve / calm down / emoji) via a **configurable OpenAI-compatible gateway** (e.g. auth2api/Claude)
- **Blitztext (B)** with a light Claude correction; **Blitztext Local (L)** fully offline
- **freely configurable** global hotkeys (hold/toggle)
- **autostart**, **pause/activate**, tray status indicator
- **custom app/tray icon** (branding) instead of the Tauri default
- **GPU acceleration (CUDA)** for `large-v3` + beam search (`--features cuda`); native kernels sm_75–120 (Turing–Blackwell) + PTX fallback
- MSI/NSIS installers (CPU and CUDA variants)

## Next sensible steps

- **Terminal paste fallback** (`Ctrl+Shift+V` for detected terminal windows)
- Onboarding polish, waveform display
- Optional: more providers/models (e.g. Gemini), online-STT convenience
- A small test layer around prompt assembly and quality filters

## Not in focus for now

- Production support
- Accounts, sync, teams, or hosted infrastructure
- Claims that the app is fully "offline/privacy-complete" when a cloud gateway is used
- Store distribution
