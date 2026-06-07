# Mitwirken

Danke, dass du dir Blitztext (Windows) ansiehst.

Dieses Repository ist bewusst ein **Preview/Lernprojekt**. Beiträge sollen es leichter machen, das Projekt zu verstehen, zu bauen, zu forken oder sicher zu erweitern.

> Hinweis: Dieses Repository enthält die **Windows-App**. Das ursprüngliche macOS-Projekt (von cmagnussen) liegt im [Original-Repository](https://github.com/cmagnussen/blitztext-app) – siehe Herkunft in der [README](README.md).

## Gute erste Beiträge

- Build-Anleitung verbessern
- verwirrende UI-Texte korrigieren
- bessere Fehlermeldungen
- Tests rund um Prompt-Aufbau und Qualitätsfilter
- Setup vereinfachen
- eigenes App-Icon / Branding

## Vor einem Pull Request

Bitte beschreibe:

- was geändert wurde
- warum
- wie du es getestet hast

Halte Änderungen klein und fokussiert; keine unzusammenhängenden Aufräumarbeiten im selben PR.

## Lokaler Build (Windows)

```powershell
pnpm install
pnpm tauri dev          # starten (Entwicklung)
pnpm tauri build        # Installer (MSI/NSIS)
```

Voraussetzungen und libclang-Hinweis siehe [README.md](README.md).
Immer über die **Tauri-CLI** bauen, nicht über nacktes `cargo build`.

## Sicherheit & Datenschutz

- **Niemals** API-Keys, Token, private Audios oder vertrauliche Transkripte committen.
- Keine Telemetrie, gehosteten Dienste oder externen Abhängigkeiten ohne vorherige Diskussion (Issue).
- Datenschutzrelevante Änderungen im PR klar benennen.
- Ehrlich bleiben: einen Cloud-Pfad (z. B. KI-Umschreibung über einen Gateway) nicht als „offline/lokal" beschreiben.

## Projekt-Rahmen

Im Fokus steht die Windows-App in diesem Repository. Größere Richtungsänderungen bitte zuerst in einem Issue besprechen.
