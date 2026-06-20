<script lang="ts">
  import { api } from '../api';
  import { config, courses, isAuthenticated } from '../stores';
  import type { ImportSummary } from '../types';
  import Button from '../ui/Button.svelte';
  import { TriangleAlert, Check } from 'lucide-svelte';
  import AthleteFormModal from './AthleteFormModal.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { t, i } from '../i18n';

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
      fetchStatus = `${$t.common.error} · ${e?.message ?? JSON.stringify(e)}`;
    } finally { fetching = false; }
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center gap-3">
    <Button variant="primary" disabled={importing} onclick={importFile}>
      {importing ? $t.athletes.importing : $t.athletes.importButton}
    </Button>
    <Button onclick={() => (showForm = true)}>{$t.athletes.addManualButton}</Button>
    {#if $config?.sync_enabled}
      <Button disabled={fetching} onclick={fetchFromServer}>
        {$t.athletes.fetchFromServer}
      </Button>
    {/if}
  </div>

  <div class="hud" style="color: var(--fg-3)">
    {$t.athletes.columnHint}
  </div>

  {#if $config?.sync_enabled && !$isAuthenticated}
    <div class="hud" style="color: var(--fg-3)">
      {$t.athletes.syncLoginHint}
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
          <Check size={14} /> {i($t.athletes.insertedCount, { n: summary.inserted })}
        </span>
        <span class="hud-strong" style="color: var(--fg-1)">
          {i($t.athletes.updatedCount, { n: summary.updated })}
        </span>
        {#if summary.courses_created > 0}
          <span class="hud-strong" style="color: var(--fg-1)">
            {i($t.athletes.coursesCreated, { n: summary.courses_created })}
          </span>
        {/if}
        {#if summary.errors.length > 0}
          <span class="hud-strong" style="color: var(--accent-pending)">
            <TriangleAlert size={14} /> {i($t.athletes.rowsDiscarded, { n: summary.errors.length })}
          </span>
        {/if}
      </div>
      {#if summary.errors.length > 0}
        <ul class="mt-3 max-h-40 overflow-auto flex flex-col gap-1">
          {#each summary.errors as err (err.row + err.message)}
            <li class="hud" style="color: var(--accent-pending)">
              {i($t.athletes.rowError, { row: err.row, message: err.message })}
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
