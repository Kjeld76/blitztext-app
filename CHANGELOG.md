**English** · [Deutsch](CHANGELOG.de.md)

# Changelog — Blitztext for Windows

All notable changes to the Windows port of Blitztext are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project adheres to [Semantic Versioning](https://semver.org/).
The Windows app has its **own version track** (independent of the macOS app), starting at `0.1.0`.

## [Unreleased]

### Fixed
- **Beginning of recordings was clipped**: the UI signalled "recording" before the
  microphone was actually live, so the first word(s) were lost while WASAPI opened the
  input device (~100–300 ms). Recording start now waits for the audio stream to actually
  play before signalling readiness, and the duration is timed from that point.
- **Spoken sentence was sometimes answered instead of improved**: when a transcript
  was phrased like a question or request (e.g. "Can you help me find a date?"), the LLM
  could reply to it instead of just correcting/improving the wording, and that reply was
  pasted. The transcript is now wrapped in explicit markers and every text prompt carries
  a guard clause instructing the model to treat the input strictly as text to edit — never
  as an instruction. Applies to all text workflows (correction, improver, emoji, calm down),
  including custom prompts.

## [0.1.1] - 2026-06-07

First public release with **GPU acceleration**. Includes all improvements since `0.1.0`.

### Added
- **GPU acceleration (CUDA)** active: `large-v3` runs on the NVIDIA GPU
  (tested on an RTX PRO 4000, Blackwell `sm_120`) — much faster than on CPU.
  The GPU build also enables **beam search** for higher accuracy.
- **Pause/activate** without quitting: a global switch in the header and the tray menu;
  paused deregisters all global hotkeys (the app stays in the tray).
- **Tray icon shows the state**: colored = active, grayscale = paused.
- **Closable window**: the close button (✕) hides to the tray; a left-click on the tray
  toggles the window (visible ⇄ hidden).
- **Blitztext (B)** performs a **light Claude correction** after local transcription
  (Haiku, punctuation/typos only); falls back to the raw transcript if the gateway is
  unreachable. New setting `correctionModel`.
- **GPU feature flag** (`--features cuda`): also enables beam search (more accurate).
- The model dropdown shows the **installation status** (uninstalled ones disabled).
- **Custom app/tray icon** (lightning bolt + speech bubbles) instead of the Tauri default;
  also in the window header and as the favicon. Tray colored = active, grayscale = paused.

### Changed
- **Updated whisper-rs to 0.16** (from 0.12): supports CUDA 13.x and the
  Blackwell architecture (`sm_120`); the CUDA target architecture is configurable via
  `CMAKE_CUDA_ARCHITECTURES`.
- **Audio resampling** to 16 kHz with an anti-aliasing low-pass filter → better recognition.
- **Hotkey modes**: `R` (calm down) now uses **Sonnet** instead of Opus.
- The window is no longer "always on top".

### Fixed
- Hotkey **capture**: global shortcuts are paused during capture, so combinations like
  `Ctrl+Alt+B` (rather than only single keys) can be assigned.
- UI serialization (camelCase): fixes "undefined MB", an empty model dropdown, and an
  incorrect token status display.

### Known
- **Auto-paste** uses a simulated `Ctrl+V` and works in normal programs
  (e.g. a text editor, Obsidian). Some **terminals** don't map `Ctrl+V` to paste
  (use right-click or `Ctrl+Shift+V` there) — a terminal fallback is planned.

## [0.1.0] - 2026-06-06

First working Windows version (Tauri, Rust + SvelteKit). Starts as a tray app.

### Added
- **5 workflows** ported from the macOS app: Blitztext, Blitztext Local, Blitztext+,
  Blitztext $%&!, Blitztext :) — LLM prompts adopted verbatim.
- **Local transcription** via whisper.cpp (`whisper-rs`) with the `large-v3` model
  (highest accuracy); in-app model download with progress. Optional online STT.
- **LLM rewriting** via a configurable OpenAI-compatible endpoint
  (base_url + model + token) — for auth2api/Ollama/OpenAI. "Test connection".
- **Global hotkeys**, freely configurable (hold/toggle, Esc to cancel), defaults
  `Ctrl+Alt+B/L/P/R/E`.
- **Auto-paste** via the clipboard + a simulated Ctrl+V (no special permissions needed).
- **Tray icon** with status indicator; **autostart** at Windows start (toggleable).
- Credentials in the **Windows Credential Manager**; settings as JSON in `%APPDATA%\Blitztext`.
- Settings UI (hotkey editor, gateway config, model management, autostart).

### Known limitations
- GPU acceleration (CUDA) not enabled by default yet — `large-v3` runs on the CPU at first.
- An end-to-end functional test (record → transcript → rewrite) requires a downloaded
  Whisper model and a running auth2api proxy.
