<script lang="ts">
  import { renderPreview } from "./api";
  import { app } from "./state.svelte";
  import type { Monitor, Profile } from "./types";

  let width = $state(0);
  let height = $state(0);
  const PAD = 16;

  let bounds = $derived.by(() => {
    const ms = app.monitors;
    const x0 = Math.min(...ms.map((m) => m.x));
    const y0 = Math.min(...ms.map((m) => m.y));
    const w = Math.max(...ms.map((m) => m.x + m.width)) - x0;
    const h = Math.max(...ms.map((m) => m.y + m.height)) - y0;
    const f = Math.min((width - 2 * PAD) / w, (height - 2 * PAD) / h);
    return { x0, y0, f, left: (width - w * f) / 2, top: (height - h * f) / 2 };
  });

  function box(m: Monitor) {
    const b = bounds;
    return `left:${b.left + (m.x - b.x0) * b.f}px;top:${b.top + (m.y - b.y0) * b.f}px;width:${m.width * b.f}px;height:${m.height * b.f}px`;
  }

  let seq = 0;
  $effect(() => {
    const profile = $state.snapshot(app.profile) as Profile;
    const monitors = $state.snapshot(app.monitors) as Monitor[];
    const target = Math.round(width * devicePixelRatio);
    if (!monitors.length || target <= 0) return;
    const id = ++seq;
    const timer = setTimeout(async () => {
      try {
        const previews = await renderPreview(profile, monitors, target);
        if (id === seq) app.previews = Object.fromEntries(previews.map((p) => [p.id, p]));
      } catch (e) {
        if (id === seq) app.banner = { kind: "error", text: String(e) };
      }
    }, 150);
    return () => clearTimeout(timer);
  });
</script>

<div class="preview" bind:clientWidth={width} bind:clientHeight={height}>
  {#if width > 0}
    {#each app.monitors as m (m.id)}
      <button class="monitor" class:selected={app.profile.mode !== "perMonitor" || m.id === app.selectedMonitorId} style={box(m)} onclick={() => (app.selectedMonitorId = m.id)}>
        {#if app.previews[m.id]}<img src={app.previews[m.id].dataUrl} alt="" />{/if}
        <span>{m.number}</span>
      </button>
    {/each}
  {/if}
</div>

<style>
  .preview {
    position: relative;
    height: 190px;
    flex-shrink: 0;
    background: #060e1a;
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
  }
  .monitor {
    position: absolute;
    padding: 0;
    border: 2px solid #2a3b52;
    border-radius: 2px;
    background: #000;
    overflow: hidden;
  }
  .monitor.selected {
    border-color: var(--cyan);
    box-shadow: 0 0 10px rgba(34, 211, 238, 0.35);
  }
  img {
    width: 100%;
    height: 100%;
    display: block;
  }
  span {
    position: absolute;
    left: 8px;
    top: 50%;
    transform: translateY(-50%);
    font-family: Orbitron, sans-serif;
    font-size: 44px;
    color: #fff;
    text-shadow: 0 0 6px #000;
    pointer-events: none;
  }
</style>
