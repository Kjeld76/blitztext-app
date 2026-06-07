[English](SECURITY.md) · **Deutsch**

# Sicherheitsrichtlinie

Blitztext (Windows) ist experimentelle Software – „as is", ohne Gewähr, Support-Garantie oder Produktionsreife.

> Diese Richtlinie betrifft die Windows-App in diesem Repository. Das macOS-Original liegt im [Original-Repository](https://github.com/cmagnussen/blitztext-app).

## Unterstützte Versionen

Nur der aktuelle Entwicklungs-Branch wird für Sicherheitskorrekturen berücksichtigt.

## Schwachstellen melden

Bitte **keine** öffentlichen Issues mit sensiblen Sicherheitsdetails eröffnen.

Nutze die **private Vulnerability-Reporting**-Funktion von GitHub für dieses Repository. Ist sie nicht verfügbar, öffne ein minimales Issue mit dem Titel `Security contact request` ohne technische Details.

Keine API-Keys/Token, privaten Aufnahmen oder vertraulichen Transkripte in einem Report mitschicken.

Bitte angeben:

- was du gefunden hast
- wie es reproduzierbar ist
- welche Daten/Systemzugriffe betroffen sein könnten
- ein Lösungsvorschlag, falls vorhanden

## Sicherheitshinweise (Windows)

- **Transkription läuft standardmäßig lokal** auf dem Gerät (whisper.cpp) – kein Cloud-Versand.
- Die **KI-Umschreibung** sendet den (transkribierten) **Text** an den von dir konfigurierten OpenAI-kompatiblen Endpunkt (z. B. einen lokal laufenden auth2api-Proxy).
- Ein **optionaler Online-STT-Endpunkt** sendet Audio an den konfigurierten Dienst – nur, wenn du ihn einrichtest (Standard: leer = lokal).
- Zugangs-Token liegt im **Windows Credential Manager**, nicht im Klartext-JSON.
- Temporäre Audiodateien können während der Verarbeitung kurz existieren.
- **Auto-Paste** erfolgt per simuliertem Strg+V (SendInput) – ohne besondere Systemberechtigung.
- **Globale Hotkeys** sind systemweit aktiv (im UI/Tray pausierbar).
- Hinweis: Ein Reverse-Proxy wie auth2api, der ein Abo als API bereitstellt, kann den AGB des jeweiligen Anbieters widersprechen. Nutzung auf eigene Verantwortung.

Nutze diese Preview nicht für vertrauliche oder regulierte Daten ohne eigene Prüfung.
