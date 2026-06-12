<script lang="ts">
  import { breakpoint } from '../breakpoint';
  import { syncStatus, config, layoutMode } from '../stores';
  import type { LayoutMode, NavView } from '../stores';
  import { cycleQuickToggle, resolvedTheme } from '../theme';
  import Button from '../ui/Button.svelte';
  import SegmentedControl from '../ui/SegmentedControl.svelte';

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

  let mobileDrawerOpen = $state(false);
  let isMobile = $derived($breakpoint === 'mobile');

  $effect(() => {
    if (!isMobile) {
      mobileDrawerOpen = false;
    }
  });

  function closeDrawer() {
    mobileDrawerOpen = false;
  }

  function navClick(id: NavView) {
    onNav(id);
    closeDrawer();
  }

  function duplicatesClick() {
    onDuplicates();
    closeDrawer();
  }
</script>

<header
  class="border-b"
  style="background: var(--bg-1); border-color: var(--line-2); box-shadow: var(--shadow-sm)"
>
  <div class="flex items-center gap-3 sm:gap-4 lg:gap-6 px-3 sm:px-4 lg:px-5 py-3">
    <!-- Brand -->
    <div class="flex items-center gap-3 shrink-0">
      <div class="hud-strong text-sm sm:text-base" style="color: var(--fg-0); letter-spacing: 0.08em">
        TRAIL<span style="color: var(--accent-running)">·</span>TRACE
      </div>
      <div class="hud hidden sm:block" style="color: var(--fg-3)">CHRONO v0.1</div>
    </div>

    <div class="flex-1"></div>

    {#if isMobile}
      <div class="flex items-center gap-2 shrink-0">
        <div class="op-chip" style="color: {operatorColor($config?.operator_id)}">
          {$config?.operator_id || 'NO-OP'}
        </div>

        <div class="flex items-center gap-2">
          <span
            class={$syncStatus.is_online ? 'dot-running' : 'dot-idle'}
            style={$syncStatus.is_online ? '' : 'background: var(--accent-pending);'}
          ></span>
          <div
            class="hud hidden sm:block"
            style="color: {$syncStatus.is_online ? 'var(--accent-running)' : 'var(--accent-pending)'}"
          >
            {$syncStatus.is_online ? 'ONLINE' : 'OFFLINE'}
          </div>
        </div>

        <Button
          variant="ghost"
          class="text-xs px-2 py-2"
          onclick={() => (mobileDrawerOpen = !mobileDrawerOpen)}
          ariaLabel="Menu"
          ariaExpanded={mobileDrawerOpen}
          title="Menu"
        >
          ☰
        </Button>
      </div>
    {:else}
      <!-- Nav -->
      <nav class="flex items-center gap-1 ml-1 lg:ml-4">
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
        <SegmentedControl
          ariaLabel="Layout"
          options={modes.map((m) => ({ value: m.id, label: m.icon, title: m.label }))}
          value={$layoutMode}
          onChange={(v) => layoutMode.set(v as LayoutMode)}
        />
      {/if}

      <!-- Status cluster -->
      <div class="flex items-center gap-2 lg:gap-4 shrink-0">
        <div class="op-chip" style="color: {operatorColor($config?.operator_id)}">
          {$config?.operator_id || 'NO-OP'}
        </div>

        <div class="flex items-center gap-2">
          <span
            class={$syncStatus.is_online ? 'dot-running' : 'dot-idle'}
            style={$syncStatus.is_online ? '' : 'background: var(--accent-pending);'}
          ></span>
          <div
            class="hud hidden lg:block"
            style="color: {$syncStatus.is_online ? 'var(--accent-running)' : 'var(--accent-pending)'}"
          >
            {$syncStatus.is_online ? 'ONLINE' : 'OFFLINE'}
          </div>
        </div>

        <div class="hud hidden lg:block">
          <span style="color: var(--fg-2)">QUEUE</span>
          <span class="num ml-1" style="color: var(--fg-0)">{$syncStatus.pending_count}</span>
        </div>

        <Button
          variant="ghost"
          size="sm"
          onclick={cycleQuickToggle}
          title={$resolvedTheme === 'dark' ? 'Passa a chiaro' : 'Passa a scuro'}
          ariaLabel="Cambia tema"
        >
          {$resolvedTheme === 'dark' ? '☀' : '☾'}
        </Button>

        <Button variant="ghost" size="sm" onclick={duplicatesClick} title="Duplicati">
          <span style="color: var(--accent-dup)">⚠</span> DUP
        </Button>
      </div>
    {/if}
  </div>

  {#if isMobile && mobileDrawerOpen}
    <div
      class="border-t px-3 py-3 flex flex-col gap-3"
      style="border-color: var(--line-2); background: var(--bg-1)"
    >
      <nav class="flex flex-wrap gap-1">
        {#each navItems as n (n.id)}
          <button
            class="nav-link"
            data-active={current === n.id}
            data-disabled={!n.enabled}
            disabled={!n.enabled}
            onclick={() => n.enabled && navClick(n.id)}
            title={n.enabled ? n.label : `${n.label} — prossimamente`}
          >
            {n.label}
          </button>
        {/each}
      </nav>

      {#if current === 'timing'}
        <SegmentedControl
          fullWidth
          ariaLabel="Layout"
          options={modes.map((m) => ({ value: m.id, label: m.icon, title: m.label }))}
          value={$layoutMode}
          onChange={(v) => layoutMode.set(v as LayoutMode)}
        />
      {/if}

      <div class="flex items-center gap-2 flex-wrap">
        <div class="hud">
          <span style="color: var(--fg-2)">QUEUE</span>
          <span class="num ml-1" style="color: var(--fg-0)">{$syncStatus.pending_count}</span>
        </div>

        <Button
          variant="ghost"
          size="sm"
          onclick={cycleQuickToggle}
          title={$resolvedTheme === 'dark' ? 'Passa a chiaro' : 'Passa a scuro'}
          ariaLabel="Cambia tema"
        >
          {$resolvedTheme === 'dark' ? '☀ Chiaro' : '☾ Scuro'}
        </Button>

        <Button variant="ghost" size="sm" onclick={duplicatesClick} title="Duplicati">
          <span style="color: var(--accent-dup)">⚠</span> DUP
        </Button>
      </div>
    </div>
  {/if}
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
