<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import Icon from "./Icon.svelte";
  import Select from "./Select.svelte";
  import { app } from "./state.svelte";
  import { SIZINGS, type Sizing } from "./types";

  // Icon = an 18x13 "screen" with the image drawn according to the mode: [x, y, w, h] per rect.
  const RECTS: Record<Sizing, number[][]> = {
    center: [[6, 4, 6, 5]],
    stretch: [[1, 1, 16, 11]],
    fitBestNoClip: [[3, 1, 12, 11]],
    fitBestClip: [[0, 0, 18, 13]],
    fitWidth: [[1, 3, 16, 7]],
    fitHeight: [[5, 1, 8, 11]],
    tile: [
      [1, 1, 4, 5],
      [7, 1, 4, 5],
      [13, 1, 4, 5],
      [1, 7, 4, 5],
      [7, 7, 4, 5],
      [13, 7, 4, 5],
    ],
  };

  const file = $derived.by(() => {
    const path = app.source.imagePath ?? "";
    const cut = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
    return { name: path.slice(cut + 1), dir: path.slice(0, Math.max(cut, 0)) };
  });

  async function loadImage() {
    const path = await open({ multiple: false, directory: false, filters: [{ name: "Images", extensions: ["jpg", "jpeg", "png", "bmp", "webp", "gif"] }] });
    if (typeof path === "string") app.source.imagePath = path;
  }
</script>

{#snippet sizingIcon(s: Sizing)}
  <span class="ico"
    >{#each RECTS[s] as [x, y, w, h], i (i)}<i style="left:{x}px;top:{y}px;width:{w}px;height:{h}px"></i>{/each}</span
  >
{/snippet}

<div class="lbl">Sizing</div>
<Select bind:value={app.source.sizing} options={SIZINGS} icon={sizingIcon} />
<label class="chk stretch"><span>Don't stretch smaller images</span><input type="checkbox" bind:checked={app.source.noUpscale} /><span class="tgl"></span></label>
<div class="image">
  <button type="button" class="btn" onclick={loadImage}><Icon name="folder" />Load Image</button>
  <div class="meta selectable">
    {#if app.source.imagePath}
      <div class="name" title={app.source.imagePath}>{file.name}</div>
      <div class="dir">{file.dir}</div>
    {:else}
      <div class="dir">No image</div>
    {/if}
    {#if app.sourceError}<div class="error">{app.sourceError}</div>{/if}
  </div>
</div>

<style>
  .stretch {
    width: 100%;
    justify-content: space-between;
    margin: 14px 0;
  }
  .image {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }
  .meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .name {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .dir {
    font-size: 12px;
    color: var(--dim);
    line-height: 1.4;
    word-break: break-all;
  }
  .error {
    font-size: 12px;
    color: var(--danger);
  }
  .ico {
    position: relative;
    width: 18px;
    height: 13px;
    flex-shrink: 0;
    border-radius: 2px;
    background: var(--line);
    overflow: hidden;
  }
  .ico i {
    position: absolute;
    border-radius: 1px;
    background: var(--cyan);
    opacity: 0.9;
  }
</style>
