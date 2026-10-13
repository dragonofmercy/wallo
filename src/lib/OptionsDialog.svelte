<script lang="ts">
  import { onMount } from "svelte";
  import { getAutostart, getCheckUpdates, getKeepWallpaper, setAutostart, setCheckUpdates, setKeepWallpaper } from "./api";
  import { app } from "./state.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let dialog: HTMLDialogElement;

  onMount(() => dialog.showModal());

  // Not profile settings: applied immediately, read back from the backend.
  let autostart = $state(false);
  let keep = $state(false);
  let checkUpdates = $state(true);
  let error = $state("");
  getAutostart().then(
    (v) => (autostart = v),
    () => {},
  );
  getKeepWallpaper().then(
    (v) => (keep = v),
    () => {},
  );
  getCheckUpdates().then(
    (v) => (checkUpdates = v),
    () => {},
  );

  function checkUpdatesSaved(enabled: boolean) {
    checkUpdates = enabled;
    if (enabled) app.checkUpdate();
  }

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
  <label class="option">
    <span class="text"><strong>Start with Windows</strong><span>Wallo starts in the tray when you sign in, without opening the window.</span></span>
    <span class="switch"><input type="checkbox" role="switch" checked={autostart} onchange={(e) => toggle(e, setAutostart, (v) => (autostart = v))} /><span class="track"></span></span>
  </label>
  <label class="option">
    <span class="text"><strong>Keep wallpaper applied</strong><span>Reapplies the active profile when monitors change or another program changes the wallpaper.</span></span>
    <span class="switch"><input type="checkbox" role="switch" checked={keep} onchange={(e) => toggle(e, setKeepWallpaper, (v) => (keep = v))} /><span class="track"></span></span>
  </label>
  <label class="option">
    <span class="text"><strong>Check for updates automatically</strong><span>Looks for a new Wallo version each time the window opens.</span></span>
    <span class="switch"><input type="checkbox" role="switch" checked={checkUpdates} onchange={(e) => toggle(e, setCheckUpdates, checkUpdatesSaved)} /><span class="track"></span></span>
  </label>
  {#if error}<p class="error selectable">{error}</p>{/if}
  <div class="buttons">
    <button type="button" class="btn ghost" onclick={onclose}>Close</button>
  </div>
</dialog>

<style>
  dialog {
    width: 480px;
  }
  .option {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 14px 0;
    cursor: pointer;
  }
  .option + .option {
    border-top: 1px solid var(--line);
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .text span {
    color: var(--muted);
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
