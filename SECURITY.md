**English** · [Deutsch](SECURITY.de.md)

# Security Policy

Blitztext (Windows) is experimental software — provided "as is", without warranty, support guarantee, or production readiness.

> This policy concerns the Windows app in this repository. The macOS original lives in the [original repository](https://github.com/cmagnussen/blitztext-app).

## Supported versions

Only the current development branch is considered for security fixes.

## Reporting vulnerabilities

Please do **not** open public issues with sensitive security details.

Use GitHub's **private vulnerability reporting** feature for this repository. If it is unavailable, open a minimal issue titled `Security contact request` without technical details.

Do not include any API keys/tokens, private recordings, or confidential transcripts in a report.

Please include:

- what you found
- how it is reproducible
- which data/system access could be affected
- a proposed fix, if you have one

## Security notes (Windows)

- **Transcription runs locally by default** on the device (whisper.cpp) — nothing is sent to the cloud.
- **AI rewriting** sends the (transcribed) **text** to the OpenAI-compatible endpoint you configured (e.g. a locally running auth2api proxy).
- An **optional online STT endpoint** sends audio to the configured service — only if you set it up (default: empty = local).
- The access token is stored in the **Windows Credential Manager**, not in plaintext JSON.
- Temporary audio files may briefly exist during processing.
- **Auto-paste** is done via a simulated Ctrl+V (SendInput) — without special system permissions.
- **Global hotkeys** are active system-wide (pausable in the UI/tray).
- Note: a reverse proxy like auth2api that exposes a subscription as an API may conflict with the respective provider's terms of service. Use at your own risk.

Do not use this preview for confidential or regulated data without your own review.
