<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "./Icon.svelte";
  import { app } from "./state.svelte";

  const close = () => getCurrentWindow().close();

  async function ok() {
    if (await app.apply()) await close();
  }
</script>

<footer>
  <button type="button" class="btn ghost" disabled={app.busy} onclick={close}>Cancel</button>
  <button type="button" class="btn" disabled={app.busy} onclick={() => app.apply()}><Icon name="check" />{app.busy ? "Applying..." : "Apply"}</button>
  <button type="button" class="btn primary ok" disabled={app.busy} onclick={ok}>OK</button>
</footer>

<style>
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
  .ok {
    min-width: 96px;
    justify-content: center;
  }
</style>
