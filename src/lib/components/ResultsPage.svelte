<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { on } from '../events';
  import { courses, activeCourseId } from '../stores';
  import { formatMsToHms } from '../format';
  import type { ResultRow } from '../types';
  import Button from '../ui/Button.svelte';
  import ConfirmModal from './ConfirmModal.svelte';

  let selectedCourseId = $state<number | null>(null);
  let rows = $state<ResultRow[]>([]);
  let filter = $state('');
  let categoryFilter = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);
  let confirmState = $state<{ message: string; onConfirm: () => void } | null>(null);

  // Default to the active timing course, else the first course.
  $effect(() => {
    if (selectedCourseId == null) {
      selectedCourseId = $activeCourseId ?? $courses[0]?.id ?? null;
    }
  });

  $effect(() => {
    if (selectedCourseId != null) load(selectedCourseId);
  });

  async function load(courseId: number) {
    busy = true;
    error = null;
    try {
      rows = await api.getResultsByCourse(courseId);
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    let u: (() => void) | null = null;
    on('athlete:finished', () => {
      if (selectedCourseId != null) load(selectedCourseId);
    }).then((fn) => (u = fn));
    return () => u?.();
  });

  let categories = $derived(
    Array.from(
      new Set(rows.map((r) => r.category).filter((c): c is string => !!c)),
    ).sort(),
  );

  let filtered = $derived(
    rows.filter((r) => {
      if (categoryFilter && r.category !== categoryFilter) return false;
      const q = filter.trim().toLowerCase();
      if (!q) return true;
      return (
        String(r.bib_number ?? '').includes(q) ||
        (r.first_name ?? '').toLowerCase().includes(q) ||
        (r.last_name ?? '').toLowerCase().includes(q) ||
        (r.category ?? '').toLowerCase().includes(q)
      );
    }),
  );

  // Finishers keep their overall position; non-finishers render after them.
  let positioned = $derived.by(() => {
    let pos = 0;
    return filtered.map((r) => {
      const isFinish = r.status === 'Finished' && r.total_time_ms != null;
      if (isFinish) pos += 1;
      return { r, pos: isFinish ? pos : null };
    });
  });

  function statusLabel(s: string): string {
    switch (s) {
      case 'Finished': return 'ARRIVATO';
      case 'Running': return 'IN GARA';
      case 'Withdrawn': return 'RITIRATO';
      case 'DNS': return 'NON PARTITO';
      default: return s.toUpperCase();
    }
  }

  function statusColor(s: string): string {
    switch (s) {
      case 'Finished': return 'var(--accent-finish)';
      case 'Running': return 'var(--accent-running)';
      case 'Withdrawn': return 'var(--accent-pending)';
      case 'DNS': return 'var(--fg-3)';
      default: return 'var(--fg-3)';
    }
  }

  function doWithdraw(r: ResultRow) {
    if (r.athlete_id == null) return;
    confirmState = {
      message: `Segnare il pettorale #${r.bib_number} come RITIRATO?`,
      onConfirm: async () => {
        try {
          await api.withdrawByAthleteId(r.athlete_id!);
          if (selectedCourseId != null) await load(selectedCourseId);
        } catch (e: any) { error = e?.message ?? String(e); }
      },
    };
  }

  function doDns(r: ResultRow) {
    if (r.athlete_id == null) return;
    confirmState = {
      message: `Segnare il pettorale #${r.bib_number} come NON PARTITO (DNS)?`,
      onConfirm: async () => {
        try {
          await api.markDnsByAthleteId(r.athlete_id!);
          if (selectedCourseId != null) await load(selectedCourseId);
        } catch (e: any) { error = e?.message ?? String(e); }
      },
    };
  }

  let finishedCount = $derived(rows.filter((r) => r.status === 'Finished').length);
</script>

