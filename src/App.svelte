<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from './lib/api';
  import { courses, config, activeCourseId, recentFinishes, isAuthenticated } from './lib/stores';
  import { on } from './lib/events';
  import Header from './lib/components/Header.svelte';
  import Workspace from './lib/components/Workspace.svelte';
  import PendingDock from './lib/components/PendingDock.svelte';
  import SettingsPage from './lib/components/SettingsPage.svelte';
  import DuplicateReviewModal from './lib/components/DuplicateReviewModal.svelte';
  import IntroFlow from './lib/components/IntroFlow.svelte';

  type View = 'workspace' | 'settings' | 'intro';
  let view = $state<View>('workspace');
  let booted = $state(false);
  let showDup = $state(false);
  let dockCollapsed = $state(false);

  function needsIntro(): boolean {
    const cfg = $config;
    if (!cfg) return true;
    if (!cfg.operator_id?.trim()) return true;
    if ($courses.length === 0 && !$isAuthenticated) return true;
    return false;
  }

  onMount(() => {
    let unsubFinish: (() => void) | null = null;

    (async () => {
      const [c, cfg, authed] = await Promise.all([
        api.getCourses(), api.getConfig(), api.isAuthenticated(),
      ]);
      courses.set(c);
      config.set(cfg);
      isAuthenticated.set(authed);
      if (c.length > 0) activeCourseId.set(c[0].id);
      view = needsIntro() ? 'intro' : 'workspace';
      booted = true;

      unsubFinish = await on('athlete:finished', (t: any) => {
        if (!t) return;
        recentFinishes.update(arr => [{
          timing_id: t.id,
          course_id: t.course_id,
          bib_number: null,
          total_ms: t.total_time_ms,
          operator_id: t.operator_id,
          at_ms: Date.now(),
        }, ...arr].slice(0, 30));
      });
    })();

    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const inInput = target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA';
      if (inInput) return;
      if (view !== 'workspace') return;
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
      unsubFinish?.();
    };
  });

  async function onIntroReady() {
    // refresh state after intro completes
    const [c, cfg, authed] = await Promise.all([
      api.getCourses(), api.getConfig(), api.isAuthenticated(),
    ]);
    courses.set(c);
    config.set(cfg);
    isAuthenticated.set(authed);
    if (c.length > 0) activeCourseId.set(c[0].id);
    view = 'workspace';
  }
</script>

{#if !booted}
  <main class="min-h-screen flex items-center justify-center">
    <div class="text-center">
      <div class="text-2xl font-semibold" style="color: var(--fg-0); letter-spacing: -0.01em">
        TrailTrace<span style="color: var(--accent-running)">·</span>Chrono
      </div>
      <div class="hud mt-2" style="color: var(--fg-3)">CARICAMENTO…</div>
    </div>
  </main>
{:else if view === 'intro'}
  <IntroFlow onReady={onIntroReady} />
{:else if view === 'settings'}
  <Header
    onSettings={() => view = 'workspace'}
    onDuplicates={() => showDup = true}
  />
  <SettingsPage onBack={() => view = 'workspace'} />
{:else}
  <div class="h-screen flex flex-col">
    <Header
      onSettings={() => view = 'settings'}
      onDuplicates={() => showDup = true}
    />
    <div class="flex-1 flex min-h-0">
      <div class="flex-1 min-w-0">
        <Workspace />
      </div>
      {#if !dockCollapsed}
        <div style="width: 340px; min-width: 340px;">
          <PendingDock />
        </div>
      {/if}
      <button
        class="px-2 py-3 border-l text-xs font-medium"
        style="background: var(--bg-2); border-color: var(--line-2); color: var(--fg-2); writing-mode: vertical-rl; letter-spacing: 0.1em;"
        onclick={() => dockCollapsed = !dockCollapsed}
        title={dockCollapsed ? 'Espandi coda' : 'Riduci coda'}
      >
        {dockCollapsed ? '◀ CODA' : 'CODA ▶'}
      </button>
    </div>
  </div>
{/if}

{#if showDup}
  <DuplicateReviewModal onClose={() => showDup = false} />
{/if}
