**English** · [Deutsch](THIRD-PARTY-NOTICES.de.md)

# Third-Party Notices

Blitztext for Windows bundles or uses third-party components. Their respective
license terms continue to apply.

## NVIDIA CUDA runtime (GPU/CUDA installer only)

The **GPU installer** (`…-cuda-setup.exe` / `…-cuda…_en-US.msi`) includes
unmodified, redistributable runtime libraries from the NVIDIA® CUDA® Toolkit:

- `cudart64_13.dll` — CUDA Runtime
- `cublas64_13.dll` — cuBLAS
- `cublasLt64_13.dll` — cuBLASLt

© NVIDIA Corporation. These libraries are explicitly listed as *redistributable*
in "Attachment A" of the CUDA Toolkit EULA and are shipped as part of the
application under that EULA (unmodified, used exclusively by Blitztext). The
currently applicable EULA governs:

<https://docs.nvidia.com/cuda/eula/>

The **CPU installer** does not include these libraries and does not need them.

## whisper.cpp / ggml

Local transcription via [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
(ggml), integrated through [`whisper-rs`](https://github.com/tazz4843/whisper-rs).
License: MIT.

## Other open-source components

Tauri and SvelteKit, as well as numerous Rust and npm packages, are under their
respective licenses (mostly MIT/Apache-2.0). See the individual projects or
`src-tauri/Cargo.toml` and `package.json`.
