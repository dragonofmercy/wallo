<script lang="ts">
  let { value = $bindable(), label, unit = "", min, max, step = 1, decimals = 0, disabled = false }: { value: number; label: string; unit?: string; min: number; max: number; step?: number; decimals?: number; disabled?: boolean } = $props();

  const clamp = (v: number) => Math.min(max, Math.max(min, v));
  const current = () => (Number.isFinite(value) ? value : 0);

  // Keystroke filter: digits, a leading "-" when negatives are allowed, one decimal separator when decimals are shown.
  function sanitize(input: HTMLInputElement) {
    const caret = input.selectionStart ?? input.value.length;
    let seenDot = false;
    const kept = [...input.value].filter((c, i) => {
      if (c >= "0" && c <= "9") return true;
      if (c === "-") return i === 0 && min < 0;
      if ((c === "." || c === ",") && decimals > 0 && !seenDot) return (seenDot = true);
      return false;
    });
    const removed = input.value.length - kept.length;
    if (removed === 0) return;
    input.value = kept.join("");
    // The rejected character was just typed at the caret, so the caret moves back by what was removed.
    const pos = Math.max(0, caret - removed);
    input.setSelectionRange(pos, pos);
  }

  // Parses typed or pasted text; returns NaN when it is not a plain number (letters, exponents, several signs...).
  function parse(text: string) {
    const t = text.replace(/\s/g, "");
    if (!/^[+-]?[\d.,]*$/.test(t) || !/\d/.test(t)) return NaN;
    // "1,000" / "12,500" (comma + exactly 3 digits, no ".") is a thousands separator: it commits 1000 / 12500.
    // Otherwise a single comma with no "." is the decimal separator ("1,5" -> 1.5); with a "." present commas are thousands separators.
    const n = /^[+-]?\d{1,3}(,\d{3})+$/.test(t) || t.includes(".") ? t.replace(/,/g, "") : t.replace(",", ".");
    return Number(n);
  }

  function commit(input: HTMLInputElement) {
    const v = parse(input.value);
    // Invalid input keeps the previous value.
    value = Number.isFinite(v) ? clamp(Number(v.toFixed(decimals))) : current();
    // Svelte skips the DOM write when the bound value did not change, so always rewrite the text (rejected or clamped-to-same input).
    input.value = current().toFixed(decimals);
  }
</script>

<div class="lbl">{label}</div>
<div class="field num" class:off={disabled}>
  <input
    type="text"
    inputmode="decimal"
    aria-label={label}
    {disabled}
    value={current().toFixed(decimals)}
    oninput={(e) => !/^insertFrom(Paste|Drop)$/.test((e as unknown as InputEvent).inputType) && sanitize(e.currentTarget)}
    onchange={(e) => commit(e.currentTarget)}
  />
  {#if unit}<span class="unit">{unit}</span>{/if}
  <div class="step">
    <button type="button" tabindex="-1" aria-label="Increase {label}" {disabled} onclick={() => (value = clamp(current() + step))}>
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2 6.5l3-3 3 3" /></svg>
    </button>
    <button type="button" tabindex="-1" aria-label="Decrease {label}" {disabled} onclick={() => (value = clamp(current() - step))}>
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2 3.5l3 3 3-3" /></svg>
    </button>
  </div>
</div>

<style>
  .num {
    display: flex;
    align-items: stretch;
    width: 128px;
    overflow: hidden;
  }
  input {
    flex: 1;
    min-width: 0;
    border: 0;
    background: transparent;
    color: var(--text);
    font:
      500 13px Inter,
      sans-serif;
    padding: 0 10px;
    outline: none;
    font-variant-numeric: tabular-nums;
  }
  .unit {
    display: flex;
    align-items: center;
    padding-right: 6px;
    color: var(--dim);
    font-size: 12px;
  }
  .step {
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--line);
  }
  .step button {
    flex: 1;
    width: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 0;
    padding: 0;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
  .step button:hover:not(:disabled) {
    background: rgba(59, 130, 246, 0.18);
    color: #fff;
  }
  .step button + button {
    border-top: 1px solid var(--line);
  }
  svg {
    width: 10px;
    height: 10px;
    stroke: currentColor;
    fill: none;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .off {
    opacity: 0.45;
    pointer-events: none;
  }
</style>
