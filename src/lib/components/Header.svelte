<script lang="ts">
  import { syncStatus, config, layoutMode } from '../stores';
  import type { LayoutMode } from '../stores';

  let { onSettings, onDuplicates } = $props<{
    onSettings: () => void;
    onDuplicates: () => void;
  }>();

  const modes: { id: LayoutMode; label: string; icon: string }[] = [
    { id: 'tabs',  label: 'Tabs',  icon: '▭' },
    { id: 'split', label: 'Split', icon: '▭▭' },
    { id: 'grid',  label: 'Grid',  icon: '▦' },
  ];

  function operatorColor(id?: string) {
    if (!id) return 'var(--fg-3)';
    const last = id.trim().slice(-1).toUpperCase();
    if (last === 'A') return 'var(--op-a)';
    if (last === 'B') return 'var(--op-b)';
    if (last === 'C') return 'var(--op-c)';
    if (last === 'D') return 'var(--op-d)';
    return 'var(--accent-running)';
  }
</script>

<header class="relative panel-2 border-b" style="border-color: var(--line-2)">
  <div class="flex items-center gap-6 px-5 py-3">
    <!-- Brand -->
    <div class="flex items-center gap-3">
      <div class="hud-strong text-base" style="color: var(--fg-0)">
        TRAIL<span style="color: var(--accent-running)">·</span>TRACE
      </div>
      <div class="hud" style="color: var(--fg-3)">CHRONO v0.1</div>
    </div>

    <div class="flex-1"></div>

    <!-- Layout switcher -->
    <div class="flex items-center gap-3">
      <div class="hud">Layout</div>
      <div class="seg">
        {#each modes as m (m.id)}
          <button
            class="seg-item"
            data-active={$layoutMode === m.id}
            onclick={() => layoutMode.set(m.id)}
            aria-label={m.label}
          >
            <span class="num">{m.icon}</span>
            <span class="ml-2">{m.label}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="flex-1"></div>

    <!-- Status cluster -->
    <div class="flex items-center gap-4">
      <div class="op-chip" style="color: {operatorColor($config?.operator_id)}">
        {$config?.operator_id || 'NO-OP'}
      </div>
      <div class="flex items-center gap-2">
        <span class={$syncStatus.is_online ? 'dot-running' : 'dot-idle'}
              style={$syncStatus.is_online ? '' : 'background: var(--accent-pending); box-shadow: 0 0 8px var(--accent-pending);'}></span>
        <div class="hud" style="color: {$syncStatus.is_online ? 'var(--accent-running)' : 'var(--accent-pending)'}">
          {$syncStatus.is_online ? 'ONLINE' : 'OFFLINE'}
        </div>
      </div>
      <div class="hud">
        <span style="color: var(--fg-2)">QUEUE</span>
        <span class="num ml-1" style="color: var(--fg-0)">{$syncStatus.pending_count}</span>
      </div>

      <button class="btn-base btn-ghost" onclick={onDuplicates} title="Duplicati">
        <span style="color: var(--accent-dup)">⚠</span> DUP
      </button>
      <button class="btn-base" onclick={onSettings}>SETTINGS</button>
    </div>
  </div>
</header>
