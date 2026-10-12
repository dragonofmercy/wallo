<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getAutostart, setAutostart } from "./api";
  import Icon from "./Icon.svelte";
  import { app } from "./state.svelte";

  const close = () => getCurrentWindow().close();

  async function ok() {
    if (await app.apply()) await close();
  }

  // Not a profile setting: applied immediately, read back from the registry.
  let autostart = $state(false);
  getAutostart().then(
    (v) => (autostart = v),
    () => {},
  );

  async function toggleAutostart(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    try {
      await setAutostart(input.checked);
      autostart = input.checked;
    } catch (err) {
      input.checked = autostart;
      app.banner = { kind: "error", text: String(err) };
    }
  }
</script>

<footer>
  <label class="chk autostart"><input type="checkbox" checked={autostart} onchange={toggleAutostart} /><span class="box"><svg viewBox="0 0 12 12"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg></span>Start with Windows</label>
  <button type="button" class="btn ghost" disabled={app.busy} onclick={close}>Cancel</button>
  <button type="button" class="btn" disabled={app.busy} onclick={() => app.apply()}><Icon name="check" />{app.busy ? "Applying..." : "Apply"}</button>
  <button type="button" class="btn primary ok" disabled={app.busy} onclick={ok}>OK</button>
</footer>

<style>
  footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
  }
  .autostart {
    margin-right: auto;
    margin-left: 6px;
  }
  .ok {
    min-width: 96px;
    justify-content: center;
  }
</style>
