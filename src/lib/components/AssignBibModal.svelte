<script lang="ts">
  import type { PendingFinish } from '../types';
  import { api } from '../api';
  import { formatMsToHms } from '../format';

  let { pending, onClose }: { pending: PendingFinish; onClose: () => void } = $props();
  let bib = $state('');
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function assign() {
    error = null;
    const n = parseInt(bib);
    if (!Number.isFinite(n)) { error = 'pettorale non valido'; return; }
    busy = true;
    try {
      await api.assignPending(pending.id, n);
      onClose();
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally { busy = false; }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter') assign();
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="modal-backdrop" role="dialog">
  <div class="panel-2 w-full max-w-lg">
    <div class="px-5 py-3 flex items-center justify-between border-b" style="border-color: var(--line-2)">
      <div class="hud-strong" style="color: var(--accent-pending)">ASSEGNA PETTORALE</div>
      <button class="btn-base btn-ghost text-xs" onclick={onClose}>ESC</button>
    </div>

    <div class="p-6">
      <div class="hud mb-2">TIMESTAMP CATTURATO</div>
      <div class="chronodial num text-6xl mb-6" data-state="running">
        {formatMsToHms(pending.finish_timestamp_ms % 86_400_000)}
      </div>

      <div class="hud mb-2">PETTORALE</div>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        bind:value={bib}
        type="number"
        inputmode="numeric"
        class="w-full text-5xl tabular num"
        autofocus
        autocomplete="off"
        placeholder="—"
      />
      {#if error}
        <div class="hud mt-3" style="color: var(--accent-finish)">⚠ {error}</div>
      {/if}

      <div class="flex gap-2 mt-6">
        <button class="btn-base btn-primary flex-1 py-3" disabled={busy} onclick={assign}>
          ASSEGNA <span class="kbd ml-2" style="background:transparent; color:inherit; border-color:currentColor; opacity:0.6">↵</span>
        </button>
        <button class="btn-base flex-1 py-3" onclick={onClose}>ANNULLA</button>
      </div>
    </div>
  </div>
</div>
