<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from './lib/api';
  import {
    courses, config, activeCourseId, isAuthenticated, type NavView,
    startDisplayPoll, stopDisplayPoll, refreshAthletes, refreshCheckpoints,
  } from './lib/stores';
  import { on } from './lib/events';
  import Header from './lib/components/Header.svelte';
  import Workspace from './lib/components/Workspace.svelte';
  import SettingsPage from './lib/components/SettingsPage.svelte';
  import ResultsPage from './lib/components/ResultsPage.svelte';
  import ExportPage from './lib/components/ExportPage.svelte';
  import DuplicateReviewModal from './lib/components/DuplicateReviewModal.svelte';
  import IntroFlow from './lib/components/IntroFlow.svelte';

  type View = NavView | 'intro';
  let view = $state<View>('timing');
  let booted = $state(false);
  let showDup = $state(false);

  function needsIntro(): boolean {
    return !$config?.operator_id?.trim();
  }

  onMount(() => {
    (async () => {
      const [c, cfg, authed] = await Promise.all([
        api.getCourses(),
        api.getConfig(),
        api.isAuthenticated(),
      ]);
      courses.set(c);
      config.set(cfg);
      isAuthenticated.set(authed);
      if (c.length > 0) activeCourseId.set(c[0].id);
      view = needsIntro() ? 'intro' : 'timing';
      booted = true;
      await Promise.all([refreshAthletes(), refreshCheckpoints()]);
    })();

    // Single shared display poll + roster refresh on data-changing events.
    startDisplayPoll();
    const unsubs: Array<() => void> = [];
    const refreshCourses = () => api.getCourses().then(courses.set);
    on('data:changed', () => {
      refreshAthletes();
      refreshCheckpoints();
      refreshCourses();
    }).then((u) => unsubs.push(u));
    on('athlete:finished', () => refreshAthletes()).then((u) => unsubs.push(u));
    // Course lifecycle changes started_at/ended_at; keep the store in sync so
    // status dots reflect the real state.
    on('course:started', refreshCourses).then((u) => unsubs.push(u));
    on('course:ended', refreshCourses).then((u) => unsubs.push(u));
    on('course:reset', refreshCourses).then((u) => unsubs.push(u));

    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const inInput = target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA';
      if (inInput) return;
      if (view !== 'timing') return;
      if (e.key >= '1' && e.key <= '9') {
        const idx = parseInt(e.key) - 1;
        const list = $courses;
        const cur = list[idx];
        if (cur) activeCourseId.set(cur.id);
      } else if (e.key.toLowerCase() === 'd') {
        showDup = true;
      }
    };
    window.addEventListener('keydown', onKey);

    return () => {
      window.removeEventListener('keydown', onKey);
      stopDisplayPoll();
      for (const u of unsubs) u();
    };
  });

  async function onIntroReady() {
    const [c, cfg, authed] = await Promise.all([
      api.getCourses(),
      api.getConfig(),
      api.isAuthenticated(),
    ]);
    courses.set(c);
    config.set(cfg);
    isAuthenticated.set(authed);
    if (c.length > 0) activeCourseId.set(c[0].id);
    view = 'timing';
  }

  function nav(v: NavView) {
    view = v;
  }
</script>

{#if !booted}
  <main class="min-h-screen flex items-center justify-center">
    <div class="text-center">
      <div
        class="text-2xl font-semibold"
        style="color: var(--fg-0); letter-spacing: -0.01em"
      >
        M-Chrono
      </div>
      <div class="hud mt-2" style="color: var(--fg-3)">CARICAMENTO…</div>
    </div>
  </main>
{:else if view === 'intro'}
  <IntroFlow onReady={onIntroReady} />
{:else}
  <div class="h-screen flex flex-col">
    <Header current={view} onNav={nav} onDuplicates={() => (showDup = true)} />

    <main class="flex-1 min-h-0 overflow-hidden">
      {#if view === 'timing'}
        <div class="h-full min-h-0">
          <Workspace />
        </div>
      {:else if view === 'settings'}
        <div class="h-full overflow-auto">
          <SettingsPage onBack={() => (view = 'timing')} />
        </div>
      {:else if view === 'results'}
        <div class="h-full overflow-auto">
          <ResultsPage />
        </div>
      {:else if view === 'export'}
        <div class="h-full overflow-auto">
          <ExportPage />
        </div>
      {/if}
    </main>
  </div>
{/if}

{#if showDup}
  <DuplicateReviewModal onClose={() => (showDup = false)} />
{/if}
