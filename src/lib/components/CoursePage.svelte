<script lang="ts">
  import { api } from '../api';
  import type { Course } from '../types';
  import CourseTimer from './CourseTimer.svelte';
  import FinishInput from './FinishInput.svelte';
  import PendingFinishList from './PendingFinishList.svelte';

  let { course, onBack }: { course: Course; onBack: () => void } = $props();
  let lastFinish = $state<string | null>(null);
  let busy = $state(false);

  async function start() {
    busy = true;
    try { await api.startCourse(course.id); }
    catch (e: any) { lastFinish = `errore start: ${e?.message ?? e}`; }
    finally { busy = false; }
  }

  async function capturePending() {
    try { await api.capturePending(course.id); }
    catch (e: any) { lastFinish = `errore: ${e?.message ?? e}`; }
  }
</script>

<div class="flex items-center justify-between mb-4">
  <button class="btn preset-tonal" onclick={onBack}>← Home</button>
  <h2 class="text-3xl font-bold">{course.name}</h2>
  <div></div>
</div>

<CourseTimer courseId={course.id} />

<div class="flex gap-4 my-4">
  {#if !course.started_at_ms}
    <button class="btn preset-filled text-2xl py-4 flex-1" disabled={busy} onclick={start}>
      START PERCORSO
    </button>
  {/if}
  <button class="btn preset-tonal text-2xl py-4 flex-1" onclick={capturePending}>
    Prossimo arrivato
  </button>
</div>

<FinishInput onFinish={(m) => lastFinish = m} />

<PendingFinishList courseId={course.id} />

{#if lastFinish}
  <p class="mt-4 text-2xl text-green-400">Ultimo: {lastFinish}</p>
{/if}
