<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "./Icon.svelte";
  import AboutDialog from "./AboutDialog.svelte";
  import OptionsDialog from "./OptionsDialog.svelte";
  import { app } from "./state.svelte";

  let about = $state(false);

  const close = () => getCurrentWindow().close();

  async function ok() {
    if (await app.apply()) await close();
  }
</script>

<footer>
  <button type="button" class="btn ghost" onclick={() => (app.optionsOpen = true)}><Icon name="sliders" />Options</button>
  <button type="button" class="btn ghost about" onclick={() => (about = true)}><Icon name="info" />About</button>
  <button type="button" class="btn ghost" disabled={app.busy} onclick={close}><Icon name="close" />Cancel</button>
  <button type="button" class="btn" disabled={app.busy} onclick={() => app.apply()}><Icon name="check" />{app.busy ? "Applying..." : "Apply"}</button>
  <button type="button" class="btn primary ok" disabled={app.busy} onclick={ok}><Icon name="checkCircle" />OK</button>
</footer>

{#if app.optionsOpen}<OptionsDialog onclose={() => (app.optionsOpen = false)} />{/if}
{#if about}<AboutDialog onclose={() => (about = false)} />{/if}

<style>
  footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
  }
  .about {
    margin-right: auto;
  }
  .ok {
    min-width: 96px;
    justify-content: center;
  }
</style>
