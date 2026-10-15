<script lang="ts">
  import { onMount } from "svelte";
  import { getAutostart, getCheckUpdates, getKeepWallpaper, setAutostart, setCheckUpdates, setKeepWallpaper } from "./api";
  import { app } from "./state.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let dialog: HTMLDialogElement;

  onMount(() => dialog.showModal());

  // Not profile settings: applied immediately, read back from the backend.
  const options = $state([
    { title: "Start with Windows", desc: "Wallo starts in the tray when you sign in, without opening the window.", get: getAutostart, set: setAutostart, on: false },
    { title: "Keep wallpaper applied", desc: "Reapplies the active profile when monitors change or another program changes the wallpaper.", get: getKeepWallpaper, set: setKeepWallpaper, on: false },
    { title: "Check for updates automatically", desc: "Looks for a new Wallo version each time the window opens.", get: getCheckUpdates, set: setCheckUpdates, on: true, enabled: () => app.checkUpdate() },
  ]);
  let error = $state("");
  for (const o of options)
    o.get().then(
      (v) => (o.on = v),
      () => {},
    );

  async function toggle(e: Event, o: (typeof options)[number]) {
    const input = e.currentTarget as HTMLInputElement;
    error = "";
    try {
      await o.set(input.checked);
      o.on = input.checked;
      if (o.on) o.enabled?.();
    } catch (err) {
      input.checked = !input.checked;
      error = String(err);
    }
  }
</script>

<dialog bind:this={dialog} oncancel={(e) => (e.preventDefault(), onclose())}>
  <h2>Options</h2>
  {#each options as o (o.title)}
    <label class="option">
      <span class="text"><strong>{o.title}</strong><span>{o.desc}</span></span>
      <span class="switch"><input type="checkbox" role="switch" checked={o.on} onchange={(e) => toggle(e, o)} /><span class="track"></span></span>
    </label>
  {/each}
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
</style>
