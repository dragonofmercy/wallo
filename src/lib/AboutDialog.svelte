<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import logo from "../../src-tauri/icons/icon.svg";
  import { openGithub } from "./api";
  import Icon from "./Icon.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let dialog: HTMLDialogElement;
  let version = $state("");
  let error = $state("");

  onMount(() => dialog.showModal());
  getVersion().then(
    (v) => (version = v),
    () => {},
  );

  // The href is only shown on hover: the backend opens the page in the default browser.
  function open(e: MouseEvent, page: "repo" | "releases") {
    e.preventDefault();
    openGithub(page).catch((err) => (error = String(err)));
  }
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
    <a href="https://github.com/dragonofmercy/wallo" onclick={(e) => open(e, "repo")}>GitHub<Icon name="external" size={13} /></a>
    <a href="https://github.com/dragonofmercy/wallo/releases" onclick={(e) => open(e, "releases")}>Release notes<Icon name="external" size={13} /></a>
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
    gap: 20px;
  }
  a {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--accent-hi);
    text-decoration: none;
    font-weight: 500;
  }
  a:hover {
    text-decoration: underline;
  }
  a:focus-visible {
    outline: 2px solid var(--cyan);
    outline-offset: 2px;
    border-radius: 3px;
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
