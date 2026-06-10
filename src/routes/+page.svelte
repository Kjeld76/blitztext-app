<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import {
    api,
    WORKFLOWS,
    type SettingsContainer,
    type CredentialStatus,
    type ModelMeta,
    type StatusEvent,
  } from "$lib/api";

  let settings = $state<SettingsContainer | null>(null);
  let cred = $state<CredentialStatus | null>(null);
  let models = $state<ModelMeta[]>([]);
  let autostart = $state(false);
  let active = $state(true);
  let status = $state<StatusEvent>({ phase: "idle", workflow: null, message: null });
  let view = $state<"main" | "settings">("main");
  let tab = $state<"anpassen" | "zugang">("anpassen");

  let llmToken = $state("");
  let sttKey = $state("");
  let testResult = $state("");
  let testing = $state(false);
  let saveHint = $state("");
  let downloads = $state<Record<string, number>>({});
  let capturingFor = $state<string | null>(null);
  let level = $state(0);

  // RMS → Balkenbreite in %, über dB-Skala (−52…−32 dB → 0…100 %);
  // identisch zur Anzeige im Sprech-Overlay (overlay.html).
  function levelToPercent(lvl: number): number {
    if (!lvl || lvl <= 0) return 0;
    const db = 20 * Math.log10(lvl);
    return Math.max(0, Math.min(100, ((db + 52) / 20) * 100));
  }

  const phaseLabel: Record<string, string> = {
    idle: "Bereit",
    recording: "Aufnahme läuft …",
    processing: "Verarbeite …",
    done: "Fertig",
    error: "Fehler",
  };

  onMount(() => {
    void reload();
    const unStatus = listen<StatusEvent>("status", (e) => {
      status = e.payload;
    });
    const unProg = listen<any>("model-progress", (e) => {
      const p = e.payload;
      downloads = { ...downloads, [p.name]: p.progress };
      if (p.done || p.progress >= 1) void refreshModels();
    });
    const unNav = listen<string>("navigate", (e) => {
      if (e.payload === "settings") view = "settings";
    });
    const unActive = listen<boolean>("active", (e) => {
      active = e.payload;
    });

    // Pegel pollen, solange aufgenommen wird.
    const timer = setInterval(async () => {
      if (status.phase === "recording") {
        try {
          level = await api.audioLevel();
        } catch {}
      } else {
        level = 0;
      }
    }, 80);

    const onKey = (ev: KeyboardEvent) => handleCaptureKey(ev);
    window.addEventListener("keydown", onKey, true);

    return () => {
      void unStatus.then((f) => f());
      void unProg.then((f) => f());
      void unNav.then((f) => f());
      void unActive.then((f) => f());
      clearInterval(timer);
      window.removeEventListener("keydown", onKey, true);
    };
  });

  async function reload() {
    settings = await api.getSettings();
    cred = await api.credentialStatus();
    autostart = await api.getAutostart();
    active = await api.getActive();
    await refreshModels();
  }

  async function refreshModels() {
    models = await api.listModels();
  }

  async function save() {
    if (!settings) return;
    settings.windows.launchAtLogin = autostart;
    await api.saveSettings($state.snapshot(settings) as SettingsContainer);
    saveHint = "Gespeichert ✓";
    setTimeout(() => (saveHint = ""), 1500);
  }

  async function saveToken() {
    await api.setCredential("llm_gateway_token", llmToken);
    llmToken = "";
    cred = await api.credentialStatus();
  }
  async function saveSttKey() {
    await api.setCredential("stt_api_key", sttKey);
    sttKey = "";
    cred = await api.credentialStatus();
  }

  async function test() {
    testing = true;
    testResult = "";
    try {
      const r = await api.testLlmConnection();
      testResult = "OK: " + r;
    } catch (e) {
      testResult = "Fehler: " + e;
    } finally {
      testing = false;
    }
  }

  async function toggleAutostart() {
    autostart = !autostart;
    await save();
  }

  function tile(key: string) {
    if (status.phase === "recording") {
      void api.stopRecording();
    } else {
      void api.startRecording(key);
    }
  }

  // --- Hotkey-Capture ---
  async function startCapture(key: string) {
    capturingFor = key;
    // Globale Hotkeys pausieren, sonst fängt das OS die Kombination ab.
    await api.setHotkeyCapture(true);
  }
  function handleCaptureKey(ev: KeyboardEvent) {
    if (!capturingFor) return;
    ev.preventDefault();
    ev.stopPropagation();
    if (ev.key === "Escape") {
      capturingFor = null;
      void api.setHotkeyCapture(false); // abbrechen -> alte Hotkeys reaktivieren
      return;
    }
    if (["Control", "Alt", "Shift", "Meta"].includes(ev.key)) return; // auf Hauptkey warten
    const mods: string[] = [];
    if (ev.ctrlKey) mods.push("Ctrl");
    if (ev.altKey) mods.push("Alt");
    if (ev.shiftKey) mods.push("Shift");
    if (ev.metaKey) mods.push("Super");
    let main = ev.key.length === 1 ? ev.key.toUpperCase() : ev.key;
    if (ev.code === "Space") main = "Space";
    const accel = [...mods, main].join("+");
    const target = capturingFor;
    capturingFor = null;
    if (settings) {
      settings.windows.hotkeys[target] = accel;
      // save() persistiert und registriert die Hotkeys neu (reaktiviert sie).
      void save();
    }
  }

  function fmtMb(mb: number) {
    return mb >= 1000 ? (mb / 1000).toFixed(1) + " GB" : mb + " MB";
  }
