<script lang="ts">
  import { app } from "./state.svelte";
  import { defaultAdjustments } from "./types";

  let isTile = $derived(app.source.sizing === "tile");
</script>

<div class="grid">
  <label>Scale (percent):<input type="number" min="1" max="1000" step="1" value={(app.source.scale ?? 100).toFixed(2)} onchange={(e) => (app.source.scale = parseFloat(e.currentTarget.value))} /></label>
  <label>Rotation (degrees):<input type="number" min="-360" max="360" step="1" bind:value={app.source.rotation} disabled={isTile} /></label>
  <label>Move Horizontal (pixels):<input type="number" min="-10000" max="10000" step="1" bind:value={app.source.offsetX} /></label>
  <label>Move Vertical (pixels):<input type="number" min="-10000" max="10000" step="1" bind:value={app.source.offsetY} /></label>
  <label>Blur Amount:<input type="number" min="0" max="100" step="1" bind:value={app.source.blur} /></label>
</div>
<label class="check"><input type="checkbox" bind:checked={app.source.flipH} /> Flip image horizontally (mirrored)</label>
<label class="check"><input type="checkbox" bind:checked={app.source.flipV} /> Flip image vertically (mirrored)</label>
<div class="reset"><button onclick={() => Object.assign(app.source, defaultAdjustments)}>Reset</button></div>

<style>
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px 16px;
    margin-bottom: 14px;
  }
  .grid label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: var(--muted);
  }
  .grid input {
    width: 90px;
    color: var(--text);
  }
  .check {
    display: block;
    margin: 6px 0;
  }
  .reset {
    display: flex;
    justify-content: flex-end;
    margin-top: 12px;
  }
</style>
