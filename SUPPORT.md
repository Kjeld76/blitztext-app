**English** · [Deutsch](SUPPORT.de.md)

# Support

Blitztext (Windows) is an experimental preview. There is **no** SLA, no paid support, and no guarantee that issues will be fixed.

> Concerns the Windows app in this repository; the macOS original lives in the [original repository](https://github.com/cmagnussen/blitztext-app).

## Before asking for help

- Make sure the app builds/starts: `pnpm install && pnpm tauri dev` (or `pnpm tauri build`).
- Check the **LLM gateway** in the settings (base URL/model/token) and test the connection.
- Check the **microphone permission**: Windows Settings → Privacy → Microphone → "Let desktop apps access your microphone".
- Make sure the **Whisper model** (`large-v3`) is downloaded/active in the settings.

## Where to ask

GitHub Issues for reproducible bugs and focused feature ideas.

Please do **not** post:

- API keys / tokens
- private audio recordings
- confidential transcripts
- screenshots with sensitive content

For security-relevant reports, see [SECURITY.md](SECURITY.md) instead of a public issue.
