<script lang="ts">
  import { api } from '../api';
  import { config } from '../stores';
  import Button from '../ui/Button.svelte';
  import AthleteImportPanel from './AthleteImportPanel.svelte';
  import RaceSetupPanel from './RaceSetupPanel.svelte';
  import { TriangleAlert, ArrowLeft, ArrowRight } from 'lucide-svelte';

  let { onReady }: { onReady: () => void } = $props();

  // Steps: 0 operator, 1 race+courses, 2 athletes
  let step = $state(0);
  let operatorId = $state($config?.operator_id ?? '');
  let saving = $state(false);
  let error = $state<string | null>(null);
  let imported = $state(false);
  let raceConfigured = $state(false);

  const stepCount = 3;
  const stepLabels = ['OPERATORE', 'GARA', 'ATLETI'];

  async function nextStep() {
    if (step === 0) {
      error = null;
      if (!operatorId.trim()) {
        error = 'nome operatore obbligatorio';
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
        <div class="hud" style="color: var(--fg-3)">CONFIGURAZIONE INIZIALE</div>
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
          {#each stepLabels as label, i (label)}
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
          <div class="hud reveal reveal-1" style="color: var(--fg-3)">BENVENUTO</div>
          <h1 class="text-5xl font-semibold mt-3 reveal reveal-2"
              style="color: var(--fg-0); letter-spacing: -0.02em; line-height: 1.05">
            Chi sta<br>cronometrando?
          </h1>
          <p class="mt-6 text-lg leading-relaxed reveal reveal-3" style="color: var(--fg-1); max-width: 56ch">
            Un nome che identifica questa postazione nei record di
            cronometraggio. Tutto il resto — atleti, sincronizzazione, tema —
            si configura dopo, quando serve.
          </p>

          <label class="mt-10 max-w-md block reveal reveal-4">
            <span class="hud block mb-2">NOME OPERATORE</span>
            <!-- svelte-ignore a11y_autofocus -->
            <input bind:value={operatorId} placeholder="es. PC-A, PC-B"
                   autofocus class="w-full text-2xl py-3" />
          </label>
        </section>

      {:else if step === 1}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">PASSAGGIO 02 / 03</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            Crea la gara e i percorsi
          </h2>
          <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
            Dai un nome alla gara e aggiungi i percorsi (es. 21K, 10K).
            Puoi saltare e configurarli dopo dalle impostazioni.
          </p>

          <div class="mt-8">
            <RaceSetupPanel onChange={() => (raceConfigured = true)} />
          </div>
        </section>

      {:else}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">PASSAGGIO 03 / 03</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            Importa gli atleti
          </h2>
          <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
            Carica un foglio Excel o CSV, aggiungili a mano, oppure salta:
            puoi importarli in qualsiasi momento dalle impostazioni.
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
        <Button variant="ghost" onclick={() => (step -= 1)}><ArrowLeft size={14} /> INDIETRO</Button>
      {/if}
    </div>

    <div class="flex items-center gap-3">
      {#if step === 1 && !raceConfigured}
        <Button variant="ghost" onclick={() => (step = 2)} title="Configura la gara più tardi dalle impostazioni">
          SALTA — CONFIGURA PIÙ TARDI
        </Button>
      {/if}
      {#if step === 2 && !imported}
        <Button variant="ghost" onclick={onReady} title="Importa più tardi dalle impostazioni">
          SALTA — IMPORTA PIÙ TARDI
        </Button>
      {/if}
      <Button variant="primary" class="px-6 py-3" disabled={saving} onclick={nextStep}>
        {#if step === 2}
          AVVIA WORKSPACE <ArrowRight size={14} />
        {:else}
          AVANTI <ArrowRight size={14} />
        {/if}
      </Button>
    </div>
  </footer>
</div>
