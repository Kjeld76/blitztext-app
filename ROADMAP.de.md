[English](ROADMAP.md) · **Deutsch**

# Roadmap

Eine Preview-Roadmap, kein Versprechen.

> Betrifft die Windows-App in diesem Repository; das macOS-Original liegt im [Original-Repository](https://github.com/cmagnussen/blitztext-app).

## Aktueller Stand (erreicht)

- Native **Windows-11-Tray-App** (Tauri, Rust + SvelteKit)
- **Lokale Transkription** mit Whisper `large-v3` (whisper.cpp)
- KI-Workflows (verbessern / beruhigen / Emojis) über einen **konfigurierbaren OpenAI-kompatiblen Gateway** (z. B. auth2api/Claude)
- **Blitztext (B)** mit leichter Claude-Korrektur; **Blitztext Lokal (L)** rein offline
- **frei konfigurierbare** globale Hotkeys (Halten/Toggle)
- **Autostart**, **Pausieren/Aktivieren**, Tray-Statusanzeige
- **Eigenes App-/Tray-Icon** (Branding) statt Tauri-Standard
- **GPU-Beschleunigung (CUDA)** für `large-v3` + Beam-Search (`--features cuda`); native Kernel sm_75–120 (Turing–Blackwell) + PTX-Fallback
- MSI/NSIS-Installer (CPU- und CUDA-Variante)

## Nächste sinnvolle Schritte

- **Terminal-Paste-Fallback** (`Strg+Umschalt+V` bei erkannten Terminal-Fenstern)
- Onboarding-Feinschliff, Waveform-Anzeige
- Optional: weitere Provider/Modelle (z. B. Gemini), Online-STT-Komfort
- Kleine Test-Schicht um Prompt-Aufbau und Qualitätsfilter

## Vorerst nicht im Fokus

- Produktions-Support
- Accounts, Sync, Teams oder gehostete Infrastruktur
- Aussagen, die App sei vollständig „offline/datenschutz-komplett", wenn ein Cloud-Gateway genutzt wird
- Store-Distribution
