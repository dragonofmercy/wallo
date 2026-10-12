<script lang="ts">
  import { onMount } from "svelte";
  import Adjustments from "./lib/Adjustments.svelte";
  import BackgroundMode from "./lib/BackgroundMode.svelte";
  import BusyOverlay from "./lib/BusyOverlay.svelte";
  import Footer from "./lib/Footer.svelte";
  import MonitorPreview from "./lib/MonitorPreview.svelte";
  import ProfileBar from "./lib/ProfileBar.svelte";
  import WallpaperSource from "./lib/WallpaperSource.svelte";
  import { app } from "./lib/state.svelte";

  let ready = $state(false);

  onMount(async () => {
    try {
      await app.init();
      ready = true;
    } catch (e) {
      app.banner = { kind: "error", text: String(e) };
    }
  });
</script>

<main>
  {#if app.banner}
    <div class="banner {app.banner.kind}">
      <span>{app.banner.text}</span>
      <button type="button" class="btn ghost" onclick={() => (app.banner = null)}>Dismiss</button>
    </div>
  {/if}
  {#if ready}
    <section class="card profile">
      <ProfileBar />
    </section>
    <MonitorPreview />
    <div class="columns">
      <div class="left">
        <section class="card">
          <h2>Background Mode</h2>
          <BackgroundMode />
        </section>
        <section class="card">
          <h2>Wallpaper Source</h2>
          <WallpaperSource />
        </section>
      </div>
      <section class="card">
        <h2>Monitor Settings</h2>
        <Adjustments />
      </section>
    </div>
    <Footer />
  {/if}
  {#if app.busy}<BusyOverlay />{/if}
</main>

<style>
  main {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
  }
  .profile {
    padding: 12px 14px;
  }
  .columns {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }
  .left {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .left .card:last-child {
    flex: 1;
  }
</style>
