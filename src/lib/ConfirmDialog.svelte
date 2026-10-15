<script lang="ts">
  import { onMount } from "svelte";

  let { title, message, confirmLabel, onconfirm, oncancel }: { title: string; message: string; confirmLabel: string; onconfirm: () => void; oncancel: () => void } = $props();

  let dialog: HTMLDialogElement;

  onMount(() => dialog.showModal());
</script>

<dialog bind:this={dialog} oncancel={(e) => (e.preventDefault(), oncancel())}>
  <h2>{title}</h2>
  <p>{message}</p>
  <div class="buttons">
    <!-- Focus on Cancel: a reflex Enter must not confirm. -->
    <!-- svelte-ignore a11y_autofocus -->
    <button type="button" class="btn ghost" onclick={oncancel} autofocus>Cancel</button>
    <button type="button" class="btn danger" onclick={onconfirm}>{confirmLabel}</button>
  </div>
</dialog>

<style>
  dialog {
    width: 360px;
  }
  p {
    margin: 0;
  }
</style>
