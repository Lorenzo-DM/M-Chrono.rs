<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { courses } from '../stores';
  import type { Checkpoint } from '../types';
  import Button from '../ui/Button.svelte';
  import { TriangleAlert, X } from 'lucide-svelte';

  let checkpoints = $state<Checkpoint[]>([]);
  let selectedCourseId = $state<number | null>(null);
  let newName = $state('');
  let error = $state<string | null>(null);
  let busy = $state(false);

  $effect(() => {
    if (selectedCourseId == null) selectedCourseId = $courses[0]?.id ?? null;
  });

  onMount(load);

  async function load() {
    try {
      checkpoints = await api.getCheckpoints();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  let courseCheckpoints = $derived(
    checkpoints
      .filter((c) => c.course_id === selectedCourseId)
      .sort((a, b) => a.position - b.position),
  );

  async function add() {
    error = null;
    if (selectedCourseId == null) { error = 'seleziona un percorso'; return; }
    if (!newName.trim()) { error = 'nome checkpoint obbligatorio'; return; }
    busy = true;
    try {
      const position = courseCheckpoints.length + 1;
      await api.saveCheckpoint(null, {
        course_id: selectedCourseId,
        name: newName.trim(),
        position,
      });
      newName = '';
      await load();
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally { busy = false; }
  }

  async function remove(id: number) {
    error = null;
    try {
      await api.deleteCheckpoint(id);
      await load();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }
</script>

<div class="flex flex-col gap-3">
  <p class="text-sm" style="color: var(--fg-2)">
    Punti di passaggio intermedi. Durante la gara registra i transiti per pettorale
    dalla scheda del percorso; i tempi parziali finiscono nell'export.
  </p>

  <div class="flex items-end gap-2">
    <label class="flex flex-col gap-1">
      <span class="hud">PERCORSO</span>
      <select bind:value={selectedCourseId}>
        {#each $courses as c (c.id)}
          <option value={c.id}>{c.name}</option>
        {/each}
      </select>
    </label>
    <label class="flex flex-col gap-1 flex-1 max-w-xs">
      <span class="hud">NUOVO CHECKPOINT</span>
      <input
        bind:value={newName}
        placeholder="es. KM 10, Rifugio"
        autocomplete="off"
        onkeydown={(e) => e.key === 'Enter' && add()}
      />
    </label>
    <Button variant="primary" disabled={busy} onclick={add}>+ AGGIUNGI</Button>
  </div>

  {#if courseCheckpoints.length > 0}
    <ul class="flex flex-col gap-1">
      {#each courseCheckpoints as cp (cp.id)}
        <li class="flex items-center justify-between px-3 py-2 rounded"
            style="background: var(--bg-2)">
          <span style="color: var(--fg-0)">
            <span class="num" style="color: var(--fg-3)">{cp.position}.</span> {cp.name}
          </span>
          <Button variant="ghost" size="sm" onclick={() => remove(cp.id)} title="Rimuovi"><X size={14} /></Button>
        </li>
      {/each}
    </ul>
  {:else}
    <div class="hud" style="color: var(--fg-3)">Nessun checkpoint per questo percorso.</div>
  {/if}

  {#if error}
    <div class="hud" style="color: var(--accent-finish)"><TriangleAlert size={14} /> {error}</div>
  {/if}
</div>
