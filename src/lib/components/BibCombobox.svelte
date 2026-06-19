<script lang="ts">
  import type { Athlete } from '../types';

  let {
    athletes,
    onSelect,
    onFreeEntry,
    autofocus = false,
    placeholder = 'cerca per numero o nome…',
    compact = false,
  }: {
    athletes: Athlete[];
    onSelect: (a: Athlete | null) => void;
    onFreeEntry?: (bib: number) => void;
    autofocus?: boolean;
    placeholder?: string;
    compact?: boolean;
  } = $props();

  let search = $state('');
  let dropdownOpen = $state(false);
  let highlightIdx = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLUListElement | null>(null);
  let dropPos = $state({ top: 0, left: 0, width: 0 });

  let filtered = $derived(
    athletes.filter(a => {
      const q = search.toLowerCase().trim();
      if (!q) return true;
      return (
        String(a.bib_number).includes(q) ||
        a.first_name.toLowerCase().includes(q) ||
        a.last_name.toLowerCase().includes(q)
      );
    })
  );

  let freeBib = $derived.by<number | null>(() => {
    if (!onFreeEntry) return null;
    const q = search.trim();
    const n = parseInt(q);
    if (!Number.isFinite(n) || n <= 0) return null;
    const exact = athletes.find(a => a.bib_number === n);
    return exact ? null : n;
  });

  $effect(() => {
    if (highlightIdx >= filtered.length) highlightIdx = Math.max(0, filtered.length - 1);
  });

  $effect(() => {
    void highlightIdx;
    if (!dropdownOpen || !listEl) return;
    requestAnimationFrame(() => {
      listEl?.querySelector<HTMLElement>('.highlighted')?.scrollIntoView({ block: 'nearest' });
    });
  });

  // position: fixed dropdown tracks input as the scroll container scrolls
  $effect(() => {
    if (!dropdownOpen || !inputEl) return;

    function updatePos() {
      if (!inputEl) return;
      const r = inputEl.getBoundingClientRect();
      dropPos = { top: r.bottom + 4, left: r.left, width: r.width };
    }

    updatePos();

    let scroller: Element | null = inputEl.parentElement;
    while (scroller && scroller !== document.body) {
      const s = getComputedStyle(scroller);
      if (s.overflow === 'auto' || s.overflow === 'scroll' ||
          s.overflowY === 'auto' || s.overflowY === 'scroll') {
        scroller.addEventListener('scroll', updatePos, { passive: true });
        break;
      }
      scroller = scroller.parentElement;
    }
    window.addEventListener('resize', updatePos, { passive: true });

    return () => {
      scroller?.removeEventListener('scroll', updatePos);
      window.removeEventListener('resize', updatePos);
    };
  });

  function selectAthlete(a: Athlete) {
    search = `${a.bib_number} – ${a.first_name} ${a.last_name}`;
    dropdownOpen = false;
    onSelect(a);
  }

  function onInputKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (dropdownOpen) { dropdownOpen = false; e.stopPropagation(); }
      return;
    }
    if (!dropdownOpen) return;
    const totalItems = filtered.length + (freeBib !== null ? 1 : 0);
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightIdx = Math.min(highlightIdx + 1, totalItems - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightIdx = Math.max(highlightIdx - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (freeBib !== null && highlightIdx === filtered.length) {
        onFreeEntry!(freeBib);
        search = '';
        dropdownOpen = false;
      } else if (filtered[highlightIdx]) {
        selectAthlete(filtered[highlightIdx]);
      }
    }
  }
</script>

<div class="bib-combobox" class:compact>
  <!-- svelte-ignore a11y_autofocus -->
  <input
    bind:this={inputEl}
    bind:value={search}
    type="text"
    {autofocus}
    autocomplete="off"
    {placeholder}
    oninput={() => { onSelect(null); dropdownOpen = true; highlightIdx = 0; }}
    onfocus={() => {
      if (inputEl) {
        const r = inputEl.getBoundingClientRect();
        dropPos = { top: r.bottom + 4, left: r.left, width: r.width };
      }
      dropdownOpen = true;
    }}
    onblur={() => { dropdownOpen = false; }}
    onkeydown={onInputKeydown}
  />

  {#if dropdownOpen && (filtered.length > 0 || freeBib !== null)}
    <ul
      class="bib-dropdown panel-2"
      bind:this={listEl}
      style="top: {dropPos.top}px; left: {dropPos.left}px; width: {dropPos.width}px;"
    >
      {#each filtered as a, i (a.id)}
        {@const anonAthlete = a.anonymous || (!a.first_name && !a.last_name)}
        <li>
          <button
            class="drop-item"
            class:highlighted={i === highlightIdx}
            onmousedown={(e) => { e.preventDefault(); selectAthlete(a); }}
            onmousemove={() => { highlightIdx = i; }}
          >
            <span class="num drop-bib">{a.bib_number}</span>
            {#if anonAthlete}
              <span class="drop-name drop-anon">senza nome</span>
            {:else}
              <span class="drop-name">{a.first_name} {a.last_name}</span>
            {/if}
          </button>
        </li>
      {/each}
      {#if freeBib !== null}
        <li>
          <button
            class="drop-item drop-item-free"
            class:highlighted={highlightIdx === filtered.length}
            onmousedown={(e) => {
              e.preventDefault();
              onFreeEntry!(freeBib!);
              search = '';
              dropdownOpen = false;
            }}
            onmousemove={() => { highlightIdx = filtered.length; }}
          >
            <span class="num drop-bib" style="color: var(--accent-pending)">#{freeBib}</span>
            <span class="drop-name" style="color: var(--fg-2)">non in lista → registra e assegna</span>
          </button>
        </li>
      {/if}
    </ul>
  {/if}
</div>

<style>
  .bib-combobox {
    position: relative;
    width: 100%;
  }

  .bib-combobox.compact {
    width: 10rem;
    flex-shrink: 0;
  }

  input {
    width: 100%;
    font-size: 1.1rem;
  }

  .compact input {
    font-size: 0.8rem;
    padding: 0.2rem 0.35rem;
    width: 100%;
  }

  .bib-dropdown {
    position: fixed;
    z-index: 9999;
    max-height: 16rem;
    overflow-y: auto;
    border: 1px solid var(--line-2);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-lg);
    list-style: none;
    padding: 0.25rem 0;
    margin: 0;
  }

  .drop-item {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    text-align: left;
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--fg-0);
  }

  .drop-item.highlighted {
    background: var(--bg-3);
  }

  .drop-bib {
    min-width: 2.5rem;
    font-size: 1rem;
    color: var(--accent-pending);
    flex-shrink: 0;
  }

  .compact .drop-bib {
    font-size: 0.85rem;
    min-width: 2rem;
  }

  .drop-name {
    font-size: 0.9rem;
    color: var(--fg-0);
  }

  .compact .drop-name {
    font-size: 0.8rem;
  }

  .drop-item-free {
    border-top: 1px dashed var(--line-2);
    margin-top: 0.1rem;
  }
  .drop-anon {
    color: var(--accent-pending);
    font-style: italic;
  }
</style>
