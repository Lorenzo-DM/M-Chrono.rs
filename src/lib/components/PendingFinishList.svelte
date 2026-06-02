<script lang="ts">
  import type { PendingFinish } from '../types';
  import { api } from '../api';
  import { onMount } from 'svelte';
  import { on } from '../events';
  import { formatMsToHms } from '../format';
  import AssignBibModal from './AssignBibModal.svelte';

  let { courseId }: { courseId: number } = $props();
  let items = $state<PendingFinish[]>([]);
  let selected = $state<PendingFinish | null>(null);

  async function refresh() {
    items = await api.getPendingFinishes(courseId);
  }

  onMount(() => {
    refresh();
    let unsub1: (() => void) | null = null;
    let unsub2: (() => void) | null = null;
    on('pending:captured', () => refresh()).then(u => { unsub1 = u; });
    on('athlete:finished', () => refresh()).then(u => { unsub2 = u; });
    return () => {
      unsub1?.();
      unsub2?.();
    };
  });
</script>

<div class="border border-white/20 p-4 mt-4">
  <h3 class="text-xl mb-2">Pending ({items.length})</h3>
  {#if items.length === 0}
    <p class="opacity-70">Nessuno.</p>
  {:else}
    <ul class="flex flex-col gap-1">
      {#each items as p (p.id)}
        <li class="flex items-center justify-between">
          <span>#{p.id} · op {p.operator_id} · {formatMsToHms(p.finish_timestamp_ms % 86_400_000)}</span>
          <button class="btn preset-tonal" onclick={() => selected = p}>Assegna</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if selected}
  <AssignBibModal pending={selected} onClose={() => { selected = null; refresh(); }} />
{/if}
