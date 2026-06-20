<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { formatMsToHms } from '../format';
  import Button from '../ui/Button.svelte';
  import { TriangleAlert, Check } from 'lucide-svelte';

  let { onClose }: { onClose: () => void } = $props();
  let groups = $state<any[]>([]);
  let loading = $state(true);

  async function refresh() {
    loading = true;
    try { groups = await api.getDuplicateGroups(); } finally { loading = false; }
  }

  onMount(() => { refresh(); });

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="modal-backdrop overflow-auto p-4" role="dialog">
  <div class="panel-2 w-full max-w-4xl mx-auto">
    <div class="px-5 py-3 flex items-center justify-between border-b" style="border-color: var(--line-2)">
      <div class="flex items-center gap-3">
        <span style="color: var(--accent-dup)"><TriangleAlert size={16} /></span>
        <div class="hud-strong" style="color: var(--accent-dup)">DUPLICATI DA REVISIONARE</div>
        <div class="hud">·</div>
        <div class="hud">{groups.length}</div>
      </div>
      <div class="flex gap-2">
        <Button variant="ghost" size="sm" class="px-2 py-1" onclick={refresh}>↻ AGGIORNA</Button>
        <Button variant="ghost" size="sm" onclick={onClose}>ESC</Button>
      </div>
    </div>

    <div class="p-4">
      {#if loading}
        <div class="hud" style="color: var(--fg-2)">CARICAMENTO…</div>
      {:else if groups.length === 0}
        <div class="hud" style="color: var(--accent-start)"><Check size={14} /> NESSUN DUPLICATO FLAGGATO</div>
      {:else}
        <div class="flex flex-col gap-3">
          {#each groups as g (g.group_id)}
            <div class="panel" style="border-color: var(--accent-dup)">
              <div class="flex items-center justify-between px-4 py-2 border-b" style="border-color: var(--line-2)">
                <div class="flex items-center gap-3">
                  <span class="hud-strong" style="color: var(--accent-dup)">
                    PETTORALE #{g.bib_number ?? '?'}
                  </span>
                  <span class="hud">Δ</span>
                  <span class="num text-lg" style="color: var(--fg-0)">{g.delta_ms}ms</span>
                </div>
                <span class="hud" style="color: var(--fg-3)">{g.timings.length} letture</span>
              </div>
              <div>
                {#each g.timings as t (t.id)}
                  <div class="ticker-row">
                    <span class="op-chip" style="color: var(--fg-2)">{t.operator_id}</span>
                    <span class="num" style="color: var(--fg-0)">
                      {formatMsToHms(t.total_time_ms ?? 0)}
                    </span>
                    <span class="hud ml-auto" style="color: var(--fg-3)">id #{t.id}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
