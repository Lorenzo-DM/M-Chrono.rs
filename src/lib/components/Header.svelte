<script lang="ts">
  import { syncStatus, config, layoutMode } from '../stores';
  import type { LayoutMode, NavView } from '../stores';

  let {
    current = 'timing',
    onNav,
    onDuplicates,
  } = $props<{
    current?: NavView;
    onNav: (v: NavView) => void;
    onDuplicates: () => void;
  }>();

  const navItems: { id: NavView; label: string; enabled: boolean }[] = [
    { id: 'timing',   label: 'Timing',   enabled: true  },
    { id: 'results',  label: 'Results',  enabled: false },
    { id: 'settings', label: 'Settings', enabled: true  },
    { id: 'export',   label: 'Export',   enabled: false },
  ];

  const modes: { id: LayoutMode; label: string; icon: string }[] = [
    { id: 'tabs',  label: 'Tabs',  icon: '▭'  },
    { id: 'split', label: 'Split', icon: '▭▭' },
    { id: 'grid',  label: 'Grid',  icon: '▦'  },
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

<header class="border-b" style="background: var(--bg-1); border-color: var(--line-2); box-shadow: var(--shadow-sm)">
  <div class="flex items-center gap-6 px-5 py-3">
    <!-- Brand -->
    <div class="flex items-center gap-3 shrink-0">
      <div class="hud-strong text-base" style="color: var(--fg-0); letter-spacing: 0.08em">
        TRAIL<span style="color: var(--accent-running)">·</span>TRACE
      </div>
      <div class="hud" style="color: var(--fg-3)">CHRONO v0.1</div>
    </div>

    <!-- Nav -->
    <nav class="flex items-center gap-1 ml-4">
      {#each navItems as n (n.id)}
        <button
          class="nav-link"
          data-active={current === n.id}
          data-disabled={!n.enabled}
          disabled={!n.enabled}
          onclick={() => n.enabled && onNav(n.id)}
          title={n.enabled ? n.label : `${n.label} — prossimamente`}
        >
          {n.label}
        </button>
      {/each}
    </nav>

    <div class="flex-1"></div>

    <!-- Layout switcher (timing view only) -->
    {#if current === 'timing'}
      <div class="seg" title="Layout">
        {#each modes as m (m.id)}
          <button
            class="seg-item"
            data-active={$layoutMode === m.id}
            onclick={() => layoutMode.set(m.id)}
            aria-label={m.label}
            title={m.label}
          >
            <span class="num">{m.icon}</span>
          </button>
        {/each}
      </div>
    {/if}

    <!-- Status cluster -->
    <div class="flex items-center gap-4 shrink-0">
      <div class="op-chip" style="color: {operatorColor($config?.operator_id)}">
        {$config?.operator_id || 'NO-OP'}
      </div>

      <div class="flex items-center gap-2">
        <span
          class={$syncStatus.is_online ? 'dot-running' : 'dot-idle'}
          style={$syncStatus.is_online
            ? ''
            : 'background: var(--accent-pending);'}
        ></span>
        <div
          class="hud"
          style="color: {$syncStatus.is_online ? 'var(--accent-running)' : 'var(--accent-pending)'}"
        >
          {$syncStatus.is_online ? 'ONLINE' : 'OFFLINE'}
        </div>
      </div>

      <div class="hud">
        <span style="color: var(--fg-2)">QUEUE</span>
        <span class="num ml-1" style="color: var(--fg-0)">{$syncStatus.pending_count}</span>
      </div>

      <button class="btn-base btn-ghost text-xs" onclick={onDuplicates} title="Duplicati">
        <span style="color: var(--accent-dup)">⚠</span> DUP
      </button>
    </div>
  </div>
</header>

<style>
  .nav-link {
    padding: 0.4rem 0.85rem;
    font-size: 0.82rem;
    font-weight: 500;
    letter-spacing: 0.04em;
    color: var(--fg-2);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color 100ms, border-color 100ms, background 100ms;
  }
  .nav-link:hover:not(:disabled) {
    color: var(--fg-0);
    background: var(--bg-3);
  }
  .nav-link[data-active='true'] {
    color: var(--fg-0);
    border-bottom-color: var(--accent-running);
  }
  .nav-link[data-disabled='true'] {
    color: var(--fg-3);
    cursor: not-allowed;
    opacity: 0.55;
  }
</style>
