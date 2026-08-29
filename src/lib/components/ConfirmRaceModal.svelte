<script lang="ts">
  import type { Course } from '../types';
  import Button from '../ui/Button.svelte';
  import { TriangleAlert } from 'lucide-svelte';
  import { t } from '../i18n';

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

  let copy = $derived(variant === 'end' ? {
    title: $t.modals.confirmRace.endTitle,
    titleColor: 'var(--accent-finish)',
    lead: $t.modals.confirmRace.endLead,
    body: $t.modals.confirmRace.endBody,
    cta: $t.modals.confirmRace.endCta,
    busy: $t.modals.confirmRace.endBusy,
    ctaBg: 'var(--accent-finish)',
  } : {
    title: $t.modals.confirmRace.restartTitle,
    titleColor: 'var(--accent-pending)',
    lead: $t.modals.confirmRace.restartLead,
    body: $t.modals.confirmRace.restartBody,
    cta: $t.modals.confirmRace.restartCta,
    busy: $t.modals.confirmRace.restartBusy,
    ctaBg: 'var(--accent-pending)',
  });

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
        <div class="hud" style="color: var(--fg-2)">{$t.modals.confirmRace.courseLabel}</div>
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

      <div class="hud mb-2">{$t.modals.confirmRace.typeToConfirm}</div>
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
        <Button class="flex-1 py-3" onclick={onClose}>{$t.common.cancel}</Button>
        <Button
          class="flex-1 py-3"
          style={matches
            ? `background: ${copy.ctaBg}; border-color: ${copy.ctaBg}; color: var(--on-accent)`
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
