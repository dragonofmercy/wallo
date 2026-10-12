<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import SizingSelect from "./SizingSelect.svelte";
  import { app } from "./state.svelte";

  async function loadImage() {
    const path = await open({ multiple: false, directory: false, filters: [{ name: "Images", extensions: ["jpg", "jpeg", "png", "bmp", "webp", "gif"] }] });
    if (typeof path === "string") app.source.imagePath = path;
  }
</script>

<div class="row">
  <span class="label">Sizing:</span>
  <SizingSelect bind:value={app.source.sizing} />
</div>
<label class="indent"><input type="checkbox" bind:checked={app.source.noUpscale} /> Don't stretch smaller images</label>
<div class="image">
  <button onclick={loadImage}>Load Image</button>
  <div class="path">
    {#if app.source.imagePath}{app.source.imagePath}{:else}<span class="muted">No image</span>{/if}
    {#if app.sourceError}<p class="error">{app.sourceError}</p>{/if}
  </div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .label {
    width: 52px;
    text-align: right;
    color: var(--muted);
  }
  .indent {
    display: block;
    margin: 10px 0 10px 60px;
  }
  .image {
    display: flex;
    gap: 8px;
    align-items: flex-start;
  }
  .path {
    flex: 1;
    min-height: 70px;
    padding: 6px 8px;
    background: var(--field);
    border: 1px solid var(--border);
    border-radius: 6px;
    word-break: break-all;
  }
  .muted {
    color: var(--muted);
  }
  .error {
    color: var(--danger);
    margin: 6px 0 0;
  }
</style>
