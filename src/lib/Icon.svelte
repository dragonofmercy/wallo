<script lang="ts">
  const PATHS = {
    plus: "M8 3v10M3 8h10",
    pencil: "M10.5 2.5l3 3-8 8H2.5v-3z",
    copy: "M5 6.5A1.5 1.5 0 0 1 6.5 5h5.5A1.5 1.5 0 0 1 13.5 6.5V12a1.5 1.5 0 0 1-1.5 1.5H6.5A1.5 1.5 0 0 1 5 12zM11 5V3.5a1 1 0 0 0-1-1H3.5a1 1 0 0 0-1 1V10a1 1 0 0 0 1 1H5",
    trash: "M2.5 4.5h11M6.5 4.5V3h3v1.5M4 4.5l.7 9h6.6l.7-9",
    folder: "M2 5.5V12a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V6.5a1 1 0 0 0-1-1H8L6.5 3.5H3a1 1 0 0 0-1 1z",
    reset: "M3 8a5 5 0 1 0 1.5-3.5M3 2.5v3h3",
    check: "M3 8.5l3.2 3L13 4.5",
    image: "M2.5 3h11a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1zM1.5 10.5l4-3.5 3 2.5 2.5-2 3.5 3",
    close: "M4 4l8 8M12 4l-8 8",
    checkCircle: "M8 14.5a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13zM5.2 8.2l1.9 1.9 3.7-3.9",
    download: "M8 2.5v8M4.5 7.5L8 11l3.5-3.5M3 13.5h10",
    chevron: "M4 6l4 4 4-4",
    info: "M8 14.5a6.5 6.5 0 1 0 0-13 6.5 6.5 0 0 0 0 13zM8 7.5v4M8 5h.01",
    sliders: "M2 4.5h12M2 11.5h12M5.5 3v3M10.5 10v3",
  };

  let { name, size = 15 }: { name: keyof typeof PATHS; size?: number } = $props();
</script>

<svg viewBox="0 0 16 16" width={size} height={size} aria-hidden="true"><path d={PATHS[name]} /></svg>

<style>
  svg {
    flex-shrink: 0;
    stroke: currentColor;
    fill: none;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
