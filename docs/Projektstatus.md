---
title: "Blitztext – Windows-Portierung · Projektstatus"
aliases: [Blitztext Windows, BlitztextWin, Blitztext Port]
tags: [projekt/blitztext, plattform/windows, status/aktiv, tech/tauri, tech/rust, ki/whisper, ki/claude]
status: in-arbeit
erstellt: 2026-06-06
aktualisiert: 2026-06-07
repo: https://github.com/Kjeld76/blitztext-app
branch: main
version: 0.1.1
---

# Blitztext – Windows-Portierung · Projektstatus

> [!abstract] Kurzfassung
> **Blitztext** ist ursprünglich eine **macOS-App** (Swift/SwiftUI): Sprache → Text per Whisper, plus KI-gestützte Umschreibe-Workflows. Ziel dieses Projekts ist eine **eigenständige, native Windows-11-App** mit gleicher Funktion, gebaut mit **Tauri (Rust + Web-UI)** – inkl. **Autostart**, **frei konfigurierbaren Hotkeys** und Nutzung vorhandener **Abos statt API-Kosten** (Claude via `auth2api`).
> **Stand 07.06.2026:** Die App ist **lauffähig** (startet als Tray-App, transkribiert lokal, schreibt per Claude um). **GPU-Beschleunigung (CUDA) ist aktiv und validiert** – `large-v3` läuft auf der RTX (Blackwell, `sm_120`) mit Beam-Search, Tempo deutlich besser. Offen ist nur noch das **erste GitHub-Release `v0.1.1`** mit Installern.

---

## 1 · Warum dieses Projekt? (Eingangserklärung)

Die bestehende Blitztext-App läuft **ausschließlich auf macOS** und baut tief auf macOS-Frameworks (AppKit, CoreML/WhisperKit, Carbon/NSEvent-Hotkeys, ServiceManagement, Keychain). **Das Mac-Binary lässt sich nicht unter Windows ausführen** – weder per Emulation noch per Kompatibilitätsschicht. Wer Blitztext unter Windows nutzen will, braucht also eine **Neuimplementierung**.

Gleichzeitig wollte ich die **laufenden API-Kosten vermeiden**: Statt pro Token zu bezahlen, sollen vorhandene **Abos (Claude Max, Gemini Pro)** genutzt werden. Das gelingt über den selbst-gehosteten Proxy **`auth2api`**, der ein Abo als **OpenAI-kompatible API** bereitstellt.

**Zielbild:**
- Native Windows-Tray-App mit denselben 5 Workflows wie auf dem Mac.
- **Höchste Transkriptions-Genauigkeit** (Whisper `large-v3`), lokal.
- **KI-Umschreibung über Claude** (via `auth2api`) – ohne API-Abrechnung.
- **Globale Hotkeys**, in der App frei belegbar.
- **Autostart** beim Windows-Hochfahren.

> [!info] Wichtige Klarstellung
> **Claude kann kein Audio.** Die **Spracherkennung läuft immer lokal** (Whisper). `auth2api`/Claude wird nur für **Text** (Korrektur/Umschreiben) genutzt.

---

## 2 · Was wurde entschieden (und warum)

| Thema | Entscheidung | Begründung |
|---|---|---|
| **Tech-Stack** | Tauri (Rust-Backend + SvelteKit-UI) | Schlankes Binary, native Tray/Hotkey/Autostart-Plugins, später cross-plattform möglich |
| **Transkription** | Lokal, whisper.cpp via `whisper-rs`, Modell **`large-v3`** | Höchste Genauigkeit; läuft offline; keine Kosten |
| **KI-Umschreibung** | Konfigurierbarer **OpenAI-kompatibler** Endpunkt | Funktioniert mit `auth2api`, Ollama, OpenAI – ohne Code-Änderung |
| **Anbieter** | **`auth2api`** (Claude Max), Port **8317** | Nutzt vorhandenes Abo statt API-Kosten |
| **Hotkeys** | Frei konfigurierbar (Hold/Toggle) | macOS nutzt `fn` – die gibt es unter Windows nicht |
| **Autostart** | `tauri-plugin-autostart` (Registry `Run`) | Ersetzt macOS `SMAppService` |
| **Versionierung** | Eigene SemVer-Spur ab `v0.1.0`, Fork, Feature-Branch | Saubere, vom Mac-Projekt unabhängige Historie |

