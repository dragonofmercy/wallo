<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { app } from "./state.svelte";

  const close = () => getCurrentWindow().close();

  async function ok() {
    if (await app.apply()) await close();
  }
</script>

<footer>
  <button class="primary" disabled={app.busy} onclick={ok}>OK</button>
  <button disabled={app.busy} onclick={close}>Cancel</button>
  <button disabled={app.busy} onclick={() => app.apply()}>{app.busy ? "Applying..." : "Apply"}</button>
</footer>

<style>
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  button {
    min-width: 80px;
  }
</style>
