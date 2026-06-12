<script lang="ts">
  import { onMount } from 'svelte';
  import { breakpoint } from '../breakpoint';
  import { api } from '../api';
  import { on } from '../events';
  import { formatMsToHms } from '../format';
  import { courses } from '../stores';
  import type { PendingFinish } from '../types';
  import AssignBibModal from './AssignBibModal.svelte';
  import Button from '../ui/Button.svelte';

  let allPending = $state<PendingFinish[]>([]);
  let selected = $state<PendingFinish | null>(null);
  let dockExpanded = $state(false);
  let dockMode = $derived($breakpoint === 'mobile' || $breakpoint === 'tablet' ? 'drawer' : 'side');

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

  $effect(() => {
    if (typeof document === 'undefined') return;
    const cls = 'has-dock-drawer';
    if (dockMode === 'drawer') {
      document.body.classList.add(cls);
      if (!dockExpanded) {
        document.body.dataset.dock = 'collapsed';
      } else {
        document.body.dataset.dock = 'expanded';
      }
    } else {
      dockExpanded = false;
      document.body.classList.remove(cls);
      delete document.body.dataset.dock;
    }
  });
</script>

<aside class="pending-dock panel-2" data-mode={dockMode} data-expanded={dockMode === 'drawer' ? dockExpanded : true}>
  {#if dockMode === 'drawer'}
    <button class="pending-dock-handle" onclick={() => (dockExpanded = !dockExpanded)} aria-label="Mostra o nascondi pending">
      <span class="hud-strong" style="color: var(--accent-pending)">CODA</span>
      <span class="num" style="color: var(--accent-pending)">{allPending.length}</span>
      <span class="pending-dock-chevron">{dockExpanded ? '▾' : '▴'}</span>
    </button>
  {:else}
    <div class="pending-dock-header">
      <div class="hud-strong" style="color: var(--accent-pending)">
        CODA · {allPending.length}
      </div>
      <Button variant="ghost" size="sm" class="px-2 py-1" onclick={refresh}>↻</Button>
    </div>
  {/if}

  <div class="pending-dock-body">
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

    <div class="pending-dock-footer">
      Click su una riga per assegnare pettorale
    </div>
  </div>
</aside>

{#if selected}
  <AssignBibModal pending={selected} onClose={() => { selected = null; refresh(); }} />
{/if}

<style>
  .pending-dock {
    display: flex;
    flex-direction: column;
    border-color: var(--line-2);
  }

  .pending-dock[data-mode='side'] {
    width: clamp(18rem, 22vw, 24rem);
    min-width: 18rem;
    height: 100%;
    border-left: 1px solid var(--line-2);
  }

  .pending-dock-header,
  .pending-dock-handle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.8rem 1rem;
    border-bottom: 1px solid var(--line-2);
  }

  .pending-dock-handle {
    width: 100%;
    cursor: pointer;
    background: var(--bg-1);
    border: none;
    border-bottom: 1px solid var(--line-2);
  }

  .pending-dock[data-mode='drawer'] {
    position: fixed;
    inset-inline: 0;
    bottom: 0;
    z-index: 60;
    max-height: 72vh;
    box-shadow: var(--shadow-lg);
    border-top: 1px solid var(--line-2);
    border-left: none;
    border-right: none;
    border-bottom: none;
    border-radius: 14px 14px 0 0;
  }

  .pending-dock[data-mode='drawer'][data-expanded='false'] {
    max-height: 3.25rem;
  }

  .pending-dock[data-mode='drawer'][data-expanded='false'] .pending-dock-body {
    display: none;
  }

  .pending-dock-body {
    display: flex;
    flex: 1;
    min-height: 0;
    flex-direction: column;
  }

  .pending-dock-footer {
    padding: 0.7rem 0.85rem;
    border-top: 1px solid var(--line-2);
    color: var(--fg-3);
    font-size: 0.75rem;
  }

  .pending-dock-chevron {
    margin-left: auto;
    color: var(--fg-2);
  }
</style>
