<script lang="ts">
  import { api } from '../api';
  import { save as saveDialog, open as openDialog } from '@tauri-apps/plugin-dialog';
  import Button from '../ui/Button.svelte';
  import ConfirmModal from './ConfirmModal.svelte';
  import { TriangleAlert, Check, Download, Upload } from 'lucide-svelte';
  import { t, i } from '../i18n';

  let busy = $state(false);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);
  let confirmRestore = $state<string | null>(null);

  function today(): string {
    return new Date().toISOString().slice(0, 10);
  }

  function reset() {
    message = null;
    error = null;
  }

  async function exportXlsx() {
    reset();
    const path = await saveDialog({
      defaultPath: `risultati_${today()}.xlsx`,
      filters: [{ name: 'Excel', extensions: ['xlsx'] }],
    });
    if (!path) return;
    busy = true;
    try {
      const s = await api.exportResultsXlsx(path);
      message = i($t.export.xlsxSuccess, { athletes: s.athletes_count, courses: s.courses_count });
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally { busy = false; }
  }

  async function exportCsv() {
    reset();
    const path = await saveDialog({
      defaultPath: `risultati_${today()}.csv`,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });
    if (!path) return;
    busy = true;
    try {
      const s = await api.exportResultsCsv(path);
      message = i($t.export.csvSuccess, { athletes: s.athletes_count, courses: s.courses_count });
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally { busy = false; }
  }

  async function backup() {
    reset();
    const path = await saveDialog({
      defaultPath: `mchrono_backup_${today()}.db`,
      filters: [{ name: 'Database', extensions: ['db'] }],
    });
    if (!path) return;
    busy = true;
    try {
      await api.backupDatabase(path);
      message = i($t.export.backupSuccess, { path });
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally { busy = false; }
  }

  async function pickRestore() {
    reset();
    const path = await openDialog({
      multiple: false,
      filters: [{ name: 'Database', extensions: ['db'] }],
    });
    if (!path || typeof path !== 'string') return;
    confirmRestore = path;
  }

  async function doRestore(path: string) {
    busy = true;
    try {
      await api.restoreDatabase(path);
      message = $t.export.restoreSuccess;
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally { busy = false; }
  }
</script>

<div class="p-6 max-w-3xl mx-auto">
  <div class="mb-6">
    <div class="hud" style="color: var(--fg-3)">{$t.export.sectionTitle}</div>
    <h2 class="hud-strong text-2xl mt-1" style="color: var(--fg-0)">{$t.export.pageTitle}</h2>
  </div>

  {#if message}
    <div class="panel p-3 mb-4 hud" style="color: var(--accent-start)"><Check size={14} /> {message}</div>
  {/if}
  {#if error}
    <div class="panel p-3 mb-4 hud" style="color: var(--accent-finish)"><TriangleAlert size={14} /> {error}</div>
  {/if}

  <section class="panel p-5 mb-4">
    <div class="hud mb-1">{$t.export.exportSectionTitle}</div>
    <p class="text-sm mb-4" style="color: var(--fg-2)">
      {$t.export.exportDescription}
    </p>
    <div class="flex flex-wrap gap-3">
      <Button variant="primary" disabled={busy} onclick={exportXlsx}>
        <Download size={14} /> {$t.export.xlsxButton}
      </Button>
      <Button disabled={busy} onclick={exportCsv}>
        <Download size={14} /> {$t.export.csvButton}
      </Button>
    </div>
  </section>

  <section class="panel p-5">
    <div class="hud mb-1">{$t.export.backupSectionTitle}</div>
    <p class="text-sm mb-4" style="color: var(--fg-2)">
      {@html $t.export.backupDescription}
    </p>
    <div class="flex flex-wrap gap-3">
      <Button variant="primary" disabled={busy} onclick={backup}>
        <Download size={14} /> {$t.export.backupButton}
      </Button>
      <Button disabled={busy} onclick={pickRestore}>
        <Upload size={14} /> {$t.export.restoreButton}
      </Button>
    </div>
  </section>
</div>

{#if confirmRestore}
  <ConfirmModal
    title={$t.export.restoreConfirmTitle}
    message={$t.export.restoreConfirmMessage}
    confirmLabel={$t.export.restoreConfirmButton}
    onCancel={() => (confirmRestore = null)}
    onConfirm={() => {
      const p = confirmRestore!;
      confirmRestore = null;
      doRestore(p);
    }}
  />
{/if}
