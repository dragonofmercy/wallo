<script lang="ts">
  import { ask } from "@tauri-apps/plugin-dialog";
  import Icon from "./Icon.svelte";
  import NameDialog from "./NameDialog.svelte";
  import Select from "./Select.svelte";
  import { app } from "./state.svelte";

  type Pending = { title: string; initial: string; except?: string; action: (name: string) => void };
  let pending = $state<Pending | null>(null);

  const options = $derived(app.profiles.toSorted((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: "base" })).map((p) => ({ value: p.name, label: p.name })));

  async function remove() {
    if (await ask(`Delete profile "${app.profile.name}"?`, { title: "Delete profile", kind: "warning" })) app.deleteCurrent();
  }
</script>

{#snippet profileIcon()}
  <span class="pico"><Icon name="image" /></span>
{/snippet}

<div class="bar">
  <span class="h2">Profile</span>
  <div class="grow"><Select bind:value={app.selectedName} {options} icon={profileIcon} /></div>
  <button type="button" class="btn" onclick={() => (pending = { title: "New Profile", initial: "", action: (n) => app.newProfile(n) })}><Icon name="plus" />New</button>
  <button type="button" class="btn" onclick={() => (pending = { title: "Rename Profile", initial: app.profile.name, except: app.profile.name, action: (n) => app.rename(n) })}><Icon name="pencil" />Rename</button>
  <button type="button" class="btn" onclick={() => (pending = { title: "Save Profile As", initial: `${app.profile.name} copy`, action: (n) => app.saveAs(n) })}><Icon name="copy" />Save As</button>
  <button type="button" class="btn danger" onclick={remove}><Icon name="trash" />Delete</button>
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
    align-items: center;
    gap: 10px;
  }
  .h2 {
    margin: 0 6px 0 0;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  .pico {
    display: flex;
    color: var(--cyan);
  }
</style>
