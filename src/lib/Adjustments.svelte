<script lang="ts">
  import Icon from "./Icon.svelte";
  import NumberField from "./NumberField.svelte";
  import { app } from "./state.svelte";
  import { defaultAdjustments } from "./types";

  const ROWS = ["top", "", "bottom"];
  const COLS = ["left", "", "right"];
  const anchors = [-1, 0, 1].flatMap((ay) => [-1, 0, 1].map((ax) => ({ ax, ay, title: `Align ${[ROWS[ay + 1], COLS[ax + 1]].filter(Boolean).join(" ") || "center"}` })));
</script>

<div class="grid">
  <div><NumberField label="Scale" unit="%" min={1} max={1000} decimals={2} bind:value={app.source.scale} /></div>
  <div><NumberField label="Rotation" unit="deg" min={-360} max={360} disabled={app.source.sizing === "tile"} bind:value={app.source.rotation} /></div>
  <div><NumberField label="Move horizontal" unit="px" min={-10000} max={10000} bind:value={app.source.offsetX} /></div>
  <div><NumberField label="Move vertical" unit="px" min={-10000} max={10000} bind:value={app.source.offsetY} /></div>
  <div><NumberField label="Blur amount" min={0} max={100} bind:value={app.source.blur} /></div>
  <div>
    <div class="lbl">Align</div>
    <div class="anchors">
      {#each anchors as a (a.title)}
        <button type="button" title={a.title} aria-label={a.title} disabled={!app.source.imagePath} onclick={() => app.align(a.ax, a.ay)}><span></span></button>
      {/each}
    </div>
  </div>
</div>
<div class="flips">
  <label class="chk"><input type="checkbox" bind:checked={app.source.flipH} /><span class="box"><svg viewBox="0 0 12 12"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg></span>Flip image horizontally (mirrored)</label>
  <label class="chk"><input type="checkbox" bind:checked={app.source.flipV} /><span class="box"><svg viewBox="0 0 12 12"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg></span>Flip image vertically (mirrored)</label>
</div>
<!-- Pinned to the card's bottom-right corner (the card is position: relative). -->
<button type="button" class="btn ghost reset" onclick={() => Object.assign(app.source, defaultAdjustments)}><Icon name="reset" />Reset</button>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px 18px;
  }
  .anchors {
    display: grid;
    grid-template-columns: repeat(3, 24px);
    gap: 3px;
  }
  .anchors button {
    display: grid;
    place-items: center;
    height: 16px;
    padding: 0;
    border: 1px solid var(--line2);
    border-radius: 4px;
    background: var(--field);
    cursor: pointer;
  }
  .anchors span {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--muted);
  }
  .anchors button:hover:not(:disabled) {
    border-color: var(--accent);
  }
  .anchors button:hover:not(:disabled) span {
    background: var(--cyan);
  }
  .anchors button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .flips {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 18px;
  }
  .reset {
    position: absolute;
    right: 16px;
    bottom: 14px;
  }
</style>
