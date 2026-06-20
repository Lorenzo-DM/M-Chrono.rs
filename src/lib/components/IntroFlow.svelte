<script lang="ts">
  import { api } from '../api';
  import { config } from '../stores';
  import Button from '../ui/Button.svelte';
  import AthleteImportPanel from './AthleteImportPanel.svelte';
  import RaceSetupPanel from './RaceSetupPanel.svelte';
  import { TriangleAlert, ArrowLeft, ArrowRight } from 'lucide-svelte';
  import { locale, SUPPORTED_LOCALES, t } from '../i18n';
  import type { Locale } from '../i18n';

  let { onReady }: { onReady: () => void } = $props();

  // Steps: 0 operator, 1 race+courses, 2 athletes
  let step = $state(0);
  let operatorId = $state($config?.operator_id ?? '');
  let saving = $state(false);
  let error = $state<string | null>(null);
  let imported = $state(false);
  let raceConfigured = $state(false);

  const stepCount = 3;

  async function nextStep() {
    if (step === 0) {
      error = null;
      if (!operatorId.trim()) {
        error = $t.intro.step0.operatorRequired;
        return;
      }
      saving = true;
      try {
        const cfg = await api.updateConfig({ operator_id: operatorId.trim() });
        config.set(cfg);
        step = 1;
      } catch (e: any) {
        error = e?.message ?? JSON.stringify(e);
      } finally { saving = false; }
      return;
    }
    if (step === 1) {
      step = 2;
      return;
    }
    onReady();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && step === 0) nextStep();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="min-h-screen flex flex-col bg-[var(--bg-0)]">
  <!-- Top bar with brand + step rail -->
  <header class="px-8 py-5 border-b" style="border-color: var(--line-2)">
    <div class="flex items-end justify-between gap-8">
      <div>
        <div class="hud" style="color: var(--fg-3)">{$t.intro.initialSetup}</div>
        <div class="text-2xl font-semibold mt-1" style="color: var(--fg-0); letter-spacing: -0.01em">
          M-Chrono
        </div>
      </div>

      <div class="flex-1 max-w-md">
        <div class="step-rail" style="grid-template-columns: repeat({stepCount}, 1fr);">
          {#each Array(stepCount) as _, i (i)}
            <div class="step-pip"
                 data-state={i < step ? 'done' : (i === step ? 'active' : 'idle')}></div>
          {/each}
        </div>
        <div class="flex justify-between mt-2">
          {#each $t.intro.stepLabels as label, i (label)}
            <div class="hud text-[0.62rem]"
                 style="color: {i === step ? 'var(--fg-0)' : (i < step ? 'var(--fg-2)' : 'var(--fg-3)')}">
              {String(i + 1).padStart(2, '0')} · {label}
            </div>
          {/each}
        </div>
      </div>
    </div>
  </header>

  <!-- Step body -->
  <main class="flex-1 overflow-auto">
    <div class="max-w-3xl mx-auto px-8 py-10">

      {#if step === 0}
        <section class="reveal">
          <div class="hud reveal reveal-1" style="color: var(--fg-3)">{$t.intro.step0.welcome}</div>
          <h1 class="text-5xl font-semibold mt-3 reveal reveal-2"
              style="color: var(--fg-0); letter-spacing: -0.02em; line-height: 1.05">
            {#each $t.intro.step0.title.split('\n') as line, idx (idx)}
              {line}{#if idx < $t.intro.step0.title.split('\n').length - 1}<br>{/if}
            {/each}
          </h1>
          <p class="mt-6 text-lg leading-relaxed reveal reveal-3" style="color: var(--fg-1); max-width: 56ch">
            {$t.intro.step0.description}
          </p>

          <div class="mt-8 max-w-md reveal reveal-4">
            <span class="hud block mb-2">{$t.locale.label}</span>
            <select
              value={$locale}
              onchange={(e) => locale.set((e.target as HTMLSelectElement).value as Locale)}
            >
              {#each SUPPORTED_LOCALES as l (l.code)}
                <option value={l.code}>{l.nativeName}</option>
              {/each}
            </select>
          </div>

          <label class="mt-6 max-w-md block reveal reveal-5">
            <span class="hud block mb-2">{$t.intro.step0.operatorLabel}</span>
            <!-- svelte-ignore a11y_autofocus -->
            <input bind:value={operatorId} placeholder={$t.intro.step0.operatorPlaceholder}
                   autofocus class="w-full text-2xl py-3" />
          </label>
        </section>

      {:else if step === 1}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">{$t.intro.step1.step}</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            {$t.intro.step1.title}
          </h2>
          <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
            {$t.intro.step1.description}
          </p>

          <div class="mt-8">
            <RaceSetupPanel onChange={() => (raceConfigured = true)} />
          </div>
        </section>

      {:else}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">{$t.intro.step2.step}</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            {$t.intro.step2.title}
          </h2>
          <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
            {$t.intro.step2.description}
          </p>

          <div class="mt-8">
            <AthleteImportPanel onImported={() => (imported = true)} />
          </div>
        </section>
      {/if}

      {#if error}
        <div class="hud mt-6" style="color: var(--accent-finish)"><TriangleAlert size={14} /> {error}</div>
      {/if}
    </div>
  </main>

  <!-- Footer nav -->
  <footer class="px-8 py-4 border-t flex items-center justify-between"
          style="border-color: var(--line-2); background: var(--bg-1)">
    <div>
      {#if step > 0}
        <Button variant="ghost" onclick={() => (step -= 1)}><ArrowLeft size={14} /> {$t.common.back}</Button>
      {/if}
    </div>

    <div class="flex items-center gap-3">
      {#if step === 1 && !raceConfigured}
        <Button variant="ghost" onclick={() => (step = 2)} title={$t.intro.skipConfigureLater}>
          {$t.intro.skipConfigureLater}
        </Button>
      {/if}
      {#if step === 2 && !imported}
        <Button variant="ghost" onclick={onReady} title={$t.intro.skipImportLater}>
          {$t.intro.skipImportLater}
        </Button>
      {/if}
      <Button variant="primary" class="px-6 py-3" disabled={saving} onclick={nextStep}>
        {#if step === 2}
          {$t.intro.launchWorkspace} <ArrowRight size={14} />
        {:else}
          {$t.common.next} <ArrowRight size={14} />
        {/if}
      </Button>
    </div>
  </footer>
</div>
