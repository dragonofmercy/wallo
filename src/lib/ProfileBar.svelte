<script lang="ts">
  import { ask } from "@tauri-apps/plugin-dialog";
  import NameDialog from "./NameDialog.svelte";
  import { app } from "./state.svelte";

  type Pending = { title: string; initial: string; except?: string; action: (name: string) => void };
  let pending = $state<Pending | null>(null);

  async function remove() {
    if (await ask(`Delete profile "${app.profile.name}"?`, { title: "Delete profile", kind: "warning" })) app.deleteCurrent();
  }
</script>

<div class="bar">
  <select bind:value={app.selectedName}>
    {#each app.profiles as p (p.name)}
      <option value={p.name}>{p.name}</option>
    {/each}
  </select>
  <button onclick={() => (pending = { title: "New Profile", initial: "", action: (n) => app.newProfile(n) })}>New Profile</button>
  <button onclick={() => (pending = { title: "Rename Profile", initial: app.profile.name, except: app.profile.name, action: (n) => app.rename(n) })}>Rename</button>
  <button onclick={() => (pending = { title: "Save Profile As", initial: `${app.profile.name} copy`, action: (n) => app.saveAs(n) })}>Save As</button>
  <button onclick={remove}>Delete</button>
</div>

{#if pending}
  {@const p = pending}
  <NameDialog
    title={p.title}
    initial={p.initial}
    validate={(n) => app.nameError(n, p.except)}
    onconfirm={(n) => {
      p.action(n);
      pending = null;
    }}
    oncancel={() => (pending = null)}
  />
{/if}

<style>
  .bar {
    display: flex;
    gap: 8px;
  }
  select {
    flex: 1;
  }
</style>
