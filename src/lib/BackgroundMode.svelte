<script lang="ts">
  import { app } from "./state.svelte";
  import type { Mode } from "./types";

  const MODES: { value: Mode; label: string }[] = [
    { value: "span", label: "Span an image across all monitors" },
    { value: "perMonitor", label: "Use a different image for each monitor" },
    { value: "same", label: "Use the same image on each monitor" },
  ];
</script>

<div class="modes">
  {#each MODES as m (m.value)}
    <label><input type="radio" name="mode" value={m.value} bind:group={app.profile.mode} /> {m.label}</label>
  {/each}
</div>

<style>
  .modes {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
</style>
