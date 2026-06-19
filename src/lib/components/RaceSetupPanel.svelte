<script lang="ts">
  import { api } from '../api';
  import { courses } from '../stores';
  import type { Race } from '../types';
  import Button from '../ui/Button.svelte';

  let { onChange }: { onChange?: () => void } = $props();

  let races = $state<Race[]>([]);
  let selectedRaceId = $state<number | null>(null);
  let mode = $state<'select' | 'new'>('new');
  let newRaceName = $state('');
  let newRaceDate = $state(''); // yyyy-mm-dd
  let newCourseName = $state('');
  let newCourseKm = $state('');
  let error = $state<string | null>(null);
  let busy = $state(false);

  $effect(() => {
    load();
  });

  async function load() {
    races = await api.getRaces();
    if (selectedRaceId == null && races.length > 0) {
      selectedRaceId = races[races.length - 1].id;
      mode = 'select';
    }
    if (races.length === 0) mode = 'new';
  }

  let raceCourses = $derived(
    $courses.filter((c) => c.race_id === selectedRaceId),
  );

  function fmtDate(ms: number | null): string {
    if (ms == null) return '';
    return new Date(ms).toLocaleDateString();
  }

  async function createRace() {
    error = null;
    if (!newRaceName.trim()) { error = 'nome gara obbligatorio'; return; }
    busy = true;
    try {
      const ms = newRaceDate ? new Date(newRaceDate).getTime() : null;
      const r = await api.saveRace(null, { name: newRaceName.trim(), scheduled_at_ms: ms });
      newRaceName = '';
      newRaceDate = '';
      await load();
      selectedRaceId = r.id;
      mode = 'select';
      onChange?.();
    } catch (e: any) {
      error = e?.message ?? JSON.stringify(e);
    } finally { busy = false; }
  }

  async function addCourse() {
    error = null;
    if (selectedRaceId == null) return;
    if (!newCourseName.trim()) { error = 'nome percorso obbligatorio'; return; }
    busy = true;
    try {
      const km = parseFloat(newCourseKm.replace(',', '.'));
      const distance_m = Number.isFinite(km) && km > 0 ? Math.round(km * 1000) : null;
      await api.saveCourse(null, { name: newCourseName.trim(), race_id: selectedRaceId, distance_m });
      newCourseName = '';
      newCourseKm = '';
      courses.set(await api.getCourses());
      onChange?.();
    } catch (e: any) {
      error = e?.message ?? JSON.stringify(e);
    } finally { busy = false; }
  }

  async function updateDistance(c: { id: number; name: string; race_id: number | null }, km: string) {
    const v = parseFloat(km.replace(',', '.'));
    const distance_m = Number.isFinite(v) && v > 0 ? Math.round(v * 1000) : null;
    try {
      await api.saveCourse(c.id, { name: c.name, race_id: c.race_id, distance_m });
      courses.set(await api.getCourses());
      onChange?.();
    } catch (e: any) {
      error = e?.message ?? JSON.stringify(e);
    }
  }

  async function removeCourse(id: number) {
    error = null;
    try {
      await api.deleteCourse(id);
      courses.set(await api.getCourses());
      onChange?.();
    } catch (e: any) {
      error = e?.message ?? JSON.stringify(e);
    }
  }

  async function removeRace(id: number) {
    error = null;
    if (!confirm('Eliminare la gara? I percorsi restano ma senza gara.')) return;
    try {
      await api.deleteRace(id);
      selectedRaceId = null;
      await load();
      courses.set(await api.getCourses());
      onChange?.();
    } catch (e: any) {
      error = e?.message ?? JSON.stringify(e);
    }
  }
</script>

<div class="flex flex-col gap-4">
  {#if races.length > 0}
    <div class="flex flex-wrap items-end gap-3">
      <label class="flex flex-col gap-1">
        <span class="hud">GARA</span>
        <select bind:value={selectedRaceId} onchange={() => (mode = 'select')}>
          {#each races as r (r.id)}
            <option value={r.id}>{r.name}{r.scheduled_at_ms ? ` · ${fmtDate(r.scheduled_at_ms)}` : ''}</option>
          {/each}
        </select>
      </label>
      <Button onclick={() => { mode = 'new'; selectedRaceId = null; }}>+ NUOVA GARA</Button>
      {#if mode === 'select' && selectedRaceId != null}
        <Button variant="ghost" onclick={() => removeRace(selectedRaceId!)} title="Elimina gara">ELIMINA GARA</Button>
      {/if}
    </div>
  {/if}

  {#if mode === 'new'}
    <div class="panel p-4 flex flex-col gap-3">
      <div class="hud">NUOVA GARA</div>
      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="hud">NOME GARA</span>
          <input bind:value={newRaceName} placeholder="es. Trail del Monte" autocomplete="off" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="hud">DATA (opzionale)</span>
          <input type="date" bind:value={newRaceDate} />
        </label>
      </div>
      <div>
        <Button variant="primary" disabled={busy} onclick={createRace}>CREA GARA</Button>
      </div>
    </div>
  {/if}

  {#if mode === 'select' && selectedRaceId != null}
    <div class="panel p-4 flex flex-col gap-3">
      <div class="hud">PERCORSI</div>
      <div class="flex items-end gap-2">
        <label class="flex flex-col gap-1 flex-1 max-w-sm">
          <span class="hud">NOME PERCORSO</span>
          <input
            bind:value={newCourseName}
            placeholder="es. 21K"
            autocomplete="off"
            onkeydown={(e) => e.key === 'Enter' && addCourse()}
          />
        </label>
        <label class="flex flex-col gap-1 w-28">
          <span class="hud">KM (opz.)</span>
          <input
            bind:value={newCourseKm}
            type="number"
            inputmode="decimal"
            min="0"
            step="0.1"
            placeholder="21"
            class="num"
            onkeydown={(e) => e.key === 'Enter' && addCourse()}
          />
        </label>
        <Button variant="primary" disabled={busy} onclick={addCourse}>+ AGGIUNGI</Button>
      </div>

      {#if raceCourses.length > 0}
        <ul class="flex flex-col gap-1">
          {#each raceCourses as c (c.id)}
            <li class="flex items-center justify-between px-3 py-2 rounded gap-3"
                style="background: var(--bg-2)">
              <span style="color: var(--fg-0)" class="flex-1">{c.name}</span>
              <label class="flex items-center gap-1">
                <input
                  type="number"
                  inputmode="decimal"
                  min="0"
                  step="0.1"
                  class="num w-20"
                  value={c.distance_m != null ? (c.distance_m / 1000).toString() : ''}
                  placeholder="km"
                  onchange={(e) => updateDistance(c, (e.target as HTMLInputElement).value)}
                />
                <span class="hud" style="color: var(--fg-3)">KM</span>
              </label>
              <Button variant="ghost" size="sm" onclick={() => removeCourse(c.id)} title="Rimuovi">✕</Button>
            </li>
          {/each}
        </ul>
      {:else}
        <div class="hud" style="color: var(--fg-3)">
          Nessun percorso. Aggiungine almeno uno (es. 21K, 10K).
        </div>
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="hud" style="color: var(--accent-finish)">⚠ {error}</div>
  {/if}
</div>