> [!warning] Hinweis zu auth2api (Nutzungsbedingungen)
> `auth2api` greift per Reverse-Proxy auf den Abo-Login zu. Das kann den AGB von Anthropic/OpenAI widersprechen und birgt ein Account-Risiko. Blitztext selbst ist **anbieterneutral** und spricht nur einen OpenAI-kompatiblen Endpunkt an.

---

## 3 · Architektur (Mac → Windows)

| Funktion | macOS (Original) | Windows (neu) |
|---|---|---|
| UI | SwiftUI Popover | Tauri-Fenster (WebView) + Tray |
| Tray/Status | NSStatusBar | `tray-icon` + Tooltip-Status |
| Hotkeys | NSEvent (`fn`+Mod) | `tauri-plugin-global-shortcut` (Pressed/Released) |
| Aufnahme | AVAudioRecorder | `cpal` → 16 kHz mono (mit Anti-Aliasing-Filter) |
| Transkription | WhisperKit/CoreML | **whisper.cpp `large-v3`** via `whisper-rs` |
| Umschreiben | OpenAI Chat | `reqwest` → **konfigurierbarer Gateway** (auth2api/Claude) |
| Einfügen | CGEvent Cmd+V | `arboard` + `enigo` Strg+V |
| Geheimnisse | Keychain | **Windows Credential Manager** (`keyring`) |
| Autostart | SMAppService | `tauri-plugin-autostart` |

**Ablauf eines Workflows:** Hotkey → Aufnahme (cpal) → Resampling 16 kHz → **lokale Transkription (Whisper)** → *optional* Claude-Schritt → **automatisches Einfügen** (Strg+V) ins fokussierte Fenster.

---

## 4 · Workflows & Tastenbelegung

| Hotkey | Workflow | Spracherkennung | Claude-Schritt |
|---|---|---|---|
| **Ctrl+Alt+B** | Blitztext | lokal `large-v3` | **Haiku** – leichte Korrektur (Zeichensetzung/Tippfehler) |
| **Ctrl+Alt+L** | Blitztext Lokal | lokal `large-v3` | – (rein lokal/offline) |
| **Ctrl+Alt+P** | Blitztext+ | lokal `large-v3` | **Sonnet** – Text verbessern (Ton wählbar) |
| **Ctrl+Alt+E** | Blitztext :) | lokal `large-v3` | **Sonnet** – Emojis einfügen |
| **Ctrl+Alt+R** | Blitztext $%&! | lokal `large-v3` | **Sonnet** – Frust beruhigen |

> [!tip] B vs. L
> **B** liefert dank Haiku-Korrektur sauberere Ergebnisse (Kommas, Groß-/Kleinschreibung), braucht aber den Gateway. **L** ist garantiert **rein lokal/offline** – etwas roher, dafür ohne Cloud. Fällt der Gateway aus, fällt **B automatisch auf das rohe Transkript zurück**.

- Modi: **Halten** (Taste halten = aufnehmen) oder **Drücken/Toggle** (Esc bricht ab).
- Hotkeys sind in **Einstellungen → Anpassen** frei belegbar.

---

## 5 · Aktueller Stand (verifiziert am 06.06.2026)

