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

  const selected = (m: Monitor) => app.profile.mode !== "perMonitor" || m.id === app.selectedMonitorId;

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
      <button class="monitor" class:selected={selected(m)} style={box(m)} onclick={() => (app.selectedMonitorId = m.id)}>
        {#if app.previews[m.id]}<img src={app.previews[m.id].dataUrl} alt="" />{/if}
        <span class="num">{m.number}</span>
        {#if selected(m)}<span class="badge">{m.width} x {m.height}</span>{/if}
      </button>
    {/each}
  {/if}
</div>

<style>
  .preview {
    position: relative;
    flex: 1;
    min-height: 170px;
    background: #050d19;
    border: 1px solid var(--line);
    border-radius: 12px;
    overflow: hidden;
    box-shadow: inset 0 0 40px rgba(0, 0, 0, 0.6);
  }
  .monitor {
    position: absolute;
    padding: 0;
    border: 2px solid #2a3b52;
    border-radius: 4px;
    background: #000;
    overflow: hidden;
    cursor: pointer;
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }
  .monitor:hover {
    border-color: var(--line2);
  }
  .monitor.selected {
    border-color: var(--cyan);
    box-shadow:
      0 0 0 1px rgba(34, 211, 238, 0.35),
      0 0 22px rgba(34, 211, 238, 0.35);
  }
  img {
    width: 100%;
    height: 100%;
    display: block;
  }
  .num {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    font:
      700 40px Orbitron,
      sans-serif;
    color: #fff;
    text-shadow: 0 0 10px rgba(0, 0, 0, 0.8);
    pointer-events: none;
  }
  .badge {
    position: absolute;
    right: 6px;
    top: 6px;
    padding: 2px 7px;
    border-radius: 999px;
    font:
      600 10px Inter,
      sans-serif;
    color: #cffafe;
    background: rgba(8, 51, 68, 0.8);
    border: 1px solid rgba(34, 211, 238, 0.4);
    pointer-events: none;
  }
</style>
