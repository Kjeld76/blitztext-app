**English** · [Deutsch](README.de.md)

# Blitztext for Windows

Blitztext is a **Windows 11 tray app** that turns speech into text: press a hotkey, speak, get text back, optionally rewrite it with AI, and have it pasted automatically into the app you were just using.

It is deliberately small and "hackable" — a real, legible workflow you can read and adapt, not a polished finished product.

## Origin / based on

> [!IMPORTANT]
> **This project is a port** of the original **macOS app "Blitztext"** by **cmagnussen**: <https://github.com/cmagnussen/blitztext-app>.
>
> The entire idea, the workflows, and the AI prompts come from that original (released under **MIT**). This repository contains the standalone **Windows port**; the macOS source lives in the [original repository](https://github.com/cmagnussen/blitztext-app). Thanks to the original project.

## Acknowledgements & credits

This project stands on the shoulders of two great pieces of prior work:

- 🎙️ **Blitztext (original, macOS)** by **cmagnussen** — idea, workflows, and prompts: <https://github.com/cmagnussen/blitztext-app>
- 🔌 **auth2api** by **Marc Meese** — the actual key to the flexibility: <https://community.marcmeese.de/freebie/auth2api> (Docker image: `ranktotop/auth2api`)

> [!IMPORTANT]
> **Without auth2api this flexibility wouldn't exist.** auth2api exposes an **existing subscription (e.g. Claude Max) as an OpenAI-compatible API** — so the AI workflows here run **without extra per-token costs**, through your existing account. Many thanks to Marc Meese for this tool! 🙏

## What was changed — and why

The macOS binary **cannot** run on Windows (it builds on macOS frameworks like AppKit, CoreML/WhisperKit, Keychain). The Windows version is therefore a **reimplementation with identical behavior**:

| Area | Original (macOS) | Windows port | Why |
|---|---|---|---|
| Tech | Swift / SwiftUI | **Tauri (Rust + SvelteKit)** | Runs natively on Windows, lean binary |
| Transcription | WhisperKit / CoreML | **whisper.cpp (`large-v3`)** | CoreML only exists on the Mac |
| AI rewriting | hardwired OpenAI | **configurable OpenAI-compatible endpoint** | Use existing subscriptions via a proxy (e.g. auth2api/Claude), Ollama, or OpenAI — without code changes |
| Hotkeys | `fn` + modifier (fixed) | **freely configurable** (Ctrl/Alt/…) | There is no `fn` key on Windows |
| Auto-paste | CGEvent + Accessibility | **Ctrl+V via SendInput** | Possible on Windows without special permissions |
| Secrets | macOS Keychain | **Windows Credential Manager** | Platform equivalent |
| Autostart | SMAppService | **Registry `Run` (tauri-plugin-autostart)** | Platform equivalent |

## Features

- **Blitztext** (`Ctrl+Alt+B`): record → transcribe locally → optional light AI correction (punctuation/typos) → paste.
- **Blitztext Local** (`Ctrl+Alt+L`): same as above, but **fully local/offline** (no cloud).
- **Blitztext+** (`Ctrl+Alt+P`): record → transcribe → rephrase the text more cleanly (selectable tone).
- **Blitztext $%&!** (`Ctrl+Alt+R`): turn a frustrated, dictated rant into a calm message.
- **Blitztext :)** (`Ctrl+Alt+E`): insert fitting emoji into the dictated text.

Hotkeys are freely assignable in the settings; **hold** or **press/toggle** mode.

### Usage

- **Tray icon** (bottom right, possibly in the "^" overflow): left-click opens/closes the window; right-click opens the menu (Open, Settings, Pause/Activate, Quit).
- **✕** in the window hides it to the tray (the app keeps running in the background).
- **Pause** (header switch or tray): deregisters all global hotkeys without quitting the app. The **tray icon is colored = active, grayscale = paused**.
- **Quit** only via tray → **Quit**.

## Screenshots

The main window — hold a workflow's hotkey (or click its tile), speak, and the text is pasted automatically:

![Blitztext main window](docs/screenshots/win-start.png)

**Settings → Access** — the OpenAI-compatible LLM gateway (e.g. auth2api): base URL, the correction/fast/strong models, and an optional token with a "Test connection" button:

![Settings — LLM gateway](docs/screenshots/win-settings-access.png)

**Settings** — download/select the local Whisper model, optionally configure an online STT endpoint, and toggle autostart:

![Settings — Whisper model and system](docs/screenshots/win-settings-models.png)

## Installation

