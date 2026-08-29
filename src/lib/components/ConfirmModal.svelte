<script lang="ts">
  import Button from '../ui/Button.svelte';
  import { t } from '../i18n';

  let {
    message,
    title,
    confirmLabel,
    danger = true,
    onCancel,
    onConfirm,
  }: {
    message: string;
    title?: string;
    confirmLabel?: string;
    danger?: boolean;
    onCancel: () => void;
    onConfirm: () => void;
  } = $props();

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onCancel();
    if (e.key === 'Enter') onConfirm();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="modal-backdrop" role="dialog" aria-modal="true">
  <div class="panel w-full max-w-sm" style="overflow: hidden">
    <div
      class="px-5 py-3 flex items-center justify-between border-b"
      style="border-color: var(--line-2); background: var(--bg-2)"
    >
      <div class="hud-strong" style="color: {danger ? 'var(--accent-finish)' : 'var(--fg-0)'}">
        {title ?? $t.modals.confirm.title}
      </div>
      <Button variant="ghost" size="sm" onclick={onCancel}>ESC</Button>
    </div>

    <div class="p-6">
      <p class="text-sm" style="color: var(--fg-1)">{message}</p>
      <div class="flex gap-2 mt-6">
        <Button class="flex-1 py-3" onclick={onCancel}>{$t.common.cancel}</Button>
        <Button
          class="flex-1 py-3"
          style={danger
            ? 'background: var(--accent-finish); border-color: var(--accent-finish); color: var(--on-accent)'
            : ''}
          onclick={onConfirm}
        >
          {confirmLabel ?? $t.common.confirm}
        </Button>
      </div>
    </div>
  </div>
</div>
