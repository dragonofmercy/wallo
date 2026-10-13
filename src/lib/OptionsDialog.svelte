<script lang="ts">
  import { onMount } from "svelte";
  import { getAutostart, setAutostart } from "./api";
  import { app } from "./state.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let dialog: HTMLDialogElement;

  onMount(() => dialog.showModal());

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

<dialog bind:this={dialog} oncancel={(e) => (e.preventDefault(), onclose())}>
  <h2>Options</h2>
  <label class="chk"><input type="checkbox" checked={autostart} onchange={toggleAutostart} /><span class="box"><svg viewBox="0 0 12 12"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg></span>Start with Windows</label>
  <div class="buttons">
    <button type="button" class="btn primary" onclick={onclose}>Close</button>
  </div>
</dialog>

<style>
  dialog {
    width: 320px;
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    margin-top: 18px;
  }
</style>
