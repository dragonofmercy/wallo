<script lang="ts">
  import { onMount } from "svelte";

  let { title, initial, validate, onconfirm, oncancel }: { title: string; initial: string; validate: (name: string) => string | null; onconfirm: (name: string) => void; oncancel: () => void } = $props();

  let dialog: HTMLDialogElement;
  // svelte-ignore state_referenced_locally
  let name = $state(initial);
  let error = $derived(validate(name));

  onMount(() => dialog.showModal());

  function submit(e: SubmitEvent) {
    e.preventDefault();
    if (!error) onconfirm(name.trim());
  }
</script>

<dialog bind:this={dialog} oncancel={(e) => (e.preventDefault(), oncancel())}>
  <form onsubmit={submit}>
    <h2>{title}</h2>
    <!-- svelte-ignore a11y_autofocus -->
    <input bind:value={name} autofocus />
    <p class="error">{error ?? ""}</p>
    <div class="buttons">
      <button type="submit" class="primary" disabled={!!error}>OK</button>
      <button type="button" onclick={oncancel}>Cancel</button>
    </div>
  </form>
</dialog>

<style>
  dialog {
    background: var(--bg1);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 16px;
    width: 320px;
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.55);
  }
  input {
    width: 100%;
    box-sizing: border-box;
  }
  .error {
    color: var(--danger);
    min-height: 1.2em;
    margin: 6px 0;
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
