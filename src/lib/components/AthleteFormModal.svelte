<script lang="ts">
  import { untrack } from 'svelte';
  import type { Athlete } from '../types';
  import { api } from '../api';
  import { courses } from '../stores';
  import Button from '../ui/Button.svelte';
  import { TriangleAlert } from 'lucide-svelte';

  let { athlete = null, onClose }: {
    athlete?: Athlete | null;
    onClose: (saved: boolean) => void;
  } = $props();

  const NEW_COURSE = '__new__';

  let bib = $state(untrack(() => athlete ? String(athlete.bib_number) : ''));
  let firstName = $state(untrack(() => athlete?.first_name ?? ''));
  let lastName = $state(untrack(() => athlete?.last_name ?? ''));
  let category = $state(untrack(() => athlete?.category ?? ''));
  let courseSel = $state<string>(
    untrack(() => athlete ? String(athlete.course_id)
                          : ($courses[0] ? String($courses[0].id) : NEW_COURSE))
  );
  let newCourseName = $state('');
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function save() {
    error = null;
    const n = parseInt(bib);
    if (!Number.isFinite(n) || n <= 0) { error = 'pettorale non valido'; return; }
    if (!firstName.trim() && !lastName.trim()) { error = 'nome o cognome obbligatorio'; return; }
    const isNew = courseSel === NEW_COURSE;
    if (isNew && !newCourseName.trim()) { error = 'nome percorso obbligatorio'; return; }
    busy = true;
    try {
      await api.saveAthlete(athlete?.id ?? null, {
        bib_number: n,
        first_name: firstName.trim(),
        last_name: lastName.trim(),
        course_id: isNew ? null : Number(courseSel),
        course_name: isNew ? newCourseName.trim() : null,
        category: category.trim() || null,
        anonymous: athlete?.anonymous ?? false,
      });
      courses.set(await api.getCourses());
      onClose(true);
    } catch (e: any) {
      error = e?.message ?? JSON.stringify(e);
    } finally { busy = false; }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.target as HTMLElement)?.tagName !== 'SELECT') save();
    if (e.key === 'Escape') onClose(false);
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="modal-backdrop" role="dialog">
  <div class="panel-2 w-full max-w-lg">
    <div class="px-5 py-3 flex items-center justify-between border-b" style="border-color: var(--line-2)">
      <div class="hud-strong" style="color: var(--accent-running)">
        {athlete ? 'MODIFICA ATLETA' : 'NUOVO ATLETA'}
      </div>
      <Button variant="ghost" size="sm" onclick={() => onClose(false)}>ESC</Button>
    </div>

    <div class="p-6 flex flex-col gap-4">
      <label class="flex flex-col gap-1">
        <span class="hud">PETTORALE</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input bind:value={bib} type="number" inputmode="numeric"
               class="num text-2xl" autofocus autocomplete="off" placeholder="—" />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="hud">NOME</span>
          <input bind:value={firstName} autocomplete="off" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="hud">COGNOME</span>
          <input bind:value={lastName} autocomplete="off" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="hud">CATEGORIA (opzionale)</span>
        <input bind:value={category} autocomplete="off" placeholder="es. M40, SF, U23" />
      </label>

      <label class="flex flex-col gap-1">
        <span class="hud">PERCORSO</span>
        <select bind:value={courseSel}>
          {#each $courses as c (c.id)}
            <option value={String(c.id)}>{c.name}</option>
          {/each}
          <option value={NEW_COURSE}>nuovo percorso…</option>
        </select>
      </label>

      {#if courseSel === NEW_COURSE}
        <label class="flex flex-col gap-1">
          <span class="hud">NOME NUOVO PERCORSO</span>
          <input bind:value={newCourseName} placeholder="es. 21K" autocomplete="off" />
        </label>
      {/if}

      {#if error}
        <div class="hud" style="color: var(--accent-finish)"><TriangleAlert size={14} /> {error}</div>
      {/if}

      <div class="flex gap-2 mt-2">
        <Button variant="primary" class="flex-1 py-3" disabled={busy} onclick={save}>
          {athlete ? 'SALVA' : 'AGGIUNGI'}
        </Button>
        <Button class="flex-1 py-3" onclick={() => onClose(false)}>ANNULLA</Button>
      </div>
    </div>
  </div>
</div>
