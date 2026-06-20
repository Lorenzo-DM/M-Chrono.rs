<script lang="ts">
  import { onMount } from 'svelte';
  import type { PendingFinish, Athlete } from '../types';
  import { api } from '../api';
  import { formatMsToHms } from '../format';
  import { courses } from '../stores';
  import Button from '../ui/Button.svelte';
  import { TriangleAlert } from 'lucide-svelte';
  import BibCombobox from './BibCombobox.svelte';

  let { pending, onClose }: { pending: PendingFinish; onClose: () => void } = $props();

  let athletes = $state<Athlete[]>([]);
  let selected = $state<Athlete | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);

  let course = $derived($courses.find(c => c.id === pending.course_id));
  let elapsedMs = $derived(
    course?.started_at_ms != null
      ? pending.finish_timestamp_ms - course.started_at_ms
      : null
  );

  onMount(async () => {
    try {
      const rows = await api.getAthletesByCourse(pending.course_id);
      athletes = rows.map(r => r.athlete);
    } catch {}
  });

  async function assign() {
    if (!selected) return;
    error = null;
    busy = true;
    try {
      await api.assignPending(pending.id, selected.bib_number);
      onClose();
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally { busy = false; }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') { onClose(); return; }
    if (e.key === 'Enter' && selected && !busy) assign();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="modal-backdrop" role="dialog">
  <div class="panel-2 w-full max-w-lg">
    <div class="px-5 py-3 flex items-center justify-between border-b" style="border-color: var(--line-2)">
      <div class="hud-strong" style="color: var(--accent-pending)">ASSEGNA PETTORALE</div>
      <Button variant="ghost" size="sm" onclick={onClose}>ESC</Button>
    </div>

    <div class="p-6">
      <div class="hud mb-1">TIMESTAMP CATTURATO</div>
      <div class="chronodial num text-6xl mb-1" data-state="running">
        {formatMsToHms(pending.finish_timestamp_ms % 86_400_000)}
      </div>
      {#if elapsedMs !== null}
        <div class="num text-2xl mb-6" style="color: var(--fg-2)">
          +{formatMsToHms(elapsedMs)}
        </div>
      {:else}
        <div class="mb-6"></div>
      {/if}

      <div class="hud mb-2">PETTORALE / ATLETA</div>
      <BibCombobox
        {athletes}
        autofocus
        onSelect={(a) => { selected = a; error = null; }}
      />

      {#if error}
        <div class="hud mt-3" style="color: var(--accent-finish)"><TriangleAlert size={14} /> {error}</div>
      {/if}

      <div class="flex gap-2 mt-6">
        <Button variant="primary" class="flex-1 py-3" disabled={busy || !selected} onclick={assign}>
          ASSEGNA <span class="kbd ml-2" style="background:transparent; color:inherit; border-color:currentColor; opacity:0.6">↵</span>
        </Button>
        <Button class="flex-1 py-3" onclick={onClose}>ANNULLA</Button>
      </div>
    </div>
  </div>
</div>
