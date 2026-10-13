<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import logo from "../../src-tauri/icons/icon.svg";
  import { openGithub } from "./api";

  let { onclose }: { onclose: () => void } = $props();

  let dialog: HTMLDialogElement;
  let version = $state("");
  let error = $state("");

  onMount(() => dialog.showModal());
  getVersion().then(
    (v) => (version = v),
    () => {},
  );

  const open = (page: "repo" | "releases") => openGithub(page).catch((e) => (error = String(e)));
</script>

<dialog bind:this={dialog} oncancel={(e) => (e.preventDefault(), onclose())}>
  <div class="head">
    <img src={logo} alt="" width="56" height="56" />
    <div>
      <h2>Wallo</h2>
      <span class="selectable">Version {version}</span>
    </div>
  </div>
  <p>Windows wallpaper manager. MIT License.</p>
  <div class="links">
    <button type="button" class="btn" onclick={() => open("repo")}>GitHub</button>
    <button type="button" class="btn" onclick={() => open("releases")}>Release notes</button>
  </div>
  {#if error}<p class="error selectable">{error}</p>{/if}
  <div class="buttons">
    <button type="button" class="btn ghost" onclick={onclose}>Close</button>
  </div>
</dialog>

<style>
  dialog {
    width: 360px;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  h2 {
    margin: 0 0 4px;
  }
  .head span {
    color: var(--muted);
  }
  p {
    color: var(--muted);
    margin: 16px 0;
  }
  .links {
    display: flex;
    gap: 8px;
  }
  .error {
    color: var(--danger);
    margin: 10px 0 0;
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    margin-top: 18px;
  }
</style>
