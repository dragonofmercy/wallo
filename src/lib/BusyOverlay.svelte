<div class="overlay" role="alertdialog" aria-modal="true" aria-busy="true" aria-label="Applying wallpaper">
  <div class="panel">
    <svg class="ring" viewBox="0 0 64 64" aria-hidden="true">
      <circle class="track" cx="32" cy="32" r="27" />
      <circle class="arc" cx="32" cy="32" r="27" />
    </svg>
    <div class="title">Applying Wallpaper</div>
    <div class="sub">Rendering each monitor at full resolution...</div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(4, 10, 20, 0.62);
    backdrop-filter: blur(4px);
    animation: fade 0.15s ease-out;
  }
  .panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 28px 40px;
    border-radius: 14px;
    border: 1px solid var(--line2);
    background: linear-gradient(180deg, #10233f, #0b1a30);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.6);
  }
  .ring {
    width: 64px;
    height: 64px;
    margin-bottom: 10px;
    animation: spin 1s linear infinite;
  }
  circle {
    fill: none;
    stroke-width: 5;
  }
  .track {
    stroke: var(--line);
  }
  .arc {
    stroke: var(--cyan);
    stroke-linecap: round;
    stroke-dasharray: 50 170;
    filter: drop-shadow(0 0 4px rgba(34, 211, 238, 0.6));
  }
  .title {
    font:
      700 15px Orbitron,
      sans-serif;
    letter-spacing: 0.04em;
    color: var(--text);
  }
  .sub {
    font-size: 12px;
    color: var(--muted);
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
</style>
