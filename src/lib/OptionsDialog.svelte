<script lang="ts">
  import { onMount } from "svelte";
  import { getAutostart, getKeepWallpaper, setAutostart, setKeepWallpaper } from "./api";

  let { onclose }: { onclose: () => void } = $props();

  let dialog: HTMLDialogElement;

  onMount(() => dialog.showModal());

  // Not profile settings: applied immediately, read back from the backend.
  let autostart = $state(false);
  let keep = $state(false);
  let error = $state("");
  getAutostart().then(
    (v) => (autostart = v),
    () => {},
  );
  getKeepWallpaper().then(
    (v) => (keep = v),
    () => {},
  );

  async function toggle(e: Event, save: (enabled: boolean) => Promise<void>, saved: (enabled: boolean) => void) {
    const input = e.currentTarget as HTMLInputElement;
    error = "";
    try {
      await save(input.checked);
      saved(input.checked);
    } catch (err) {
      input.checked = !input.checked;
      error = String(err);
    }
  }
</script>

<dialog bind:this={dialog} oncancel={(e) => (e.preventDefault(), onclose())}>
  <h2>Options</h2>
  <label class="chk"><input type="checkbox" checked={autostart} onchange={(e) => toggle(e, setAutostart, (v) => (autostart = v))} /><span class="box"><svg viewBox="0 0 12 12"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg></span>Start with Windows</label>
  <label class="chk"><input type="checkbox" checked={keep} onchange={(e) => toggle(e, setKeepWallpaper, (v) => (keep = v))} /><span class="box"><svg viewBox="0 0 12 12"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg></span>Keep wallpaper applied</label>
  <p class="hint">Reapplies the active profile when monitors change or another program changes the wallpaper.</p>
  {#if error}<p class="error selectable">{error}</p>{/if}
  <div class="buttons">
    <button type="button" class="btn primary" onclick={onclose}>Close</button>
  </div>
</dialog>

<style>
  dialog {
    width: 360px;
  }
  .chk + .chk {
    margin-top: 10px;
  }
  .hint {
    color: var(--muted);
    margin: 4px 0 0 28px;
  }
  .error {
    color: var(--danger);
    margin: 10px 0 0;
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    margin-top: 18px;
  }
</style>
