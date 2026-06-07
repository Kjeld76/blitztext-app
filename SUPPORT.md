# Support

Blitztext (Windows) ist eine experimentelle Preview. Es gibt **kein** SLA, keinen bezahlten Support und keine Garantie, dass Probleme behoben werden.

> Betrifft die Windows-App in diesem Repository; das macOS-Original liegt im [Original-Repository](https://github.com/cmagnussen/blitztext-app).

## Bevor du um Hilfe bittest

- Stelle sicher, dass die App baut/startet: `pnpm install && pnpm tauri dev` (bzw. `pnpm tauri build`).
- Prüfe den **LLM-Gateway** in den Einstellungen (Basis-URL/Modell/Token) und teste die Verbindung.
- Prüfe die **Mikrofon-Berechtigung**: Windows-Einstellungen → Datenschutz → Mikrofon → „Desktop-Apps Zugriff erlauben".
- Stelle sicher, dass das **Whisper-Modell** (`large-v3`) in den Einstellungen heruntergeladen/aktiv ist.

## Wo fragen

GitHub Issues für reproduzierbare Fehler und fokussierte Feature-Ideen.

Bitte **nicht** posten:

- API-Keys / Token
- private Audioaufnahmen
- vertrauliche Transkripte
- Screenshots mit sensiblen Inhalten

Für sicherheitsrelevante Meldungen siehe [SECURITY.md](SECURITY.md) statt eines öffentlichen Issues.
