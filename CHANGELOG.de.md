[English](CHANGELOG.md) · **Deutsch**

# Changelog — Blitztext für Windows

Alle nennenswerten Änderungen an der Windows-Portierung von Blitztext werden hier dokumentiert.

Das Format orientiert sich an [Keep a Changelog](https://keepachangelog.com/de/1.1.0/),
und das Projekt folgt [Semantic Versioning](https://semver.org/lang/de/).
Die Windows-App hat eine **eigene Versionsspur** (unabhängig von der macOS-App), beginnend bei `0.1.0`.

## [0.2.0] - 2026-06-11

Komfort-Release: Sprech-Overlay (Pille) mit Live-Pegel, optionales Mikrofon-Vorwärmen,
terminal-bewusstes Einfügen — dazu Fixes für abgeschnittene Aufnahme-Anfänge und
Transkripte, die beantwortet statt bearbeitet wurden.

### Hinzugefügt
- **Sprech-Overlay (Pille)**: Während der Aufnahme zeigt eine kleine Pille **oben mittig**
  auf dem Bildschirm den Status und einen **Live-Pegel**. Oben platziert, damit das
  Windows-Lautstärke-Flyout (das manche Headsets beim Öffnen des Mikrofons auslösen) sie
  nicht verdeckt. Der Pegel nutzt eine dB-Skala (−52…−32 dB), die Anzeige im Hauptfenster
  wurde angeglichen — leise Mikrofone zeigen jetzt einen brauchbaren Ausschlag.
- **Mikrofon-Vorwärmen (Pre-Roll)**, optional: hält das Eingabegerät warm, sodass eine
  Aufnahme sofort startet, ohne auf das Öffnen des Geräts zu warten. Neue Einstellung
  unter *System* (`app.prerollEnabled`).
- **Terminal-bewusstes Einfügen**: Auto-Paste prüft das Vordergrundfenster (Fensterklasse +
  Prozessname) und sendet in bekannten Terminals (Windows Terminal, wezterm, alacritty, …)
  **Strg+Umschalt+V** statt Strg+V. Neue Einstellung `windows.pasteShortcut` unter *System*:
  Automatisch / immer Strg+V / immer Strg+Umschalt+V. Klassisches conhost bleibt bewusst
  bei Strg+V. Damit ist die Terminal-Einschränkung aus „Bekannt" in 0.1.1 behoben.
- **Unit-Tests + CI**: 12 Unit-Tests (Transkript-Qualitätsfilter, Prompt-Aufbau);
  die CI führt `cargo test` unter Windows aus und ist auch manuell startbar.

### Behoben
- **Anfang der Aufnahme wurde abgeschnitten**: Die UI signalisierte „Aufnahme", bevor das
  Mikrofon wirklich live war — die ersten Worte gingen verloren, während WASAPI das
  Eingabegerät öffnete (~100–300 ms). Der Aufnahmestart wartet jetzt, bis der Audio-Stream
  tatsächlich läuft, und stempelt die Dauer erst ab diesem Zeitpunkt.
- **Eingesprochener Satz wurde mitunter beantwortet statt verbessert**: War ein Transkript
  wie eine Frage oder Bitte formuliert (z. B. „Kannst du mir einen Termin finden?"), konnte
  das LLM darauf antworten, statt nur den Wortlaut zu korrigieren/verbessern — und diese
  Antwort wurde eingefügt. Das Transkript wird jetzt in eindeutige Marker eingefasst, und
  jedes Text-Prompt erhält eine Schutzklausel, die das Modell anweist, die Eingabe strikt
  als zu bearbeitenden Text zu behandeln — niemals als Anweisung. Gilt für alle
  Text-Workflows (Korrektur, Verbesserer, Emoji, Dampf ablassen), auch für eigene Prompts.
- **Sprech-Pille verschwand bei schnellen Folgeaufnahmen**: Ein verzögertes „idle"-Signal
  (1,1 s nach Abschluss eines Workflows) versteckte die Pille einer bereits wieder
  laufenden Aufnahme. „idle" wird jetzt nur noch gemeldet, wenn die Engine nicht
  beschäftigt ist; Fehler beim Zeigen/Verstecken des Overlays werden geloggt.

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
