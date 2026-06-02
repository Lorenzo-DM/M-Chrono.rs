<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { on } from '../events';
  import { formatMsToHms } from '../format';
  import { courses } from '../stores';
  import type { PendingFinish } from '../types';
  import AssignBibModal from './AssignBibModal.svelte';

  let allPending = $state<PendingFinish[]>([]);
  let selected = $state<PendingFinish | null>(null);

  async function refresh() {
    try {
      const lists = await Promise.all(
        $courses.map(c => api.getPendingFinishes(c.id))
      );
      allPending = lists.flat().sort((a, b) => b.finish_timestamp_ms - a.finish_timestamp_ms);
    } catch {}
  }

  onMount(() => {
    refresh();
    let u1: (() => void) | null = null;
    let u2: (() => void) | null = null;
    on('pending:captured', () => refresh()).then(u => { u1 = u; });
    on('athlete:finished', () => refresh()).then(u => { u2 = u; });
    return () => { u1?.(); u2?.(); };
  });

  $effect(() => {
    // re-fetch when courses list changes
    void $courses;
    refresh();
  });

  function courseName(id: number) {
    return $courses.find(c => c.id === id)?.name ?? `#${id}`;
  }
</script>

<aside class="panel-2 h-full flex flex-col border-l" style="border-color: var(--line-2)">
  <div class="flex items-center justify-between px-4 py-3 border-b" style="border-color: var(--line-2)">
    <div class="hud-strong" style="color: var(--accent-pending)">
      CODA · {allPending.length}
    </div>
    <button class="btn-base btn-ghost text-xs px-2 py-1" onclick={refresh}>↻</button>
  </div>

  {#if allPending.length === 0}
    <div class="flex-1 flex items-center justify-center px-4">
      <div class="text-center">
        <div class="hud mb-2">NESSUN PENDING</div>
        <div class="text-xs" style="color: var(--fg-3)">
          Premi <span class="kbd">TAP</span> o <span class="kbd">␣</span> per catturare un tempo
        </div>
      </div>
    </div>
  {:else}
    <ul class="flex-1 overflow-auto">
      {#each allPending as p (p.id)}
        <li class="slide-in border-b" style="border-color: var(--line-1)">
          <button
            class="w-full text-left px-3 py-3 hover:bg-[var(--bg-3)] flex flex-col gap-1"
            onclick={() => selected = p}
          >
            <div class="flex items-center justify-between">
              <span class="hud" style="color: var(--accent-pending)">#{p.id}</span>
              <span class="hud">{courseName(p.course_id)}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="num text-xl" style="color: var(--fg-0)">
                {formatMsToHms(p.finish_timestamp_ms % 86_400_000)}
              </span>
              <span class="hud" style="color: var(--fg-3)">{p.operator_id}</span>
            </div>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="px-3 py-2 border-t text-xs" style="border-color: var(--line-2); color: var(--fg-3)">
    Click su una riga per assegnare pettorale
  </div>
</aside>

{#if selected}
  <AssignBibModal pending={selected} onClose={() => { selected = null; refresh(); }} />
{/if}
