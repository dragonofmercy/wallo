<script lang="ts">
  import { onMount } from "svelte";
  import BackgroundMode from "./lib/BackgroundMode.svelte";
  import MonitorPreview from "./lib/MonitorPreview.svelte";
  import ProfileBar from "./lib/ProfileBar.svelte";
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
      <button onclick={() => (app.banner = null)}>Dismiss</button>
    </div>
  {/if}
  {#if ready}
    <section class="card">
      <h2>Wallpaper Profile</h2>
      <ProfileBar />
    </section>
    <MonitorPreview />
    <div class="columns">
      <div class="left">
        <section class="card">
          <h2>Background Mode</h2>
          <BackgroundMode />
        </section>
        <section class="card grow">
          <h2>Wallpaper Source</h2>
          <!-- WallpaperSource (Task 11) -->
        </section>
      </div>
      <section class="card">
        <h2>Monitor Settings</h2>
        <!-- Adjustments (Task 11) -->
      </section>
    </div>
    <!-- Footer (Task 11) -->
  {/if}
</main>

<style>
  main {
    box-sizing: border-box;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
  }
  .columns {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .left {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-height: 0;
  }
  .grow {
    flex: 1;
    min-height: 0;
  }
</style>
