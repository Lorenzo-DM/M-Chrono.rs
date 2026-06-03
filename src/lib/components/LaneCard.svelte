<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { on } from '../events';
  import { formatMsToHms } from '../format';
  import type { Course, PendingFinish, AthleteRow } from '../types';

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
  let pending = $state<PendingFinish[]>([]);
  let finishers = $state<AthleteRow[]>([]);
  let flashing = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let bibInputs = $state<Record<number, string>>({});

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

  function handleKey(e: KeyboardEvent) {
    if (!active) return;
    const tgt = e.target as HTMLElement | null;
    if (tgt?.tagName === 'INPUT') return;
    if (!started) return;
    if (e.code === 'Space') {
      e.preventDefault();
      doRecord();
    }
  }

  type Row =
    | { kind: 'pending'; p: PendingFinish; t: number }
    | { kind: 'finish'; f: AthleteRow; t: number };

  let rows = $derived<Row[]>(
    [
      ...pending.map<Row>((p) => ({ kind: 'pending', p, t: p.finish_timestamp_ms })),
      ...finishers.map<Row>((f) => ({ kind: 'finish', f, t: f.finish_ms ?? 0 })),
    ].sort((a, b) => b.t - a.t),
  );

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
    class="px-4 py-3 border-b flex items-center justify-between shrink-0"
    style="background: var(--bg-2); border-color: var(--line-2)"
  >
    <div class="flex flex-col">
      <div class="hud-strong text-base" style="color: var(--fg-0)">{course.name}</div>
      {#if course.distance_m}
        <div class="hud mt-0.5" style="color: var(--fg-2)">
          {(course.distance_m / 1000).toFixed(1)} KM
        </div>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      <span class={started ? 'dot-running' : 'dot-idle'}></span>
      <span
        class="lane-status"
        style="color: {started ? 'var(--accent-running)' : 'var(--fg-3)'}"
      >
        {started ? 'ACTIVE' : 'STANDBY'}
      </span>
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
    {:else}
      <button
        class="btn-base btn-accent-tap w-full text-base font-semibold"
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
            <li
              class="row row-pending slide-in"
              style="border-color: var(--line-1)"
            >
              <div class="num text-base" style="color: var(--fg-0); font-weight: 600">
                {formatMsToHms(r.p.finish_timestamp_ms % 86_400_000)}
              </div>
              <form
                class="flex items-center gap-2 ml-auto"
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
                <button
                  type="submit"
                  class="btn-base btn-primary"
                  style="padding: 0.4rem 0.7rem; font-size: 0.85rem"
                  title="Assegna"
                >
                  ✓
                </button>
              </form>
            </li>
          {:else}
            <li class="row row-finish" style="border-color: var(--line-1)">
              <div class="num" style="color: var(--accent-running); font-weight: 700; min-width: 3rem">
                #{r.f.athlete.bib_number}
              </div>
              <div
                class="flex-1 min-w-0 truncate"
                style="color: var(--fg-1); font-weight: 500"
              >
                {r.f.athlete.first_name} {r.f.athlete.last_name}
              </div>
              <div class="num text-sm" style="color: var(--fg-0); font-weight: 600">
                {formatMsToHms(r.f.total_ms ?? 0)}
              </div>
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

<style>
  .lane-card {
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.55rem 0.75rem;
    border-bottom: 1px solid;
    font-size: 0.9rem;
  }
  .row:last-child {
    border-bottom: none;
  }
  .row-pending {
    background: rgba(160, 112, 29, 0.07);
    border-left: 3px solid var(--accent-pending);
  }
  .row-pending:hover {
    background: rgba(160, 112, 29, 0.12);
  }
  .row-finish:hover {
    background: var(--bg-2);
  }
  .bib-input {
    width: 5rem;
    text-align: center;
    padding: 0.35rem 0.4rem;
    font-size: 0.95rem;
    font-weight: 600;
  }
</style>
