<script lang="ts">
  import type { Course } from '../types';
  import Button from '../ui/Button.svelte';
  import { TriangleAlert } from 'lucide-svelte';

  type Variant = 'end' | 'restart';

  let {
    course,
    variant,
    onClose,
    onConfirm,
  }: {
    course: Course;
    variant: Variant;
    onClose: () => void;
    onConfirm: (typed: string) => Promise<void>;
  } = $props();

  let typed = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);

  let matches = $derived(typed.trim() === course.name);

  const COPY = {
    end: {
      title: 'TERMINA GARA',
      titleColor: 'var(--accent-finish)',
      lead: 'Stai per <strong>terminare definitivamente</strong> il percorso:',
      body:
        'Dopo la conferma il timer si ferma e non sarà più possibile registrare nuovi arrivi su questo percorso. Operazione irreversibile.',
      cta: 'TERMINA GARA',
      busy: 'TERMINANDO…',
      ctaBg: 'var(--accent-finish)',
    },
    restart: {
      title: '↻ RIAVVIA GARA',
      titleColor: 'var(--accent-pending)',
      lead: 'Stai per <strong>azzerare il cronometro</strong> del percorso:',
      body:
        'Tutti i tempi registrati per questo percorso (arrivi e pending) verranno eliminati. Il timer torna a 00:00:00 e potrai dare un nuovo START. Operazione irreversibile.',
      cta: 'RIAVVIA GARA',
      busy: 'RIAVVIANDO…',
      ctaBg: 'var(--accent-pending)',
    },
  } as const;
  let copy = $derived(COPY[variant]);

  async function confirm() {
    if (!matches || busy) return;
    busy = true;
    error = null;
    try {
      await onConfirm(typed.trim());
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
      <div class="hud-strong" style="color: {copy.titleColor}">
        {copy.title}
      </div>
      <Button variant="ghost" size="sm" onclick={onClose}>ESC</Button>
    </div>

    <div class="p-6">
      <p class="text-sm mb-1" style="color: var(--fg-1)">
        {@html copy.lead}
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
        {copy.body}
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
        <div class="hud mt-3" style="color: var(--accent-finish)"><TriangleAlert size={14} /> {error}</div>
      {/if}

      <div class="flex gap-2 mt-6">
        <Button class="flex-1 py-3" onclick={onClose}>ANNULLA</Button>
        <Button
          class="flex-1 py-3"
          style={matches
            ? `background: ${copy.ctaBg}; border-color: ${copy.ctaBg}; color: #f6f2e9`
            : ''}
          disabled={!matches || busy}
          onclick={confirm}
        >
          {busy ? copy.busy : copy.cta}
        </Button>
      </div>
    </div>
  </div>
</div>