Ready-made installers are in the [GitHub release](https://github.com/Kjeld76/blitztext-app/releases) — in **two variants**:

- **CPU** (`Blitztext_<version>_x64-cpu-setup.exe`): universal, small, runs on any x64 Windows. Recommended **without** an NVIDIA GPU.
- **CUDA/GPU** (`Blitztext_<version>_x64-cuda-setup.exe`): with NVIDIA GPU acceleration (`large-v3` runs much faster). Bundles the required CUDA runtime — a **separate CUDA toolkit is not needed**. Recommended **with** an NVIDIA GPU (Turing or newer).

Each is also available as `.msi`. After installing, download the Whisper model in the settings on first launch. The installers are unsigned — Windows SmartScreen may warn on first launch ("More info" → "Run anyway").

## System requirements (usage)

| | Minimum | Recommended |
|---|---|---|
| **Operating system** | Windows 10 64-bit (21H2) | Windows 11 |
| **Architecture** | x64 | x64 (no ARM64 build) |
| **Memory** | 8 GB | 16 GB (`large-v3` uses ~3 GB) |
| **Disk** | ~150 MB app + **~3.1 GB** for the `large-v3` model (downloaded in the app) | plus temporary audio data |
| **Microphone** | required | — |

**GPU (optional, CUDA installer only):** NVIDIA with CUDA compute capability **≥ 7.5** (Turing or newer: GTX 16xx, RTX 20xx / 30xx / 40xx / 50xx, RTX PRO / Blackwell), **~4 GB free VRAM** for `large-v3` (otherwise `large-v3-turbo` or CPU), a current NVIDIA driver (for Blackwell/`sm_120` with CUDA 13 support). The CUDA runtime libraries are bundled in the CUDA installer.

**Without a suitable NVIDIA GPU:** the **CPU installer** runs everywhere (including AMD/Intel/no GPU), just slower.

## Important notes (preview)

- This version targets **Windows 11**; the macOS variant lives in the [original repository](https://github.com/cmagnussen/blitztext-app).
- **Bring your own access:** speech recognition runs locally (free). For AI rewriting you need an **OpenAI-compatible endpoint** (e.g. your own auth2api proxy, Ollama, or an OpenAI key).
- **No hosted backend.** In the online case, data goes directly from your machine to the endpoint you configured.
- A learning and experimentation project, **not production-ready**, no warranty and no support guarantee.

## Requirements (development / building from source)

> Only needed if you build it yourself. For plain usage see **Installation** above.

- **Windows 11**
- **Rust** (stable, MSVC) + **VC++ Build Tools 2022**
- **Node ≥ 20** and **pnpm**
- **CMake** (for whisper.cpp) and **libclang** (for `whisper-rs`/bindgen; e.g. via `winget install LLVM.LLVM`, then set `LIBCLANG_PATH`)
- For the AI workflows: an **OpenAI-compatible endpoint** (base URL + model + optional token)
- For local transcription: the Whisper model `large-v3` (downloaded directly in the app)

## Build & run

```powershell
git clone https://github.com/Kjeld76/blitztext-app.git
cd blitztext-app
pnpm install
pnpm tauri dev          # start the app in the tray (development)
pnpm tauri build        # build the MSI/NSIS installers
```

GPU acceleration (NVIDIA/CUDA) is optional: `pnpm tauri build --features cuda` (also enables beam search for higher accuracy). **Building** requires the CUDA toolkit (13.x); the target architectures are set via `CMAKE_CUDA_ARCHITECTURES` in the local `src-tauri/.cargo/config.toml` (e.g. `75-real;86-real;89-real;120-real;75-virtual` for Turing–Blackwell). The **finished CUDA installers** ship the CUDA runtime themselves — end users don't need a toolkit.

> Always build via the **Tauri CLI** (`pnpm tauri …`), **not** via a bare `cargo build` — otherwise the app expects the dev server on `localhost:1420`.

On first launch: enter the LLM gateway (base URL/model/token) under **Settings → Access** and download the Whisper model.

## Permissions

- **Microphone**: to record your voice. If no text appears after speaking: *Windows Settings → Privacy → Microphone → "Let desktop apps access your microphone"*.
- A separate Accessibility grant like on macOS is **not needed** — pasting is done via a simulated Ctrl+V.

## Data flow

```text
Transcription (default):  your PC -> local whisper.cpp (large-v3)
AI rewriting:             your PC -> your OpenAI-compatible endpoint (e.g. auth2api -> Claude)
Transcription (optional): your PC -> external OpenAI-compatible STT service (only if configured)
```

No hosted Blitztext backend. The access token is stored in the **Windows Credential Manager**, settings in `%APPDATA%\Blitztext\settings.json`, models in `%APPDATA%\Blitztext\models\`.

## Project structure

```text
src-tauri/         Rust backend (audio, Whisper, LLM gateway, hotkeys, tray, autostart)
src/               SvelteKit UI (workflow tiles, settings)
static/            Static assets
docs/Projektstatus.md   Detailed status report (in German)
CHANGELOG.md       Change history
THIRD-PARTY-NOTICES.md  Third-party licenses (incl. the bundled NVIDIA CUDA runtime)
```

## Local models

Local transcription uses whisper.cpp. The app does not bundle a model — select and download `large-v3` (highest accuracy) in the settings. With GPU/CUDA it runs much faster.

## Contributing

Contributions are welcome, especially ones that make building, understanding, or forking easier. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Code under the **MIT License** — see [LICENSE](LICENSE). This Windows port adopts the license of the original (cmagnussen/blitztext-app).

Project names, logos, and app icons are therefore not automatically released as trademarks/brand assets — see [TRADEMARKS.md](TRADEMARKS.md).

## Legal / imprint & privacy

An experimental, non-commercial open-source project, provided "as is" under MIT, without warranty or support. Nothing is sold, and nothing is installed or operated on your behalf.

The companion website (blitztext.de) of the original is operated by Blackboat Internet GmbH:

- Imprint: <https://www.blackboat.com/impressum>
- Privacy: <https://www.blackboat.com/datenschutz>
