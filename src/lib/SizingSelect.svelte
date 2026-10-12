<script lang="ts">
  import { SIZINGS, type Sizing } from "./types";

  let { value = $bindable() }: { value: Sizing } = $props();
  let open = $state(false);
  let root: HTMLDivElement;

  // Icon = a 16x12 "screen" with the image drawn according to the mode.
  const IMAGE: Record<Sizing, string> = {
    center: '<rect x="5" y="4" width="6" height="4"/>',
    stretch: '<rect x="1" y="1" width="14" height="10"/>',
    fitBestNoClip: '<rect x="3" y="1" width="10" height="10"/>',
    fitBestClip: '<rect x="0" y="0" width="16" height="12"/>',
    fitWidth: '<rect x="1" y="3" width="14" height="6"/>',
    fitHeight: '<rect x="4" y="1" width="8" height="10"/>',
    tile: '<rect x="1" y="1" width="4" height="4"/><rect x="6" y="1" width="4" height="4"/><rect x="11" y="1" width="4" height="4"/><rect x="1" y="7" width="4" height="4"/><rect x="6" y="7" width="4" height="4"/><rect x="11" y="7" width="4" height="4"/>',
  };

  const label = $derived(SIZINGS.find((s) => s.value === value)?.label ?? "");

  function pick(v: Sizing) {
    value = v;
    open = false;
  }
</script>

<svelte:window onclick={(e) => open && !root.contains(e.target as Node) && (open = false)} onkeydown={(e) => e.key === "Escape" && (open = false)} />

{#snippet icon(s: Sizing)}
  <svg viewBox="0 0 16 12" width="16" height="12" aria-hidden="true"><rect x="0" y="0" width="16" height="12" class="screen" /><g class="img">{@html IMAGE[s]}</g></svg>
{/snippet}

<div class="select" bind:this={root}>
  <button type="button" class="current" onclick={() => (open = !open)} aria-haspopup="listbox" aria-expanded={open}>
    {@render icon(value)}<span>{label}</span><span class="caret">v</span>
  </button>
  {#if open}
    <ul role="listbox">
      {#each SIZINGS as s (s.value)}
        <li role="option" aria-selected={s.value === value}>
          <button type="button" class:active={s.value === value} onclick={() => pick(s.value)}>{@render icon(s.value)}<span>{s.label}</span></button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .select {
    position: relative;
    flex: 1;
  }
  button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    text-align: left;
  }
  .current span:first-of-type {
    flex: 1;
  }
  .caret {
    color: var(--muted);
  }
  ul {
    position: absolute;
    z-index: 10;
    left: 0;
    right: 0;
    margin: 2px 0 0;
    padding: 4px;
    list-style: none;
    background: var(--bg1);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  }
  ul button {
    border: none;
    background: transparent;
    padding: 4px 8px;
  }
  ul button:hover,
  ul button.active {
    background: rgba(59, 130, 246, 0.25);
  }
  svg :global(.screen) {
    fill: #1e3a5f;
  }
  svg :global(.img rect) {
    fill: var(--cyan);
    opacity: 0.85;
  }
</style>
