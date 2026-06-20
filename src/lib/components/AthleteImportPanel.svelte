<script lang="ts">
  import { api } from '../api';
  import { config, courses, isAuthenticated } from '../stores';
  import type { ImportSummary } from '../types';
  import Button from '../ui/Button.svelte';
  import { TriangleAlert, Check } from 'lucide-svelte';
  import AthleteFormModal from './AthleteFormModal.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';

  let { onImported }: { onImported?: () => void } = $props();

  let importing = $state(false);
  let summary = $state<ImportSummary | null>(null);
  let importError = $state<string | null>(null);
  let showForm = $state(false);
  let fetchStatus = $state<string | null>(null);
  let fetching = $state(false);

  async function refreshCourses() {
    courses.set(await api.getCourses());
    onImported?.();
  }

  async function importFile() {
    importError = null;
    const path = await openDialog({
      multiple: false,
      filters: [
        { name: 'Fogli atleti', extensions: ['xlsx', 'csv'] },
        { name: 'Excel', extensions: ['xlsx'] },
        { name: 'CSV', extensions: ['csv'] },
      ],
    });
    if (!path) return;
    importing = true;
    summary = null;
    try {
      summary = await api.importAthletesFile(path as string);
      await refreshCourses();
    } catch (e: any) {
      importError = e?.message ?? JSON.stringify(e);
    } finally { importing = false; }
  }

  async function fetchFromServer() {
    fetching = true;
    fetchStatus = '…';
    try {
      const s = await api.fetchRemoteData();
      fetchStatus = `OK · ${s.courses_count} percorsi · ${s.athletes_count} atleti`;
      await refreshCourses();
    } catch (e: any) {
      fetchStatus = `ERRORE · ${e?.message ?? JSON.stringify(e)}`;
    } finally { fetching = false; }
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center gap-3">
    <Button variant="primary" disabled={importing} onclick={importFile}>
      {importing ? 'IMPORTAZIONE…' : 'IMPORTA FILE (XLSX/CSV)'}
    </Button>
    <Button onclick={() => (showForm = true)}>AGGIUNGI MANUALMENTE</Button>
    {#if $config?.sync_enabled}
      <Button disabled={fetching} onclick={fetchFromServer}>
        SCARICA DAL SERVER
      </Button>
    {/if}
  </div>

  <div class="hud" style="color: var(--fg-3)">
    Colonne: pettorale, nome, cognome, percorso, categoria (opz.) — intestazione opzionale
  </div>

  {#if $config?.sync_enabled && !$isAuthenticated}
    <div class="hud" style="color: var(--fg-3)">
      Per scaricare dal server serve il login (impostazioni → autenticazione)
    </div>
  {/if}

  {#if fetchStatus}
    <div class="hud" style="color: var(--fg-1)">{fetchStatus}</div>
  {/if}

  {#if importError}
    <div class="hud" style="color: var(--accent-finish)"><TriangleAlert size={14} /> {importError}</div>
  {/if}

  {#if summary}
    <div class="panel p-4">
      <div class="flex flex-wrap items-center gap-4">
        <span class="hud-strong" style="color: var(--accent-start)">
          <Check size={14} /> {summary.inserted} inseriti
        </span>
        <span class="hud-strong" style="color: var(--fg-1)">
          {summary.updated} aggiornati
        </span>
        {#if summary.courses_created > 0}
          <span class="hud-strong" style="color: var(--fg-1)">
            {summary.courses_created} percorsi creati
          </span>
        {/if}
        {#if summary.errors.length > 0}
          <span class="hud-strong" style="color: var(--accent-pending)">
            <TriangleAlert size={14} /> {summary.errors.length} righe scartate
          </span>
        {/if}
      </div>
      {#if summary.errors.length > 0}
        <ul class="mt-3 max-h-40 overflow-auto flex flex-col gap-1">
          {#each summary.errors as err (err.row + err.message)}
            <li class="hud" style="color: var(--accent-pending)">
              riga {err.row} — {err.message}
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

{#if showForm}
  <AthleteFormModal
    onClose={(saved) => {
      showForm = false;
      if (saved) refreshCourses();
    }}
  />
{/if}