> [!success] Funktioniert
> - [x] App startet **als Tray-App** (kein Panic), Fenster lädt mit eingebettetem Frontend
> - [x] **Lokale Transkription** mit `large-v3` (Modell ~3 GB geladen)
> - [x] **B (lokal + Claude-Haiku-Korrektur)** – vom Nutzer als „genial" bewertet
> - [x] **L (rein lokal)** – funktioniert, erwartungsgemäß roher
> - [x] **LLM-Gateway** an `auth2api` (Port 8317, Claude) – Verbindung & Chat bestätigt
> - [x] **Token maskiert** im Credential Manager
> - [x] **Globale Hotkeys**, Hold/Toggle, Esc-Abbruch
> - [x] **Autostart**-Schalter, Tray-Statusanzeige
> - [x] Modell-Dropdown zeigt **Installationsstatus** (nicht installierte deaktiviert)
> - [x] **Pausieren/Aktivieren** ohne Beenden (Header + Tray); Hotkeys werden dabei ab-/angemeldet
> - [x] **Tray-Icon** farbig (aktiv) / Graustufen (pausiert)
> - [x] **Fenster schließbar** (✕ ins Tray) + Tray-Klick als Sichtbar-Toggle, nicht mehr „always on top"
> - [x] **Hotkey-Erfassung** im UI funktioniert für volle Kombinationen (Capture pausiert globale Shortcuts)
> - [x] Genauigkeit verbessert: **Anti-Aliasing-Resampling**
> - [x] **Eigenes App-/Tray-/Header-Icon** (Branding) statt Tauri-Standard
> - [x] **Installer gebaut**: NSIS `.exe` + MSI `.msi`

> [!todo] Offen / nächste Schritte
> - [x] **GPU-Beschleunigung (CUDA)** aktiv: `pnpm tauri build --features cuda` → `large-v3` auf RTX (Blackwell `sm_120`) + **Beam-Search**; whisper-rs auf 0.16 (CUDA 13.x/Blackwell). Tempo/Qualität bestätigt.
> - [ ] **Tag `v0.1.1`** setzen + **erstes GitHub-Release** mit Installer-Assets (NSIS/MSI) – bewusst erst mit GPU-Version (Entscheidung: kein CPU-Release `v0.1.0`)
> - [ ] **Terminal-Paste-Fallback**: Einfügen klappt in normalen Programmen; manche Terminals nehmen kein `Strg+V` → Foreground-Fenster erkennen und dort `Strg+Umschalt+V` senden.

---

## 6 · Technische Details / Betrieb

- **Einstellungen:** `%APPDATA%\Blitztext\settings.json`
- **Modelle:** `%APPDATA%\Blitztext\models\` (`ggml-large-v3.bin`)
- **Geheimnisse:** Windows Credential Manager, Dienst `de.blitztext.win`
- **Gateway (auth2api):** `http://localhost:8317/v1` (Claude, „healthy"); Container `auth2api-codex-1` auf 8318 ist „unhealthy" (Codex/ChatGPT, nicht genutzt)
- **Modelle laut Gateway:** `claude-opus-4-7`, `claude-sonnet-4-6`, `claude-haiku-4-5` (+ Aliase)
- **Build-Voraussetzung:** `libclang` (für whisper-rs/bindgen) – via lokaler, nicht eingecheckter `src-tauri/.cargo/config.toml`. Empfohlen: LLVM.
- **Bauen:** immer `pnpm tauri build` bzw. `pnpm tauri dev` – **nie** nacktes `cargo build` (erwartet sonst den Dev-Server auf `localhost:1420`).

**Systemvoraussetzungen (Nutzung):** Windows 10/11 **x64**, **8 GB RAM** (16 GB empf.), **~3,1 GB** Platz für das Modell `large-v3`, **Mikrofon**. GPU optional: **NVIDIA ab Compute Capability 7.5** (Turing+: RTX 20xx–50xx/Blackwell), **~4 GB freier VRAM**; die CUDA-Runtime (cuBLAS) ist **im GPU-Installer enthalten** (kein Toolkit nötig). Ohne passende NVIDIA-GPU → **CPU-Installer** (läuft überall, langsamer). Release als **zwei Varianten**: `…-cpu-setup.exe` (universell) und `…-cuda-setup.exe` (NVIDIA). Details: `README.md`. GPU-Build-Architekturen via `CMAKE_CUDA_ARCHITECTURES` in der lokalen `src-tauri/.cargo/config.toml` (`75;86;89;120` + PTX `75-virtual`).

