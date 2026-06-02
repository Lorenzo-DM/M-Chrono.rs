<script lang="ts">
  import type { PendingFinish } from '../types';
  import { api } from '../api';
  import { formatMsToHms } from '../format';

  let { pending, onClose }: { pending: PendingFinish; onClose: () => void } = $props();
  let bib = $state('');
  let error = $state<string | null>(null);

  async function assign() {
    const n = parseInt(bib);
    if (!Number.isFinite(n)) { error = 'pettorale non valido'; return; }
    try {
      await api.assignPending(pending.id, n);
      onClose();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }
</script>

<div class="fixed inset-0 bg-black/80 flex items-center justify-center" role="dialog">
  <div class="bg-zinc-900 p-6 max-w-md w-full">
    <h3 class="text-2xl mb-2">Assegna pettorale</h3>
    <p class="opacity-70 mb-4">
      Timestamp: {formatMsToHms(pending.finish_timestamp_ms % 86_400_000)}
    </p>
    <input bind:value={bib} type="number" inputmode="numeric"
           class="text-4xl w-full p-3 bg-black border-2 border-white"
           autofocus
           autocomplete="off" />
    {#if error}<p class="text-red-400 mt-2">{error}</p>{/if}
    <div class="flex gap-2 mt-4">
      <button class="btn preset-filled flex-1" onclick={assign}>Assegna</button>
      <button class="btn preset-tonal flex-1" onclick={onClose}>Annulla</button>
    </div>
  </div>
</div>
