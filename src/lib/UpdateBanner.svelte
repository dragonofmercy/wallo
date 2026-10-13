<script lang="ts">
  import type { Update } from "@tauri-apps/plugin-updater";
  import Icon from "./Icon.svelte";
  import { app } from "./state.svelte";

  let { update }: { update: Update } = $props();
  let progress = $state<number | null>(null);

  /** Downloads, checks the signature, then runs the installer: Windows closes Wallo and the installer restarts it. */
  async function install() {
    let total = 0;
    let done = 0;
    progress = 0;
    try {
      await update.downloadAndInstall((e) => {
        if (e.event === "Started") total = e.data.contentLength ?? 0;
        if (e.event === "Progress") done += e.data.chunkLength;
        if (total) progress = Math.round((done / total) * 100);
      });
    } catch (e) {
      progress = null;
      app.banner = { kind: "error", text: `Update failed: ${e}` };
    }
  }
</script>

<div class="update">
  <span class="ico"><Icon name="download" size={16} /></span>
  <div class="text">
    <strong>Wallo {update.version} is available</strong>
    <small>You have {update.currentVersion}. Wallo will restart to finish the update.</small>
  </div>
  <button type="button" class="btn ghost" disabled={progress !== null} onclick={() => (app.update = null)}>Later</button>
  <button type="button" class="btn primary" disabled={progress !== null} onclick={install}><Icon name="download" />{progress === null ? "Install update" : `Downloading ${progress}%`}</button>
</div>

<style>
  .update {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 8px 8px 12px;
    border-radius: 10px;
    border: 1px solid rgba(34, 211, 238, 0.45);
    background: linear-gradient(90deg, rgba(34, 211, 238, 0.14), rgba(59, 130, 246, 0.06));
    box-shadow: 0 0 18px rgba(34, 211, 238, 0.12);
  }
  .ico {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    color: var(--cyan);
    background: rgba(34, 211, 238, 0.14);
    border: 1px solid rgba(34, 211, 238, 0.35);
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  strong {
    font-size: 13px;
    color: var(--text);
  }
  small {
    font-size: 12px;
    color: var(--muted);
  }
</style>
