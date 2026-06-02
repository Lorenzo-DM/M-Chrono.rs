<script lang="ts">
  import { api } from '../api';
  import { config, isAuthenticated as isAuthStore, courses } from '../stores';
  import DeviceLoginModal from './DeviceLoginModal.svelte';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';

  let { onBack }: { onBack: () => void } = $props();
  let operatorId = $state($config?.operator_id ?? '');
  let saving = $state(false);
  let saved = $state(false);
  let showLogin = $state(false);
  let authed = $state(false);

  $effect(() => {
    api.isAuthenticated().then(b => { authed = b; isAuthStore.set(b); });
  });

  async function doLogout() {
    await api.logout();
    authed = false;
    isAuthStore.set(false);
  }

  async function save() {
    saving = true;
    saved = false;
    try {
      await api.updateOperatorId(operatorId);
      const cfg = await api.getConfig();
      config.set(cfg);
      saved = true;
    } finally {
      saving = false;
    }
  }
</script>

<div class="flex items-center mb-4 gap-4">
  <button class="btn preset-tonal" onclick={onBack}>← Home</button>
  <h2 class="text-3xl font-bold">Settings</h2>
</div>

<section class="max-w-md flex flex-col gap-4">
  <label class="flex flex-col gap-1">
    <span class="text-xl">Operator ID</span>
    <input bind:value={operatorId}
           class="p-3 bg-black border-2 border-white text-xl"
           placeholder="PC-A" />
  </label>
  <button class="btn preset-filled text-xl py-3" disabled={saving} onclick={save}>
    {saving ? 'Salvataggio…' : 'Salva'}
  </button>
  {#if saved}<p class="text-green-400">Salvato.</p>{/if}
</section>

<section class="max-w-md mt-6">
  <h3 class="text-xl mb-2">Autenticazione</h3>
  {#if authed}
    <p class="text-green-400 mb-2">Login attivo</p>
    <button class="btn preset-tonal" onclick={doLogout}>Logout</button>
  {:else}
    <p class="opacity-70 mb-2">Non autenticato</p>
    <button class="btn preset-filled" onclick={() => showLogin = true}>Accedi</button>
  {/if}
</section>

<section class="max-w-md mt-6">
  <h3 class="text-xl mb-2">Dati gara</h3>
  <button class="btn preset-filled"
          onclick={async () => {
            try {
              const s = await api.fetchRemoteData();
              alert(`Scaricati ${s.courses_count} percorsi, ${s.athletes_count} atleti`);
              courses.set(await api.getCourses());
            } catch (e: any) {
              alert(e?.message ?? String(e));
            }
          }}>
    Sincronizza atleti/percorsi
  </button>
</section>

<section class="max-w-md mt-6">
  <h3 class="text-xl mb-2">Esporta risultati</h3>
  <button class="btn preset-filled"
          onclick={async () => {
            const date = new Date().toISOString().slice(0, 10);
            const path = await saveDialog({
              defaultPath: `risultati_${date}.xlsx`,
              filters: [{ name: 'Excel', extensions: ['xlsx'] }],
            });
            if (!path) return;
            try {
              const s = await api.exportResultsXlsx(path);
              alert(`Esportati ${s.athletes_count} atleti su ${s.courses_count} percorsi`);
            } catch (e: any) {
              alert(e?.message ?? String(e));
            }
          }}>
    Esporta XLSX
  </button>
</section>

{#if showLogin}
  <DeviceLoginModal onClose={(ok) => {
    showLogin = false;
    if (ok) { authed = true; isAuthStore.set(true); }
  }} />
{/if}