</script>

<div class="app">
  <header>
    <div class="brand"><img class="brand-icon" src="/icon.png" alt="Blitztext" /> Blitztext <span class="ver">Windows</span></div>
    <nav>
      <button class:active={view === "main"} onclick={() => (view = "main")}>Start</button>
      <button class:active={view === "settings"} onclick={() => (view = "settings")}>Einstellungen</button>
      <button
        class="pause"
        class:paused={!active}
        title={active ? "Hotkeys pausieren" : "Hotkeys aktivieren"}
        onclick={() => api.setActive(!active)}
      >{active ? "⏸" : "▶"}</button>
      <button class="close" title="Ins Tray schließen" aria-label="Schließen" onclick={() => api.hideWindow()}>✕</button>
    </nav>
  </header>

  <div class="statusbar" data-phase={status.phase} class:paused={!active}>
    <span class="dot"></span>
    {#if !active}
      <span>Pausiert – Hotkeys aus</span>
    {:else}
      <span>{phaseLabel[status.phase] ?? status.phase}</span>
      {#if status.message}<span class="msg">— {status.message}</span>{/if}
    {/if}
  </div>

  {#if status.phase === "recording"}
    <div class="meter"><div class="fill" style="width:{levelToPercent(level)}%"></div></div>
  {/if}

  {#if !settings}
    <p class="loading">Lade …</p>
  {:else if view === "main"}
    <main>
      <p class="hint">
        Hotkey {settings.app.hotkeyMode === "hold" ? "halten" : "drücken"} oder Kachel anklicken,
        sprechen, fertig. Der Text wird automatisch eingefügt.
      </p>
      <div class="tiles">
        {#each WORKFLOWS as w}
          <button
            class="tile"
            class:recording={status.phase === "recording" && status.workflow === w.key}
            onclick={() => tile(w.key)}
          >
            <div class="t-name">{w.name}</div>
            <div class="t-sub">{w.subtitle}</div>
            <div class="t-key">{settings.windows.hotkeys[w.key] ?? "—"}</div>
          </button>
        {/each}
      </div>
      {#if status.phase === "recording"}
        <button class="stop" onclick={() => api.stopRecording()}>■ Stoppen &amp; verarbeiten</button>
      {/if}
    </main>
  {:else}
    <main class="settings">
      <div class="tabs">
        <button class:active={tab === "anpassen"} onclick={() => (tab = "anpassen")}>Anpassen</button>
        <button class:active={tab === "zugang"} onclick={() => (tab = "zugang")}>Zugang</button>
      </div>

      {#if tab === "anpassen"}
        <section>
          <h3>Hotkey-Modus</h3>
          <label class="row">
            <select bind:value={settings.app.hotkeyMode} onchange={save}>
              <option value="hold">Halten (Taste halten = aufnehmen)</option>
              <option value="toggle">Drücken (umschalten, Esc bricht ab)</option>
            </select>
          </label>
        </section>

        <section>
          <h3>Tastenbelegung</h3>
          {#each WORKFLOWS as w}
            <div class="row hk">
              <span class="hk-name">{w.name}</span>
              <code class="hk-val">{settings.windows.hotkeys[w.key] ?? "—"}</code>
              <button onclick={() => startCapture(w.key)}>
                {capturingFor === w.key ? "Taste drücken …" : "Ändern"}
              </button>
            </div>
          {/each}
          <button class="secondary" onclick={save}>Hotkeys speichern</button>
        </section>

        <section>
          <h3>Sprache &amp; Begriffe</h3>
          <label class="row">
            Sprache
            <input bind:value={settings.transcription.language} onblur={save} style="width:5rem" />
          </label>
          <label class="col">
            Eigennamen / Fachbegriffe (kommagetrennt)
            <input
              value={settings.textImprovement.customTerms.join(", ")}
              onblur={(e) => {
                settings!.textImprovement.customTerms = (e.currentTarget as HTMLInputElement).value
                  .split(",").map((s) => s.trim()).filter(Boolean);
                void save();
              }}
            />
          </label>
        </section>

        <section>
          <h3>Blitztext+ (Ton)</h3>
          <select bind:value={settings.textImprovement.tone} onchange={save}>
            <option value="formal">Formell</option>
            <option value="neutral">Neutral</option>
            <option value="casual">Locker</option>
          </select>
        </section>

        <section>
          <h3>Blitztext :) (Emoji-Dichte)</h3>
          <select bind:value={settings.emojiText.emojiDensity} onchange={save}>
            <option value="wenig">Wenig</option>
            <option value="mittel">Mittel</option>
            <option value="viel">Viel</option>
          </select>
        </section>

        <section>
          <h3>Blitztext $%&! (System-Prompt)</h3>
          <textarea rows="4" bind:value={settings.dampfAblassen.systemPrompt} onblur={save}></textarea>
        </section>
      {:else}
        <section>
          <h3>LLM-Gateway (auth2api / OpenAI-kompatibel)</h3>
          <label class="col">
            Basis-URL
            <input bind:value={settings.windows.gateway.baseUrl} onblur={save} placeholder="http://localhost:8080/v1" />
          </label>
          <label class="col">
            Korrektur-Modell (Ctrl+Alt+B) – leer = keine Korrektur
            <input bind:value={settings.windows.gateway.correctionModel} onblur={save} placeholder="claude-haiku-4-5" />
          </label>
          <label class="col">
            Modell „schnell" (Blitztext+ / Emoji)
            <input bind:value={settings.windows.gateway.fastModel} onblur={save} />
          </label>
          <label class="col">
            Modell „stark" ($%&! / beruhigen)
            <input bind:value={settings.windows.gateway.strongModel} onblur={save} />
          </label>
          <label class="col">
            Token (optional)
            <div class="row">
              <input type="password" bind:value={llmToken} placeholder={cred?.llmConfigured ? cred.llmMasked : "kein Token"} />
              <button onclick={saveToken}>Setzen</button>
            </div>
          </label>
          <div class="row">
            <button onclick={test} disabled={testing}>{testing ? "Teste …" : "Verbindung testen"}</button>
            {#if testResult}<span class="test-res">{testResult}</span>{/if}
          </div>
        </section>

        <section>
          <h3>Whisper-Modell (lokale Transkription)</h3>
          {#each models as m}
            <div class="row model">
              <span>{m.displayName} <small>({fmtMb(m.approxMb)})</small></span>
              {#if m.installed}
                <span class="ok">installiert ✓</span>
              {:else if downloads[m.name] !== undefined && downloads[m.name] < 1}
                <span>{Math.round(downloads[m.name] * 100)} %</span>
              {:else}
                <button onclick={() => api.downloadModel(m.name)}>Herunterladen</button>
              {/if}
            </div>
          {/each}
          <label class="col">
            Aktives Modell
            <select bind:value={settings.app.selectedLocalTranscriptionModelName} onchange={save}>
              {#each models as m}
                <option value={m.name} disabled={!m.installed}>
                  {m.displayName}{m.installed ? "" : " — nicht installiert"}
                </option>
              {/each}
            </select>
          </label>
        </section>

        <section>
          <h3>Online-STT (optional)</h3>
          <label class="col">
            STT Basis-URL (leer = nur lokal)
            <input bind:value={settings.windows.gateway.sttBaseUrl} onblur={save} placeholder="leer lassen für lokal" />
          </label>
          <label class="col">
            STT-Modell
            <input bind:value={settings.windows.gateway.sttModel} onblur={save} placeholder="whisper-1" />
          </label>
          <label class="col">
            STT-API-Key
            <div class="row">
              <input type="password" bind:value={sttKey} placeholder={cred?.sttConfigured ? cred.sttMasked : "kein Key"} />
              <button onclick={saveSttKey}>Setzen</button>
            </div>
          </label>
        </section>

        <section>
          <h3>System</h3>
          <label class="row">
            <input type="checkbox" checked={autostart} onchange={toggleAutostart} />
            Beim Windows-Start automatisch starten
          </label>
          <label class="row">
            <input type="checkbox" bind:checked={settings.app.prerollEnabled} onchange={save} />
            Mikrofon vorwärmen (kein Wortanfang verloren, sofortiger Start)
          </label>
          <p class="note">
            Hält das Mikrofon offen, solange Blitztext aktiv ist – Windows zeigt es dann
            dauerhaft als „in Benutzung". Bei Pause aus.
          </p>
          <label class="col">
            Einfügen (Tastenkürzel)
            <select bind:value={settings.windows.pasteShortcut} onchange={save}>
              <option value="auto">Automatisch (Terminals erkennen)</option>
              <option value="ctrlV">Immer Strg+V</option>
              <option value="ctrlShiftV">Immer Strg+Umschalt+V</option>
            </select>
          </label>
          <p class="note">
            Viele Terminals (z.&nbsp;B. Windows Terminal) fügen mit Strg+Umschalt+V ein.
            „Automatisch" erkennt sie und nutzt sonst Strg+V.
          </p>
        </section>
      {/if}
      {#if saveHint}<div class="save-hint">{saveHint}</div>{/if}
    </main>
  {/if}
</div>

<style>
  :global(body) { margin: 0; }
  .app {
    font-family: "Segoe UI", system-ui, sans-serif;
    color: #1c1c1e;
    background: #f6f6f8;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
  header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 14px; background: #fff; border-bottom: 1px solid #e3e3e7;
  }
  .brand { display: flex; align-items: center; gap: 6px; font-weight: 700; }
  .brand-icon { width: 20px; height: 20px; display: block; }
  .ver { font-weight: 400; font-size: .75rem; color: #888; }
  nav button, .tabs button {
    border: none; background: transparent; padding: 6px 10px; cursor: pointer;
    border-radius: 7px; color: #555;
  }
  nav button.active, .tabs button.active { background: #2962ff; color: #fff; }
  nav .pause { margin-left: 4px; }
  nav .pause.paused { background: #fb8c00; color: #fff; }
  nav .close { color: #e53935; font-weight: 700; margin-left: 4px; }
  nav .close:hover, nav .close:active { background: #e53935; color: #fff; }
  .statusbar.paused .dot { background: #fb8c00; }
  .statusbar {
    display: flex; align-items: center; gap: 8px; padding: 6px 14px; font-size: .85rem;
    background: #fff; border-bottom: 1px solid #eee;
  }
  .statusbar .dot { width: 9px; height: 9px; border-radius: 50%; background: #aaa; }
  .statusbar[data-phase="recording"] .dot { background: #e53935; }
  .statusbar[data-phase="processing"] .dot { background: #fb8c00; }
  .statusbar[data-phase="done"] .dot { background: #43a047; }
  .statusbar[data-phase="error"] .dot { background: #e53935; }
  .msg { color: #777; }
  .meter { height: 6px; background: #eee; }
  .meter .fill { height: 100%; background: #e53935; transition: width .08s linear; }
  main { padding: 14px; overflow-y: auto; }
  .hint { color: #666; font-size: .85rem; margin-top: 0; }
  .note { color: #888; font-size: .75rem; margin: 4px 0 0; }
  .tiles { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  .tile {
    text-align: left; border: 1px solid #e3e3e7; background: #fff; border-radius: 12px;
    padding: 12px; cursor: pointer;
  }
  .tile:hover { border-color: #2962ff; }
  .tile.recording { border-color: #e53935; box-shadow: 0 0 0 2px #e5393533; }
  .t-name { font-weight: 600; }
  .t-sub { font-size: .75rem; color: #888; margin: 2px 0 8px; }
  .t-key { font-size: .72rem; color: #2962ff; font-family: ui-monospace, monospace; }
  .stop {
    margin-top: 12px; width: 100%; padding: 10px; border: none; border-radius: 10px;
    background: #e53935; color: #fff; cursor: pointer; font-weight: 600;
  }
  .settings section {
    background: #fff; border: 1px solid #e3e3e7; border-radius: 12px; padding: 12px; margin-bottom: 12px;
  }
  .settings h3 { margin: 0 0 8px; font-size: .9rem; }
  .row { display: flex; align-items: center; gap: 8px; }
  .col { display: flex; flex-direction: column; gap: 4px; margin-bottom: 8px; }
  .hk { justify-content: space-between; margin-bottom: 6px; }
  .hk-name { flex: 1; }
  .hk-val { background: #f0f0f3; padding: 2px 6px; border-radius: 6px; min-width: 90px; text-align: center; }
  input, select, textarea {
    border: 1px solid #ccc; border-radius: 7px; padding: 6px 8px; font: inherit; background: #fff;
  }
  .col input, textarea { width: 100%; box-sizing: border-box; }
  button { border: 1px solid #ccc; background: #fff; border-radius: 7px; padding: 6px 10px; cursor: pointer; }
  button:hover { border-color: #2962ff; }
  .secondary { margin-top: 6px; }
  .ok { color: #43a047; font-size: .85rem; }
  .test-res { font-size: .85rem; }
  .save-hint {
    position: sticky; bottom: 0; text-align: center; color: #43a047; font-size: .85rem; padding: 6px;
  }
  .tabs { display: flex; gap: 6px; margin-bottom: 12px; }
  .loading { padding: 14px; color: #888; }
  @media (prefers-color-scheme: dark) {
    .app { background: #1c1c1e; color: #f2f2f2; }
    header, .statusbar, .tile, .settings section { background: #2a2a2e; border-color: #3a3a3e; }
    .hk-val { background: #3a3a3e; }
    input, select, textarea, button { background: #1c1c1e; color: #f2f2f2; border-color: #444; }
    nav button, .tabs button { color: #bbb; }
  }
</style>
