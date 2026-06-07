# Changelog — Blitztext für Windows

Alle nennenswerten Änderungen an der Windows-Portierung von Blitztext werden hier dokumentiert.

Das Format orientiert sich an [Keep a Changelog](https://keepachangelog.com/de/1.1.0/),
und das Projekt folgt [Semantic Versioning](https://semver.org/lang/de/).
Die Windows-App hat eine **eigene Versionsspur** (unabhängig von der macOS-App), beginnend bei `0.1.0`.

## [Unveröffentlicht]

## [0.1.1] - 2026-06-07

Erstes öffentliches Release mit **GPU-Beschleunigung**. Enthält alle Verbesserungen seit `0.1.0`.

### Hinzugefügt
- **GPU-Beschleunigung (CUDA)** aktiv: `large-v3` läuft auf der NVIDIA-GPU
  (getestet auf RTX PRO 4000, Blackwell `sm_120`) — deutlich schneller als auf CPU.
  Der GPU-Build aktiviert zugleich **Beam-Search** für höhere Genauigkeit.
- **Pausieren/Aktivieren** ohne Beenden: globaler Schalter im Header und im Tray-Menü;
  pausiert meldet alle globalen Hotkeys ab (App bleibt im Tray).
- **Tray-Icon zeigt den Zustand**: farbig = aktiv, Graustufen = pausiert.
- **Fenster schließbar**: Schließen-Button (✕) blendet ins Tray; Tray-Linksklick schaltet
  das Fenster um (sichtbar ⇄ versteckt).
- **Blitztext (B)** macht nach der lokalen Transkription eine **leichte Claude-Korrektur**
  (Haiku, nur Zeichensetzung/Tippfehler); fällt bei nicht erreichbarem Gateway auf das
  rohe Transkript zurück. Neues Setting `correctionModel`.
- **GPU-Feature-Flag** (`--features cuda`): aktiviert zugleich Beam-Search (genauer).
- Modell-Dropdown zeigt den **Installationsstatus** (nicht installierte deaktiviert).
- **Eigenes App-/Tray-Icon** (Blitz + Sprechblasen) statt Tauri-Standard; auch im
  Fenster-Header und als Favicon. Tray farbig = aktiv, Graustufen = pausiert.

### Geändert
- **whisper-rs auf 0.16** aktualisiert (von 0.12): unterstützt CUDA 13.x und die
  Blackwell-Architektur (`sm_120`); CUDA-Zielarchitektur über
  `CMAKE_CUDA_ARCHITECTURES` konfigurierbar.
- **Audio-Resampling** auf 16 kHz mit Anti-Aliasing-Tiefpass → bessere Erkennung.
- **Hotkey-Modi**: `R` (Beruhigen) nutzt jetzt **Sonnet** statt Opus.
- Fenster nicht mehr „always on top".

### Behoben
- Hotkey-**Erfassung**: globale Shortcuts werden während des Erfassens pausiert, sodass
  Kombinationen wie `Ctrl+Alt+B` (statt nur Einzeltasten) zuweisbar sind.
- UI-Serialisierung (camelCase): behebt „undefined MB", leeres Modell-Dropdown und
  falsche Token-Statusanzeige.

### Bekannt
- **Auto-Paste** nutzt simuliertes `Strg+V` und klappt in normalen Programmen
  (z. B. Editor, Obsidian). Manche **Terminals** belegen `Strg+V` nicht zum Einfügen
  (dort Rechtsklick bzw. `Strg+Umschalt+V`) — ein Terminal-Fallback ist geplant.

## [0.1.0] - 2026-06-06

Erste lauffähige Windows-Version (Tauri, Rust + SvelteKit). Startet als Tray-App.

### Hinzugefügt
- **5 Workflows** portiert aus der macOS-App: Blitztext, Blitztext Lokal, Blitztext+,
  Blitztext $%&!, Blitztext :) — LLM-Prompts wörtlich übernommen.
- **Lokale Transkription** via whisper.cpp (`whisper-rs`) mit Modell `large-v3`
  (höchste Genauigkeit); In-App-Modell-Download mit Fortschritt. Optionaler Online-STT.
- **LLM-Umschreibung** über konfigurierbaren OpenAI-kompatiblen Endpunkt
  (base_url + Modell + Token) — für auth2api/Ollama/OpenAI. „Verbindung testen".
- **Globale Hotkeys**, frei konfigurierbar (Hold/Toggle, Esc-Abbruch), Defaults
  `Ctrl+Alt+B/L/P/R/E`.
- **Auto-Paste** via Zwischenablage + simuliertes Strg+V (keine Sonderrechte nötig).
- **Tray-Icon** mit Statusanzeige; **Autostart** beim Windows-Start (umschaltbar).
- Zugangsdaten im **Windows Credential Manager**; Einstellungen als JSON in `%APPDATA%\Blitztext`.
- Einstellungs-UI (Hotkey-Editor, Gateway-Konfig, Modellverwaltung, Autostart).

### Bekannte Einschränkungen
- GPU-Beschleunigung (CUDA) noch nicht standardmäßig aktiviert — `large-v3` läuft zunächst auf CPU.
- End-to-End-Funktionstest (Aufnahme→Transkript→Umschreiben) erfordert heruntergeladenes
  Whisper-Modell und laufenden auth2api-Proxy.
