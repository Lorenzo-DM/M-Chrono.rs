<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { formatMsToHms } from '../format';

  let { onClose }: { onClose: () => void } = $props();
  let groups = $state<any[]>([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      groups = await api.getDuplicateGroups();
    } finally {
      loading = false;
    }
  });
</script>

<div class="fixed inset-0 bg-black/85 overflow-auto p-6" role="dialog">
  <div class="max-w-4xl mx-auto bg-zinc-900 p-6">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-3xl">Duplicati da revisionare</h3>
      <button class="btn preset-tonal" onclick={onClose}>Chiudi</button>
    </div>
    {#if loading}
      <p class="opacity-70">Caricamento…</p>
    {:else if groups.length === 0}
      <p class="opacity-70">Nessun duplicato flaggato.</p>
    {:else}
      {#each groups as g (g.group_id)}
        <div class="border border-yellow-500/40 p-4 mb-4">
          <p class="text-xl">Pettorale #{g.bib_number ?? '?'} · delta {g.delta_ms}ms</p>
          <ul class="mt-2">
            {#each g.timings as t (t.id)}
              <li class="flex justify-between py-1">
                <span>{t.operator_id}</span>
                <span class="font-mono">{formatMsToHms(t.total_time_ms ?? 0)}</span>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    {/if}
  </div>
</div>
