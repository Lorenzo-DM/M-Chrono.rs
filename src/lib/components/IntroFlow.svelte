<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { on } from '../events';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { config, courses, isAuthenticated } from '../stores';
  import type { AppConfig, DeviceCodeResponse } from '../types';
  import Button from '../ui/Button.svelte';

  let { onReady }: { onReady: () => void } = $props();

  // Steps: 0 welcome, 1 operator, 2 oidc, 3 api, 4 login, 5 sync, 6 done
  let step = $state(0);

  let form = $state<AppConfig>({
    oidc_issuer_url: $config?.oidc_issuer_url ?? 'http://localhost:8787',
    oidc_client_id: $config?.oidc_client_id ?? 'mock-client',
    oidc_scopes: $config?.oidc_scopes ?? 'openid profile email offline_access',
    api_base_url: $config?.api_base_url ?? 'http://localhost:8787',
    sync_interval_secs: $config?.sync_interval_secs ?? 10,
    operator_id: $config?.operator_id ?? '',
    dedup_window_ms: $config?.dedup_window_ms ?? 2000,
    dedup_warn_delta_ms: $config?.dedup_warn_delta_ms ?? 500,
  });

  let savingConfig = $state(false);
  let configError = $state<string | null>(null);

  let devResp = $state<DeviceCodeResponse | null>(null);
  let loginCountdown = $state(0);
  let loginError = $state<string | null>(null);
  let loginPending = $state(false);
  let countdownInterval: number | null = null;
  let unsubAuthOk: (() => void) | null = null;
  let unsubAuthFail: (() => void) | null = null;

  let syncSummary = $state<string | null>(null);
  let syncing = $state(false);
  let syncError = $state<string | null>(null);

  const stepCount = 6;
  const stepLabels = ['INTRO', 'OPERATORE', 'OIDC', 'API', 'LOGIN', 'DATI'];

  async function persistConfig(): Promise<boolean> {
    savingConfig = true;
    configError = null;
    try {
      const cfg = await api.updateConfig({
        oidc_issuer_url: form.oidc_issuer_url,
        oidc_client_id: form.oidc_client_id,
        oidc_scopes: form.oidc_scopes,
        api_base_url: form.api_base_url,
        sync_interval_secs: Number(form.sync_interval_secs),
        operator_id: form.operator_id,
        dedup_window_ms: Number(form.dedup_window_ms),
        dedup_warn_delta_ms: Number(form.dedup_warn_delta_ms),
      });
      config.set(cfg);
      return true;
    } catch (e: any) {
      configError = e?.message ?? String(e);
      return false;
    } finally { savingConfig = false; }
  }

  async function nextStep() {
    if (step === 1 && !form.operator_id.trim()) {
      configError = 'operator_id obbligatorio';
      return;
    }
    if (step === 2 || step === 3 || step === 1) {
      if (!(await persistConfig())) return;
    }
    if (step === 4) {
      // expect login already complete
      const ok = await api.isAuthenticated();
      if (!ok) { loginError = 'login non completato'; return; }
      isAuthenticated.set(true);
    }
    if (step === 5) {
      onReady();
      return;
    }
    step += 1;
    if (step === 4 && !devResp && !$isAuthenticated) {
      startLogin();
    }
  }

  function prevStep() {
    if (step > 0) step -= 1;
  }

  function skipLogin() {
    loginError = null;
    step += 1;
  }

  async function startLogin() {
    loginError = null;
    loginPending = true;
    try {
      devResp = await api.startDeviceLogin();
      loginCountdown = devResp.expires_in;
      countdownInterval = window.setInterval(() => {
        loginCountdown = Math.max(0, loginCountdown - 1);
      }, 1000);
      on('auth:success', async () => {
        if (countdownInterval !== null) clearInterval(countdownInterval);
        isAuthenticated.set(true);
        loginPending = false;
        // auto-advance after success
        step = 5;
      }).then(u => { unsubAuthOk = u; });
      on('auth:failed', (p: any) => {
        if (countdownInterval !== null) clearInterval(countdownInterval);
        loginError = p?.reason ?? 'login fallito';
        loginPending = false;
      }).then(u => { unsubAuthFail = u; });
    } catch (e: any) {
      loginError = e?.message ?? String(e);
      loginPending = false;
    }
  }

  async function runSync() {
    syncing = true;
    syncError = null;
    syncSummary = null;
    try {
      const s = await api.fetchRemoteData();
      syncSummary = `${s.courses_count} percorsi · ${s.athletes_count} atleti`;
      courses.set(await api.getCourses());
    } catch (e: any) {
      syncError = e?.message ?? String(e);
    } finally { syncing = false; }
  }

  function openVerification() {
    if (!devResp) return;
    openUrl(devResp.verification_uri_complete ?? devResp.verification_uri);
  }

  onMount(() => {
    return () => {
      if (countdownInterval !== null) clearInterval(countdownInterval);
      unsubAuthOk?.();
      unsubAuthFail?.();
    };
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.target as HTMLElement)?.tagName !== 'TEXTAREA') {
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === 'INPUT') return; // input enter could submit, but we want explicit button
      nextStep();
    }
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
          TrailTrace<span style="color: var(--accent-running)">·</span>Chrono
        </div>
      </div>

      <div class="flex-1 max-w-3xl">
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
            Configura<br>la tua workstation
          </h1>
          <p class="mt-6 text-lg leading-relaxed reveal reveal-3" style="color: var(--fg-1); max-width: 56ch">
            Pochi passaggi ti separano dall'avvio della gara. Imposterai il
            nome operatore, il provider di autenticazione, l'endpoint API,
            poi accederai e scaricherai atleti e percorsi.
          </p>

          <ul class="mt-10 flex flex-col gap-3 reveal reveal-4" style="color: var(--fg-2)">
            <li class="flex items-center gap-3">
              <span class="text-xs font-mono" style="color: var(--accent-running)">01</span>
              <span>identità operatore — distingue cronometristi paralleli</span>
            </li>
            <li class="flex items-center gap-3">
              <span class="text-xs font-mono" style="color: var(--accent-running)">02</span>
              <span>OIDC issuer — autenticazione device-code via Zitadel</span>
            </li>
            <li class="flex items-center gap-3">
              <span class="text-xs font-mono" style="color: var(--accent-running)">03</span>
              <span>endpoint API — sorgente percorsi, atleti, sync arrivi</span>
            </li>
            <li class="flex items-center gap-3">
              <span class="text-xs font-mono" style="color: var(--accent-running)">04</span>
              <span>login + scarico dati iniziale</span>
            </li>
          </ul>
        </section>

      {:else if step === 1}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">PASSAGGIO 02 / 06</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            Identifica questa workstation
          </h2>
          <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
            Una label che distingue questo dispositivo nei record di
            cronometraggio. Usa <span class="kbd">PC-A</span>, <span class="kbd">PC-B</span> o un nome
            descrittivo.
          </p>

          <label class="mt-8 max-w-md block">
            <span class="hud block mb-2">OPERATOR_ID</span>
            <input bind:value={form.operator_id} placeholder="es. PC-A"
                   class="w-full text-2xl py-3" />
          </label>
        </section>

      {:else if step === 2}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">PASSAGGIO 03 / 06</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            Provider di autenticazione
          </h2>
          <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
            Indica issuer e client_id del tuo tenant Zitadel. Lo scope deve
            includere <span class="kbd">offline_access</span> per i refresh
            token (≥ 30 giorni).
          </p>

          <div class="mt-8 flex flex-col gap-4 max-w-2xl">
            <label class="flex flex-col gap-1">
              <span class="hud">ISSUER URL</span>
              <input bind:value={form.oidc_issuer_url} type="url" placeholder="https://example.zitadel.cloud" />
            </label>
            <label class="flex flex-col gap-1">
              <span class="hud">CLIENT ID</span>
              <input bind:value={form.oidc_client_id} />
            </label>
            <label class="flex flex-col gap-1">
              <span class="hud">SCOPES</span>
              <input bind:value={form.oidc_scopes} />
            </label>
          </div>
        </section>

      {:else if step === 3}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">PASSAGGIO 04 / 06</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            Endpoint dati gara
          </h2>
          <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
            Base URL del backend che espone <span class="kbd">/courses</span>,
            <span class="kbd">/athletes</span> e gli endpoint di sync. Senza
            slash finale.
          </p>

          <div class="mt-8 flex flex-col gap-4 max-w-2xl">
            <label class="flex flex-col gap-1">
              <span class="hud">API BASE URL</span>
              <input bind:value={form.api_base_url} type="url" placeholder="https://api.example.com" />
            </label>
            <div class="grid grid-cols-3 gap-3">
              <label class="flex flex-col gap-1">
                <span class="hud">SYNC (sec)</span>
                <input type="number" bind:value={form.sync_interval_secs} min="1" />
              </label>
              <label class="flex flex-col gap-1">
                <span class="hud">DEDUP WIN (ms)</span>
                <input type="number" bind:value={form.dedup_window_ms} min="0" />
              </label>
              <label class="flex flex-col gap-1">
                <span class="hud">WARN Δ (ms)</span>
                <input type="number" bind:value={form.dedup_warn_delta_ms} min="0" />
              </label>
            </div>
          </div>
        </section>

      {:else if step === 4}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">PASSAGGIO 05 / 06</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            Accedi via browser
          </h2>

          {#if $isAuthenticated}
            <div class="mt-8 max-w-md panel p-5">
              <div class="flex items-center gap-3">
                <span class="dot-running"></span>
                <span class="hud-strong" style="color: var(--accent-start)">SESSIONE ATTIVA</span>
              </div>
              <p class="mt-3" style="color: var(--fg-2)">Hai già una sessione valida. Puoi avanzare.</p>
            </div>
          {:else if !devResp && !loginError && !loginPending}
            <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
              Verrà generato un codice device. Aprilo nel browser, conferma,
              e l'app proseguirà automaticamente.
            </p>
            <Button variant="primary" class="mt-8 px-6 py-3" onclick={startLogin}>
              GENERA CODICE
            </Button>
          {:else if loginError}
            <div class="mt-8 max-w-xl panel p-5" style="border-color: var(--accent-finish)">
              <div class="hud" style="color: var(--accent-finish)">ERRORE</div>
              <p class="mt-2" style="color: var(--fg-0)">{loginError}</p>
              <Button class="mt-4" onclick={() => { loginError = null; devResp = null; startLogin(); }}>
                RIPROVA
              </Button>
            </div>
          {:else if devResp}
            <div class="mt-8 grid grid-cols-12 gap-6">
              <div class="col-span-7 panel p-6">
                <div class="hud mb-2">CODICE DA INSERIRE</div>
                <div class="chronodial text-6xl py-2 tracking-[0.18em]"
                     style="color: var(--fg-0); font-weight: 600">
                  {devResp.user_code}
                </div>
                <div class="hud mt-4 mb-1">SCADENZA</div>
                <div class="num text-lg" style="color: {loginCountdown > 60 ? 'var(--fg-1)' : 'var(--accent-finish)'}">
                  {loginCountdown}s
                </div>
              </div>

              <div class="col-span-5 panel p-6 flex flex-col">
                <div class="hud mb-2">VERIFICATION URL</div>
                <button class="text-left underline break-all"
                        style="color: var(--accent-running)"
                        onclick={openVerification}>
                  {devResp.verification_uri_complete ?? devResp.verification_uri}
                </button>
                <div class="hud mt-auto pt-4" style="color: var(--fg-3)">
                  Apri nel browser, conferma, torna qui.
                </div>
              </div>
            </div>
          {/if}
        </section>

      {:else if step === 5}
        <section class="reveal">
          <div class="hud" style="color: var(--fg-3)">PASSAGGIO 06 / 06</div>
          <h2 class="text-3xl font-semibold mt-2" style="color: var(--fg-0); letter-spacing: -0.01em">
            Scarica atleti e percorsi
          </h2>
          <p class="mt-3 max-w-xl" style="color: var(--fg-2)">
            Una sincronizzazione iniziale popola il database locale. Da quel
            momento l'app funziona offline e risincronizza in background
            quando torna la rete.
          </p>

          <div class="mt-8 flex items-center gap-4">
            <Button variant="primary" class="px-6 py-3" disabled={syncing} onclick={runSync}>
              {syncing ? 'SINCRONIZZAZIONE…' : 'SINCRONIZZA ORA'}
            </Button>
            {#if syncSummary}
              <div class="flex items-center gap-2">
                <span class="dot-running" style="background: var(--accent-start); animation: none;"></span>
                <span class="hud-strong" style="color: var(--accent-start)">{syncSummary}</span>
              </div>
            {/if}
            {#if syncError}
              <span class="hud-strong" style="color: var(--accent-finish)">⚠ {syncError}</span>
            {/if}
          </div>
        </section>
      {/if}

      {#if configError && (step === 1 || step === 2 || step === 3)}
        <div class="hud mt-6" style="color: var(--accent-finish)">⚠ {configError}</div>
      {/if}
    </div>
  </main>

  <!-- Footer nav -->
  <footer class="px-8 py-4 border-t flex items-center justify-between"
          style="border-color: var(--line-2); background: var(--bg-1)">
    <div>
      {#if step > 0}
        <Button variant="ghost" onclick={prevStep}>← INDIETRO</Button>
      {/if}
    </div>

    <div class="flex items-center gap-3">
      {#if step === 4 && !$isAuthenticated}
        <Button variant="ghost" onclick={skipLogin} title="Salta per ora">
          SALTA LOGIN
        </Button>
      {/if}
      <Button
              variant="primary"
              class="px-6 py-3"
              disabled={savingConfig || (step === 5 && !syncSummary)}
              onclick={nextStep}>
        {#if step === 5}
          AVVIA WORKSPACE →
        {:else}
          AVANTI →
        {/if}
      </Button>
    </div>
  </footer>
</div>