> [!note] Online-STT (optional)
> Die Felder unter *Einstellungen → Zugang → Online-STT* erlauben, statt lokal über einen **Cloud-Transkriptionsdienst** (OpenAI-kompatibel) zu transkribieren. **Leer = alles lokal.** `auth2api` kann das **nicht** (nur Chat). Eine schnelle, kostenlose Option wäre **Groq** (`https://api.groq.com/openai/v1`, `whisper-large-v3-turbo`) – aktuell bewusst **nicht** genutzt.

---

## 7 · Versionierung & Repository

- **Repository:** `Kjeld76/blitztext-app` (`origin`) – eigenständige Windows-Portierung des Originals `cmagnussen/blitztext-app`
- **Branch:** `main`
- **Tag:** `v0.1.1` (erstes öffentliches Release, mit GPU-Beschleunigung)
- **Code:** im Repository-Root (Tauri: `src-tauri/`, `src/`); macOS-Original nicht mehr Teil dieses Repos

**Historie:** Für das erste öffentliche Release wurde das Repo auf eine **frische Historie** zurückgesetzt (Windows-App im Root, macOS-Original entfernt). Frühere Entwicklungs-Commits (Fork-Setup, Tauri-Port, Resampling/Beam-Search, CUDA-Flag, Icon, …) sind damit nicht mehr in der Git-Historie enthalten.

---

## 8 · Stand 07.06.2026 (GPU) & nächste Schritte

**Erledigt:** CUDA-Toolkit 13.3 installiert; whisper-rs auf **0.16** angehoben (0.12 bündelte altes whisper.cpp mit Arch-Default `52;61;70`, von CUDA 13.x nicht mehr unterstützt). GPU-Zielarchitekturen via `CMAKE_CUDA_ARCHITECTURES="75-real;86-real;89-real;120-real;75-virtual"` (Turing–Blackwell + PTX-Fallback) in der lokalen `src-tauri/.cargo/config.toml`. **Validiert:** `large-v3` (3094 MB) auf `CUDA0` (Blackwell), Transkription + Korrektur ok, Tempo deutlich besser, Beam-Search aktiv. **Zwei Installer** gebaut: CPU (universell, ~3,5 MB) und CUDA (mit gebündelter NVIDIA-Runtime cuBLAS, ~450 MB). Repo auf frische Historie zurückgesetzt, Windows-App ins Root verschoben.

**Offen:**
1. **GitHub-Release `v0.1.1`** veröffentlichen mit den vier Installer-Assets (CPU/CUDA je `.exe`/`.msi`). *(Hinweis: Im Repo liegen nur Quellen – Binärdateien ausschließlich als Release-Assets.)*
2. **Terminal-Paste-Fallback** (`Strg+Umschalt+V` bei erkannten Terminals).

> [!question] Spätere Ideen
> - Terminal-Paste-Fallback (`Strg+Umschalt+V` bei erkannten Terminals)
> - Onboarding-Feinschliff, Waveform-Anzeige
> - Gemini-Pro-Anbindung prüfen (falls `auth2api` das später unterstützt)

---

## Anhang · Schwierige Test-Sätze (Erkennungsqualität)

- Komposita: „Die Donaudampfschifffahrtsgesellschaftskapitänsmütze lag auf dem Formular."
- Zahlen/IBAN: „Am 3. November 2026 überwies ich 1.249,99 Euro auf DE89 3704 0044 0532 0130 00."
- Anglizismen: „Bitte deploye den Pull Request erst, wenn die CI-Pipeline grün ist."
- Zungenbrecher: „Zwischen zwei Zwetschgenzweigen zwitschern zwei Schwalben."
- Eigennamen: „Frau Schäuble fährt über Mönchengladbach nach Saarbrücken."
- Dialekt (Härteprobe): „Des hob i ned gschnoidn, owa passt scho, gell?"
