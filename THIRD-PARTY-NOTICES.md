# Drittanbieter-Hinweise (Third-Party Notices)

Blitztext für Windows bündelt bzw. nutzt Komponenten Dritter. Deren jeweilige
Lizenzbedingungen gelten fort.

## NVIDIA CUDA Runtime (nur im GPU-/CUDA-Installer)

Der **GPU-Installer** (`…-cuda-setup.exe` bzw. `…-cuda…_en-US.msi`) enthält
unveränderte, weiterverteilbare Laufzeitbibliotheken des NVIDIA® CUDA® Toolkits:

- `cudart64_13.dll` — CUDA Runtime
- `cublas64_13.dll` — cuBLAS
- `cublasLt64_13.dll` — cuBLASLt

© NVIDIA Corporation. Diese Bibliotheken sind in „Attachment A" der
CUDA-Toolkit-EULA ausdrücklich als *redistributable* gelistet und werden gemäß
dieser EULA als Teil der Anwendung mitgeliefert (unverändert, ausschließlich von
Blitztext genutzt). Maßgeblich ist die jeweils aktuelle EULA:

<https://docs.nvidia.com/cuda/eula/>

Der **CPU-Installer** enthält diese Bibliotheken nicht und benötigt sie nicht.

## whisper.cpp / ggml

Lokale Transkription über [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
(ggml), eingebunden via [`whisper-rs`](https://github.com/tazz4843/whisper-rs).
Lizenz: MIT.

## Weitere Open-Source-Bestandteile

Tauri und SvelteKit sowie zahlreiche Rust- und npm-Pakete stehen unter ihren
jeweiligen Lizenzen (überwiegend MIT/Apache-2.0). Siehe die jeweiligen Projekte
bzw. `src-tauri/Cargo.toml` und `package.json`.
