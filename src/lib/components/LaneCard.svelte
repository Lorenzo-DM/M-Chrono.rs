<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { on } from '../events';
  import { formatMsToHms } from '../format';
  import type { Course, PendingFinish, AthleteRow } from '../types';
  import ConfirmRaceModal from './ConfirmRaceModal.svelte';

  let {
    course,
    size = 'full',
    active = false,
    onFocus,
  } = $props<{
    course: Course;
    size?: 'full' | 'compact';
    active?: boolean;
    onFocus?: () => void;
  }>();

  let elapsed = $state(0);
  let started = $state(false);
  let ended = $state(false);
  let pending = $state<PendingFinish[]>([]);
  let finishers = $state<AthleteRow[]>([]);
  let flashing = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let bibInputs = $state<Record<number, string>>({});
  let showEndModal = $state(false);
  let showRestartModal = $state(false);
  let editingTimingId = $state<number | null>(null);
  let editBibInput = $state('');

  $effect(() => {
    let alive = true;
    const tick = async () => {
      if (!alive) return;
      try {
        const snap = await api.pollDisplay();
        const c = snap.courses.find((x) => x.id === course.id);
        if (c) {
          elapsed = c.elapsed_ms ?? 0;
          started = c.started;
          ended = c.ended;
        }
      } catch {
        // transient
      }
    };
    tick();
    const id = window.setInterval(tick, 100);
    return () => {
      alive = false;
      clearInterval(id);
    };
  });

  async function refresh() {
    try {
      const [p, a] = await Promise.all([
        api.getPendingFinishes(course.id),
        api.getAthletesByCourse(course.id),
      ]);
      pending = [...p].sort((x, y) => y.finish_timestamp_ms - x.finish_timestamp_ms);
      finishers = a
        .filter((r) => r.status === 'Finished')
        .sort((x, y) => (y.finish_ms ?? 0) - (x.finish_ms ?? 0));
    } catch {}
  }

  onMount(() => {
    refresh();
    let u1: (() => void) | null = null;
    let u2: (() => void) | null = null;
    on('pending:captured', () => refresh()).then((u) => (u1 = u));
    on('athlete:finished', () => refresh()).then((u) => (u2 = u));
    return () => {
      u1?.();
      u2?.();
    };
  });

  async function doStart() {
    busy = true;
    error = null;
    try {
      await api.startCourse(course.id);
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  async function doRecord() {
    error = null;
    try {
      const p = await api.capturePending(course.id);
      pending = [p, ...pending];
      flashing = true;
      setTimeout(() => (flashing = false), 600);
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function doTie() {
    if (pending.length === 0) return;
    error = null;
    try {
      const p = await api.capturePendingTie(course.id);
      pending = [p, ...pending];
      flashing = true;
      setTimeout(() => (flashing = false), 600);
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function doAssign(pid: number) {
    error = null;
    const raw = (bibInputs[pid] ?? '').trim();
    const n = parseInt(raw);
    if (!Number.isFinite(n) || n <= 0) {
      error = 'pettorale non valido';
      return;
    }
    try {
      await api.assignPending(pid, n);
      const next = { ...bibInputs };
      delete next[pid];
      bibInputs = next;
      await refresh();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function doDeletePending(pid: number, tsMs: number) {
    const label = formatMsToHms(tsMs % 86_400_000);
    if (!window.confirm(`Eliminare il tempo ${label}? Operazione irreversibile.`)) return;
    error = null;
    try {
      await api.deletePendingFinish(pid);
      const next = { ...bibInputs };
      delete next[pid];
      bibInputs = next;
      await refresh();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function doUndoFinish(timingId: number, bib: number) {
    if (!window.confirm(`Annullare l'arrivo del pettorale #${bib}? L'atleta torna in gara.`)) return;
    error = null;
    try {
      await api.undoFinish(timingId);
      await refresh();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  function startEditBib(timingId: number, currentBib: number) {
    editingTimingId = timingId;
    editBibInput = String(currentBib);
    error = null;
  }

  function cancelEditBib() {
    editingTimingId = null;
    editBibInput = '';
  }

  async function commitEditBib(timingId: number) {
    const n = parseInt(editBibInput.trim());
    if (!Number.isFinite(n) || n <= 0) {
      error = 'pettorale non valido';
      return;
    }
    error = null;
    try {
      await api.reassignBib(timingId, n);
      editingTimingId = null;
      editBibInput = '';
      await refresh();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (!active) return;
    const tgt = e.target as HTMLElement | null;
    if (tgt?.tagName === 'INPUT') return;
    if (!started || ended) return;
    if (e.code === 'Space') {
      e.preventDefault();
      doRecord();
    }
  }

  type Row =
    | { kind: 'pending'; p: PendingFinish; t: number; pos: number }
    | { kind: 'finish'; f: AthleteRow; t: number; pos: number };

  let rows = $derived.by<Row[]>(() => {
    const merged: Array<
      | { kind: 'pending'; p: PendingFinish; t: number }
      | { kind: 'finish'; f: AthleteRow; t: number }
    > = [
      ...pending.map((p) => ({ kind: 'pending' as const, p, t: p.finish_timestamp_ms })),
      ...finishers.map((f) => ({ kind: 'finish' as const, f, t: f.finish_ms ?? 0 })),
    ];
    const asc = [...merged].sort((a, b) => a.t - b.t);
    const positioned: Row[] = asc.map((r, i) => ({ ...r, pos: i + 1 }));
    return positioned.sort((a, b) => b.t - a.t);
  });

  let pendingCount = $derived(pending.length);
</script>

<svelte:window onkeydown={handleKey} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section
  class="lane-card panel relative flex flex-col {flashing ? 'flash-capture' : ''}"
  style="border-color: {active ? 'var(--accent-running)' : 'var(--line-2)'}; transition: border-color 120ms"
  aria-label="Lane {course.name}"
  onclick={() => onFocus?.()}
>
  <!-- Course header -->
  <header
    class="px-4 py-3 border-b flex items-center justify-between gap-3 shrink-0"
    style="background: var(--bg-2); border-color: var(--line-2)"
  >
    <div class="flex flex-col min-w-0">
      <div class="hud-strong text-base truncate" style="color: var(--fg-0)">
        {course.name}
      </div>
      {#if course.distance_m}
        <div class="hud mt-0.5" style="color: var(--fg-2)">
          {(course.distance_m / 1000).toFixed(1)} KM
        </div>
      {/if}
    </div>

    <div class="flex items-center gap-2 shrink-0">
      <span
        class={ended ? 'dot-idle' : started ? 'dot-running' : 'dot-idle'}
        style={ended ? 'background: var(--accent-finish)' : ''}
      ></span>
      <span
        class="lane-status"
        style="color: {ended
          ? 'var(--accent-finish)'
          : started
            ? 'var(--accent-running)'
            : 'var(--fg-3)'}"
      >
        {ended ? 'TERMINATA' : started ? 'ACTIVE' : 'STANDBY'}
      </span>

      {#if started && !ended}
        <button
          class="btn-header-end"
          title="Termina gara"
          aria-label="Termina gara"
          onclick={(e) => {
            e.stopPropagation();
            showEndModal = true;
          }}
        >
          ■ TERMINA
        </button>
      {/if}

      {#if ended}
        <button
          class="btn-header-restart"
          title="Riavvia gara (azzera timer)"
          aria-label="Riavvia gara"
          onclick={(e) => {
            e.stopPropagation();
            showRestartModal = true;
          }}
        >
          ↻ RIAVVIA
        </button>
      {/if}
    </div>
  </header>

  <!-- Timer -->
  <div
    class="bg-[var(--bg-0)] border-b flex items-center justify-center shrink-0"
    style="border-color: var(--line-2); padding: {size === 'full' ? '2rem 1rem' : '1.25rem 1rem'}"
  >
    <div
      class="chronodial num"
      data-state={started ? 'running' : 'idle'}
      style="font-size: {size === 'full'
        ? 'clamp(3.5rem, 9vw, 7rem)'
        : 'clamp(2.5rem, 5vw, 4rem)'}; font-weight: 700; letter-spacing: -0.025em"
    >
      {formatMsToHms(elapsed)}
    </div>
  </div>

  <!-- Action -->
  <div class="p-3 shrink-0 border-b" style="border-color: var(--line-2)">
    {#if !started}
      <button
        class="btn-base btn-accent-start w-full text-base font-semibold"
        style="padding: 1rem 1rem"
        disabled={busy}
        onclick={doStart}
      >
        ▶ START PERCORSO
      </button>
    {:else if ended}
      <button
        class="btn-base w-full text-base"
        style="padding: 1rem 1rem; color: var(--fg-3); cursor: default"
        disabled
      >
        ■ GARA TERMINATA
      </button>
    {:else}
      <div class="flex gap-2">
        <button
          class="btn-base btn-accent-tap flex-1 text-base font-semibold"
          style="padding: 1rem 1rem; letter-spacing: 0.06em"
          onclick={doRecord}
        >
          RECORD TIME
          {#if active}
            <span
              class="kbd ml-2"
              style="background: transparent; color: inherit; border-color: currentColor; opacity: 0.75"
              >␣</span
            >
          {/if}
        </button>
        <button
          class="btn-tie"
          title={pending.length > 0
            ? `Aggiunge un atleta con lo stesso tempo dell'ultimo arrivo (${formatMsToHms(pending[0].finish_timestamp_ms % 86_400_000)})`
            : 'Disponibile dopo il primo RECORD'}
          aria-label="Cattura tempo identico"
          disabled={pending.length === 0}
          onclick={doTie}
        >
          <span class="num text-base font-bold">+1</span>
          <span class="tie-label">STESSO TEMPO</span>
        </button>
      </div>
    {/if}
  </div>

  <!-- Mixed history list -->
  <div class="flex-1 min-h-0 flex flex-col">
    <div
      class="flex items-center justify-between px-3 py-2 border-b shrink-0"
      style="border-color: var(--line-1); background: var(--bg-1)"
    >
      <div class="hud">ARRIVI / DA ASSEGNARE</div>
      <div class="hud">
        <span style="color: var(--accent-pending)">{pendingCount}</span>
        <span style="color: var(--fg-3)"> / {finishers.length}</span>
      </div>
    </div>

    {#if rows.length === 0}
      <div class="flex-1 flex items-center justify-center px-4 py-8">
        <div class="text-center">
          <div class="hud mb-1" style="color: var(--fg-3)">NESSUN ARRIVO</div>
          <div class="text-xs" style="color: var(--fg-3)">
            {#if started}
              Premi <span class="kbd">RECORD</span>
              {#if active}o <span class="kbd">␣</span>{/if}
              per catturare un tempo
            {:else}
              Avvia il percorso per iniziare a cronometrare
            {/if}
          </div>
        </div>
      </div>
    {:else}
      <ul class="flex-1 overflow-auto">
        {#each rows as r (r.kind === 'pending' ? `p-${r.p.id}` : `f-${r.f.athlete.id}`)}
          {#if r.kind === 'pending'}
            <li class="row row-pending slide-in">
              <span class="pos-chip">{r.pos}</span>
              <span class="row-time num">
                {formatMsToHms(r.p.finish_timestamp_ms % 86_400_000)}
              </span>
              <form
                class="row-main"
                onsubmit={(e) => {
                  e.preventDefault();
                  doAssign(r.p.id);
                }}
              >
                <input
                  type="number"
                  inputmode="numeric"
                  placeholder="BIB"
                  class="bib-input num"
                  value={bibInputs[r.p.id] ?? ''}
                  oninput={(e) =>
                    (bibInputs = {
                      ...bibInputs,
                      [r.p.id]: (e.currentTarget as HTMLInputElement).value,
                    })}
                />
                <span class="row-hint">da assegnare</span>
                <button
                  type="submit"
                  class="btn-row btn-row-confirm"
                  title="Assegna pettorale"
                  aria-label="Assegna pettorale"
                >
                  ✓
                </button>
                <button
                  type="button"
                  class="btn-row btn-row-danger"
                  title="Elimina tempo"
                  aria-label="Elimina tempo"
                  onclick={(e) => {
                    e.stopPropagation();
                    doDeletePending(r.p.id, r.p.finish_timestamp_ms);
                  }}
                >
                  ✕
                </button>
              </form>
            </li>
          {:else}
            {@const isEditing = editingTimingId === r.f.timing_id && r.f.timing_id != null}
            <li class="row row-finish">
              <span class="pos-chip pos-chip-done">{r.pos}</span>
              <span class="row-time num">
                {formatMsToHms(r.f.total_ms ?? 0)}
              </span>
              {#if isEditing}
                <form
                  class="row-main"
                  onsubmit={(e) => {
                    e.preventDefault();
                    if (r.f.timing_id != null) commitEditBib(r.f.timing_id);
                  }}
                >
                  <input
                    type="number"
                    inputmode="numeric"
                    class="bib-input num"
                    bind:value={editBibInput}
                    placeholder="BIB"
                    autocomplete="off"
                  />
                  <span class="row-hint">nuovo pettorale</span>
                  <button
                    type="submit"
                    class="btn-row btn-row-confirm"
                    title="Salva"
                    aria-label="Salva"
                  >
                    ✓
                  </button>
                  <button
                    type="button"
                    class="btn-row"
                    title="Annulla modifica"
                    aria-label="Annulla modifica"
                    onclick={(e) => {
                      e.stopPropagation();
                      cancelEditBib();
                    }}
                  >
                    ⌫
                  </button>
                </form>
              {:else}
                <div class="row-main">
                  <span class="row-bib num">#{r.f.athlete.bib_number}</span>
                  <span class="row-name truncate">
                    {r.f.athlete.first_name} {r.f.athlete.last_name}
                  </span>
                  {#if r.f.timing_id != null}
                    <button
                      type="button"
                      class="btn-row"
                      title="Modifica pettorale"
                      aria-label="Modifica pettorale"
                      onclick={(e) => {
                        e.stopPropagation();
                        startEditBib(r.f.timing_id!, r.f.athlete.bib_number);
                      }}
                    >
                      ✎
                    </button>
                    <button
                      type="button"
                      class="btn-row btn-row-danger"
                      title="Annulla arrivo"
                      aria-label="Annulla arrivo"
                      onclick={(e) => {
                        e.stopPropagation();
                        doUndoFinish(r.f.timing_id!, r.f.athlete.bib_number);
                      }}
                    >
                      ✕
                    </button>
                  {/if}
                </div>
              {/if}
            </li>
          {/if}
        {/each}
      </ul>
    {/if}
  </div>

  {#if error}
    <div
      class="px-3 py-2 text-sm border-t shrink-0"
      style="color: var(--accent-finish); border-color: var(--line-2); background: var(--bg-1)"
    >
      {error}
    </div>
  {/if}
</section>

{#if showEndModal}
  <ConfirmRaceModal
    {course}
    variant="end"
    onClose={() => (showEndModal = false)}
    onConfirm={async (typed) => {
      await api.endCourse(course.id, typed);
      ended = true;
      await refresh();
    }}
  />
{/if}

{#if showRestartModal}
  <ConfirmRaceModal
    {course}
    variant="restart"
    onClose={() => (showRestartModal = false)}
    onConfirm={async (typed) => {
      await api.restartCourse(course.id, typed);
      ended = false;
      started = false;
      elapsed = 0;
      pending = [];
      finishers = [];
      bibInputs = {};
      await refresh();
    }}
  />
{/if}

<style>
  .lane-card {
    overflow: hidden;
  }
  .row {
    display: grid;
    grid-template-columns: 2.25rem 5.5rem 1fr;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--line-1);
    font-size: 0.9rem;
    transition: background 100ms;
  }
  .row:last-child {
    border-bottom: none;
  }
  .row-pending {
    background: rgba(192, 138, 42, 0.08);
    border-left: 3px solid var(--accent-pending);
    padding-left: calc(0.75rem - 3px);
  }
  .row-pending:hover {
    background: rgba(192, 138, 42, 0.14);
  }
  .row-finish {
    border-left: 3px solid transparent;
    padding-left: calc(0.75rem - 3px);
  }
  .row-finish:hover {
    background: var(--bg-2);
  }
  .pos-chip {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 2rem;
    padding: 0.18rem 0.4rem;
    border-radius: var(--radius-pill);
    background: var(--bg-3);
    color: var(--fg-1);
    font-family: 'IBM Plex Mono', ui-monospace, monospace;
    font-variant-numeric: tabular-nums;
    font-weight: 700;
    font-size: 0.78rem;
    letter-spacing: 0.02em;
  }
  .pos-chip-done {
    background: var(--accent-running);
    color: #f6f2e9;
  }
  .row-time {
    color: var(--fg-0);
    font-weight: 700;
    font-size: 0.95rem;
    letter-spacing: -0.01em;
  }
  .row-main {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    min-width: 0;
  }
  .row-bib {
    color: var(--accent-running);
    font-weight: 800;
    flex-shrink: 0;
  }
  .row-name {
    color: var(--fg-1);
    font-weight: 500;
    flex: 1;
    min-width: 0;
  }
  .row-hint {
    color: var(--fg-3);
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    flex: 1;
  }
  .bib-input {
    width: 4.5rem;
    text-align: center;
    padding: 0.3rem 0.4rem;
    font-size: 0.95rem;
    font-weight: 700;
    flex-shrink: 0;
  }
  .btn-row {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.85rem;
    height: 1.85rem;
    flex-shrink: 0;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    background: transparent;
    color: var(--fg-3);
    cursor: pointer;
    font-size: 0.9rem;
    line-height: 1;
    transition:
      background 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }
  .btn-row:hover {
    background: var(--bg-3);
    color: var(--fg-0);
    border-color: var(--line-2);
  }
  .btn-row-confirm {
    color: var(--accent-start);
    border-color: rgba(74, 140, 79, 0.35);
  }
  .btn-row-confirm:hover {
    background: var(--accent-start);
    color: #f6f2e9;
    border-color: var(--accent-start);
  }
  .btn-row-danger:hover {
    background: rgba(184, 85, 58, 0.12);
    color: var(--accent-finish);
    border-color: rgba(184, 85, 58, 0.35);
  }
  .btn-row-danger:active {
    background: rgba(184, 85, 58, 0.2);
  }
  .btn-header-end,
  .btn-header-restart {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.32rem 0.65rem;
    border-radius: var(--radius-pill);
    border: 1px solid currentColor;
    background: transparent;
    cursor: pointer;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    transition:
      background 120ms ease,
      color 120ms ease,
      transform 80ms ease;
  }
  .btn-header-end {
    color: var(--accent-finish);
  }
  .btn-header-end:hover {
    background: var(--accent-finish);
    color: #f6f2e9;
  }
  .btn-header-restart {
    color: var(--accent-pending);
  }
  .btn-header-restart:hover {
    background: var(--accent-pending);
    color: #f6f2e9;
  }
  .btn-header-end:active,
  .btn-header-restart:active {
    transform: translateY(1px);
  }
  .btn-tie {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.1rem;
    padding: 0.6rem 0.9rem;
    border-radius: var(--radius-md);
    border: 1px dashed var(--accent-pending);
    background: transparent;
    color: var(--accent-pending);
    cursor: pointer;
    line-height: 1;
    transition:
      background 120ms ease,
      color 120ms ease,
      transform 80ms ease,
      border-style 120ms ease;
  }
  .btn-tie:hover:not(:disabled) {
    background: var(--accent-pending);
    color: #fbf8ed;
    border-style: solid;
  }
  .btn-tie:active:not(:disabled) {
    transform: translateY(1px);
  }
  .btn-tie:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .tie-label {
    font-size: 0.55rem;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
</style>
