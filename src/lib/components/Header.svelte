<script lang="ts">
  import { syncStatus, config } from '../stores';

  let { onSettings } = $props<{ onSettings: () => void }>();
</script>

<header class="flex items-center justify-between px-6 py-3 border-b border-white/20">
  <h1 class="text-2xl font-bold">Trail Race Timing</h1>
  <div class="flex items-center gap-4">
    <span class="text-sm opacity-70">{$config?.operator_id || 'no-operator'}</span>
    <span class="text-sm">
      {#if $syncStatus.is_online}
        <span class="text-green-400">● online</span>
      {:else}
        <span class="text-yellow-400">● offline</span>
      {/if}
      — {$syncStatus.pending_count} pending
    </span>
    <button class="btn preset-tonal" onclick={onSettings}>Settings</button>
  </div>
</header>
