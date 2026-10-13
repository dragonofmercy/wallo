<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "./Icon.svelte";
  import OptionsDialog from "./OptionsDialog.svelte";
  import { app } from "./state.svelte";

  const close = () => getCurrentWindow().close();

  async function ok() {
    if (await app.apply()) await close();
  }

  let options = $state(false);
</script>

<footer>
  <button type="button" class="btn ghost options" onclick={() => (options = true)}><Icon name="sliders" />Options</button>
  <button type="button" class="btn ghost" disabled={app.busy} onclick={close}><Icon name="close" />Cancel</button>
  <button type="button" class="btn" disabled={app.busy} onclick={() => app.apply()}><Icon name="check" />{app.busy ? "Applying..." : "Apply"}</button>
  <button type="button" class="btn primary ok" disabled={app.busy} onclick={ok}><Icon name="checkCircle" />OK</button>
</footer>

{#if options}<OptionsDialog onclose={() => (options = false)} />{/if}

<style>
  footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
  }
  .options {
    margin-right: auto;
  }
  .ok {
    min-width: 96px;
    justify-content: center;
  }
</style>
