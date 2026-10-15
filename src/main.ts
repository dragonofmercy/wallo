import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
import "@fontsource/inter/600.css";
import "@fontsource/orbitron/600.css";
import "@fontsource/orbitron/700.css";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

// No browser context menu, except in inputs and selectable text (copy/paste). Dev tools stay reachable with Ctrl+Shift+I in dev.
document.addEventListener("contextmenu", (e) => {
  if (!(e.target instanceof Element && e.target.closest("input, .selectable"))) e.preventDefault();
});

const app = mount(App, { target: document.getElementById("app")! });

export default app;
