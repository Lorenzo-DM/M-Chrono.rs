<script lang="ts">
  import { api } from '../api';
  import type { Course } from '../types';

  let {
    course,
    onClose,
    onDone,
  }: {
    course: Course;
    onClose: () => void;
    onDone?: () => void;
  } = $props();

  let typed = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);

  let matches = $derived(typed.trim() === course.name);

  async function confirm() {
    if (!matches || busy) return;
    busy = true;
    error = null;
    try {
      await api.endCourse(course.id, typed.trim());
      onDone?.();
      onClose();
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally {
      busy = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
    if (e.key === 'Enter' && matches && !busy) confirm();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="modal-backdrop" role="dialog" aria-modal="true">
  <div class="panel w-full max-w-md" style="overflow: hidden">
    <div
      class="px-5 py-3 flex items-center justify-between border-b"
      style="border-color: var(--line-2); background: var(--bg-2)"
    >
      <div class="hud-strong" style="color: var(--accent-finish)">
        ⚠ TERMINA GARA
      </div>
      <button class="btn-base btn-ghost text-xs" onclick={onClose}>ESC</button>
    </div>

    <div class="p-6">
      <p class="text-sm mb-1" style="color: var(--fg-1)">
        Stai per <strong>terminare definitivamente</strong> il percorso:
      </p>
      <div
        class="my-3 px-3 py-2 rounded-md"
        style="background: var(--bg-2); border: 1px solid var(--line-2)"
      >
        <div class="hud" style="color: var(--fg-2)">PERCORSO</div>
        <div
          class="text-xl font-semibold mt-0.5"
          style="color: var(--fg-0); letter-spacing: -0.01em"
        >
          {course.name}
        </div>
      </div>
      <p class="text-sm mb-4" style="color: var(--fg-2)">
        Dopo la conferma il timer si ferma e non sarà più possibile registrare
        nuovi arrivi su questo percorso. Operazione irreversibile.
      </p>

      <div class="hud mb-2">DIGITA IL NOME DEL PERCORSO PER CONFERMARE</div>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        bind:value={typed}
        type="text"
        placeholder={course.name}
        class="w-full"
        autofocus
        autocomplete="off"
        spellcheck="false"
      />

      {#if error}
        <div class="hud mt-3" style="color: var(--accent-finish)">⚠ {error}</div>
      {/if}

      <div class="flex gap-2 mt-6">
        <button class="btn-base flex-1 py-3" onclick={onClose}>ANNULLA</button>
        <button
          class="btn-base flex-1 py-3"
          style={matches
            ? 'background: var(--accent-finish); border-color: var(--accent-finish); color: #f6f2e9'
            : ''}
          disabled={!matches || busy}
          onclick={confirm}
        >
          {busy ? 'TERMINANDO…' : 'TERMINA GARA'}
        </button>
      </div>
    </div>
  </div>
</div>
