# GitHub Settings Checklist

These settings are not stored in the repository. They were applied via `gh` (status: 2026-06-10); re-check after major changes.

## Security

- [x] Dependabot alerts enabled.
- [x] Secret scanning enabled.
- [x] Push protection for supported secret types enabled.
- [x] Private vulnerability reporting enabled.

## Branch Protection

`main` is protected with a deliberate solo-maintainer compromise: direct
commits stay allowed (no pull-request requirement), but the history is
protected against accidents.

- [x] block force pushes
- [x] block branch deletion
- [ ] require pull request + approval + CI — intentionally **not** enabled
      while the project is maintained solo with direct commits to `main`;
      revisit when there is more than one regular contributor.

## Actions

- [x] Default workflow permissions read-only (`permissions: contents: read` in the workflow).
- [x] Require approval for workflows from first-time contributors (GitHub default).
- [x] No repository secrets configured.

## Community

- [x] Issues enabled for bugs and focused requests.
- [x] Discussions disabled for now (revisit on demand).
- [x] Repository topics set (tauri, rust, sveltekit, whisper, speech-to-text, dictation, windows, cuda, llm, …).
- [x] Repository description set.
- [ ] Social preview image: upload manually under *Settings → General → Social preview* (1280×640 PNG; cannot be set via API).