<div class="p-6 max-w-5xl mx-auto">
  <div class="flex items-center justify-between mb-5">
    <div>
      <div class="hud" style="color: var(--fg-3)">CLASSIFICA</div>
      <h2 class="hud-strong text-2xl mt-1" style="color: var(--fg-0)">RESULTS</h2>
    </div>
    <div class="hud" style="color: var(--fg-3)">
      <span style="color: var(--accent-finish)">{finishedCount}</span> arrivati / {rows.length}
    </div>
  </div>

  <div class="flex flex-wrap items-end gap-3 mb-4">
    <label class="flex flex-col gap-1">
      <span class="hud">PERCORSO</span>
      <select bind:value={selectedCourseId}>
        {#each $courses as c (c.id)}
          <option value={c.id}>{c.name}</option>
        {/each}
      </select>
    </label>
    {#if categories.length > 0}
      <label class="flex flex-col gap-1">
        <span class="hud">CATEGORIA</span>
        <select bind:value={categoryFilter}>
          <option value="">Tutte</option>
          {#each categories as cat (cat)}
            <option value={cat}>{cat}</option>
          {/each}
        </select>
      </label>
    {/if}
    <label class="flex flex-col gap-1 flex-1 max-w-xs">
      <span class="hud">CERCA</span>
      <input bind:value={filter} placeholder="pettorale, nome, categoria…" autocomplete="off" />
    </label>
  </div>

  {#if error}
    <div class="hud mb-3" style="color: var(--accent-finish)">⚠ {error}</div>
  {/if}

  {#if busy && rows.length === 0}
    <div class="hud" style="color: var(--fg-3)">CARICAMENTO…</div>
  {:else if filtered.length === 0}
    <div class="panel p-8 text-center">
      <div class="hud" style="color: var(--fg-3)">NESSUN RISULTATO</div>
    </div>
  {:else}
    <div class="panel" style="overflow: hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="hud text-left" style="background: var(--bg-2)">
            <th class="px-3 py-2">POS</th>
            <th class="px-3 py-2">PETT.</th>
            <th class="px-3 py-2">ATLETA</th>
            <th class="px-3 py-2">CAT.</th>
            <th class="px-3 py-2">TEMPO</th>
            <th class="px-3 py-2">STATO</th>
            <th class="px-3 py-2"></th>
          </tr>
        </thead>
        <tbody>
          {#each positioned as { r, pos } (r.timing_id)}
            <tr class="border-t" style="border-color: var(--line-1)">
              <td class="px-3 py-1.5 num" style="color: var(--fg-2)">{pos ?? '—'}</td>
              <td class="px-3 py-1.5 num" style="color: var(--accent-running); font-weight: 700">
                {r.bib_number ?? '—'}
              </td>
              <td class="px-3 py-1.5" style="color: var(--fg-1)">
                {#if r.first_name || r.last_name}
                  {r.first_name ?? ''} {r.last_name ?? ''}
                {:else}
                  <span class="hud" style="color: var(--accent-pending)">SENZA NOME</span>
                {/if}
                {#if r.duplicate_flagged}
                  <span class="hud ml-2" style="color: var(--accent-finish)">DUP</span>
                {/if}
              </td>
              <td class="px-3 py-1.5" style="color: var(--fg-2)">{r.category ?? ''}</td>
              <td class="px-3 py-1.5 num" style="color: var(--fg-0); font-weight: 700">
                {r.total_time_ms != null ? formatMsToHms(r.total_time_ms) : '—'}
              </td>
              <td class="px-3 py-1.5">
                <span class="hud" style="color: {statusColor(r.status)}">{statusLabel(r.status)}</span>
              </td>
              <td class="px-3 py-1.5 text-right whitespace-nowrap">
                {#if r.status === 'Running'}
                  <Button variant="ghost" size="sm" onclick={() => doWithdraw(r)} title="Ritira">RIT</Button>
                  <Button variant="ghost" size="sm" onclick={() => doDns(r)} title="Non partito">DNS</Button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

{#if confirmState}
  <ConfirmModal
    message={confirmState.message}
    danger={false}
    onCancel={() => (confirmState = null)}
    onConfirm={() => {
      const c = confirmState;
      confirmState = null;
      c?.onConfirm();
    }}
  />
{/if}
