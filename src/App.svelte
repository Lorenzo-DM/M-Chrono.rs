<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from './lib/api';
  import { courses, config, activeCourseId, isAuthenticated, type NavView } from './lib/stores';
  import Header from './lib/components/Header.svelte';
  import Workspace from './lib/components/Workspace.svelte';
  import PendingDock from './lib/components/PendingDock.svelte';
  import SettingsPage from './lib/components/SettingsPage.svelte';
  import DuplicateReviewModal from './lib/components/DuplicateReviewModal.svelte';
  import IntroFlow from './lib/components/IntroFlow.svelte';
  import Button from './lib/ui/Button.svelte';

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
    })();

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
        TrailTrace<span style="color: var(--accent-running)">·</span>Chrono
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
        <div class="h-full min-h-0 flex">
          <div class="flex-1 min-w-0 h-full">
            <Workspace />
          </div>
          <PendingDock />
        </div>
      {:else if view === 'settings'}
        <SettingsPage onBack={() => (view = 'timing')} />
      {:else if view === 'results'}
        <div class="h-full flex items-center justify-center p-8">
          <div class="panel max-w-md w-full text-center" style="padding: 2rem 2rem">
            <div class="hud-strong mb-2" style="color: var(--fg-0)">RESULTS</div>
            <div class="hud mb-4" style="color: var(--accent-pending)">PROSSIMAMENTE</div>
            <div class="text-sm" style="color: var(--fg-2)">
              Classifica globale e per percorso, filtri per categoria, ordinamento e
              ricerca. Per ora consulta la coda di arrivi sotto ciascun timer.
            </div>
            <Button class="mt-5" onclick={() => (view = 'timing')}>← TIMING</Button>
          </div>
        </div>
      {:else if view === 'export'}
        <div class="h-full flex items-center justify-center p-8">
          <div class="panel max-w-md w-full text-center" style="padding: 2rem 2rem">
            <div class="hud-strong mb-2" style="color: var(--fg-0)">EXPORT</div>
            <div class="hud mb-4" style="color: var(--accent-pending)">PROSSIMAMENTE</div>
            <div class="text-sm" style="color: var(--fg-2)">
              Esportazione XLSX dei risultati per percorso. Disponibile a breve. Nel
              frattempo i tempi catturati vengono salvati localmente e sincronizzati
              con il cloud.
            </div>
            <Button class="mt-5" onclick={() => (view = 'timing')}>← TIMING</Button>
          </div>
        </div>
      {/if}
    </main>
  </div>
{/if}

{#if showDup}
  <DuplicateReviewModal onClose={() => (showDup = false)} />
{/if}
