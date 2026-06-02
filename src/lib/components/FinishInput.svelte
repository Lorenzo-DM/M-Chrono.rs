<script lang="ts">
  import { api } from '../api';

  let { onFinish } = $props<{ onFinish: (msg: string) => void }>();
  let bib = $state('');
  let inputEl: HTMLInputElement | null = $state(null);
  let error = $state<string | null>(null);

  async function submit(e: Event) {
    e.preventDefault();
    error = null;
    const n = parseInt(bib);
    if (!Number.isFinite(n)) { error = 'pettorale non valido'; return; }
    try {
      const t = await api.finishByBib(n);
      onFinish(`#${n} ${t.total_time_ms ?? '?'}ms`);
      bib = '';
      inputEl?.focus();
    } catch (e: any) {
      error = e?.message || String(e);
      bib = '';
      inputEl?.focus();
    }
  }

  $effect(() => { inputEl?.focus(); });
</script>

<form onsubmit={submit} class="flex flex-col gap-2">
  <label class="text-xl" for="bib-input">Pettorale</label>
  <input
    id="bib-input"
    bind:this={inputEl}
    bind:value={bib}
    type="number"
    inputmode="numeric"
    class="text-6xl w-full p-4 bg-black border-2 border-white text-white"
    autocomplete="off"
  />
  <button type="submit" class="btn preset-filled text-2xl py-4">Registra arrivo</button>
  {#if error}<p class="text-red-400">{error}</p>{/if}
</form>
