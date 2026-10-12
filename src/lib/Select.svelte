<script lang="ts" generics="T extends string">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let { value = $bindable(), options, icon }: { value: T; options: { value: T; label: string }[]; icon?: Snippet<[T]> } = $props();
  let open = $state(false);
  let root: HTMLDivElement;

  const label = $derived(options.find((o) => o.value === value)?.label ?? "");

  // The window clips anything outside it: open upward when the list does not fit below.
  let up = $state(false);
  function toggle() {
    if (!open) {
      const r = root.getBoundingClientRect();
      const need = options.length * 32 + 16;
      up = window.innerHeight - r.bottom < need && r.top > window.innerHeight - r.bottom;
    }
    open = !open;
  }

  function pick(v: T) {
    value = v;
    open = false;
  }
</script>

<svelte:window onclick={(e) => open && !root.contains(e.target as Node) && (open = false)} onkeydown={(e) => e.key === "Escape" && (open = false)} />

<div class="select" bind:this={root}>
  <button type="button" class="field current" onclick={toggle} aria-haspopup="listbox" aria-expanded={open}>
    {@render icon?.(value)}
    <span class="label">{label}</span>
    <span class="caret" class:open><Icon name="chevron" size={14} /></span>
  </button>
  {#if open}
    <ul role="listbox" class:up>
      {#each options as o (o.value)}
        <li role="option" aria-selected={o.value === value}>
          <button type="button" class:active={o.value === value} onclick={() => pick(o.value)}>{@render icon?.(o.value)}<span>{o.label}</span></button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .select {
    position: relative;
    min-width: 0;
  }
  .current {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 10px 0 12px;
    cursor: pointer;
    text-align: left;
  }
  .label {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .caret {
    display: flex;
    color: var(--muted);
    transition: transform 0.15s;
  }
  .caret.open {
    transform: rotate(180deg);
  }
  ul {
    position: absolute;
    z-index: 20;
    left: 0;
    top: 38px;
    max-width: calc(100vw - 32px);
    min-width: 100%;
    width: max-content;
    margin: 0;
    padding: 5px;
    list-style: none;
    background: #0d1f38;
    border: 1px solid var(--line2);
    border-radius: 10px;
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.55);
  }
  ul.up {
    top: auto;
    bottom: 38px;
  }
  ul button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 32px;
    padding: 0 10px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text);
    font:
      500 13px Inter,
      sans-serif;
    cursor: pointer;
    white-space: nowrap;
    text-align: left;
  }
  ul button:hover {
    background: rgba(59, 130, 246, 0.16);
  }
  ul button.active {
    background: rgba(59, 130, 246, 0.24);
    color: #fff;
  }
</style>
