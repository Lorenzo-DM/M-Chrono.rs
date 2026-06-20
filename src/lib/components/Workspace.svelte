<script lang="ts">
  import { breakpoint } from '../breakpoint';
  import { courses, layoutMode, activeCourseId, visibleLanes } from '../stores';
  import { normalizeVisibleLanes, toggleLane } from '../splitLanes';
  import LaneCard from './LaneCard.svelte';
  import type { Course } from '../types';
  import { CircleDot, Circle } from 'lucide-svelte';

  let effectiveLayout = $derived.by(() => ($breakpoint === 'mobile' ? 'tabs' : $layoutMode));

  // Single source of truth for which lanes split mode shows.
  let visibleLaneIds = $derived.by<number[]>(() =>
    effectiveLayout === 'split'
      ? normalizeVisibleLanes($courses.map(c => c.id), $visibleLanes)
      : []
  );

  // Pick which courses to show based on layout mode
  let visibleCourses = $derived.by<Course[]>(() => {
    const all = $courses;
    if (all.length === 0) return [];
    if (effectiveLayout === 'tabs') {
      const active = all.find(c => c.id === $activeCourseId) ?? all[0];
      return [active];
    }
    if (effectiveLayout === 'grid') return all;
    // split
    return visibleLaneIds
      .map(id => all.find(c => c.id === id))
      .filter(Boolean) as Course[];
  });

  // Ensure activeCourseId is set
  $effect(() => {
    if (!$activeCourseId && $courses.length > 0) {
      activeCourseId.set($courses[0].id);
    }
  });

  // Persist the normalized selection so the picker, the grid and a reload agree.
  $effect(() => {
    if (effectiveLayout !== 'split' || $courses.length === 0) return;
    const norm = visibleLaneIds;
    const cur = $visibleLanes;
    if (norm.length !== cur.length || norm.some((id, i) => id !== cur[i])) {
      visibleLanes.set(norm);
    }
  });

  function gridClass() {
    if (effectiveLayout === 'tabs') return 'grid-cols-1';
    if (effectiveLayout === 'split') {
      const n = visibleCourses.length;
      if (n <= 1) return 'grid-cols-1';
      return n >= 3 ? 'grid-cols-3' : 'grid-cols-2';
    }
    // grid
    const n = $courses.length;
    if (n <= 2) return 'grid-cols-2';
    if (n <= 4) return 'grid-cols-2 grid-rows-2';
    return 'grid-cols-3';
  }

  function toggleSplitCourse(courseId: number) {
    if (effectiveLayout !== 'split') return;
    visibleLanes.set(toggleLane(visibleLaneIds, courseId));
  }
</script>

<div class="h-full flex flex-col">
  <!-- Tab strip (tabs mode) or split-lane picker (split mode) -->
  {#if effectiveLayout === 'tabs'}
    <div class="tab-strip overflow-x-auto">
      {#each $courses as c (c.id)}
        <button
          class="tab-item hud-strong whitespace-nowrap"
          data-active={c.id === ($activeCourseId ?? $courses[0]?.id)}
          onclick={() => activeCourseId.set(c.id)}
        >
          <span class="num">{c.name}</span>
          {#if c.ended_at_ms}
            <span class="dot-idle ml-2 inline-block" style="background: var(--accent-finish)"></span>
          {:else if c.started_at_ms}
            <span class="dot-running ml-2 inline-block"></span>
          {/if}
        </button>
      {/each}
    </div>
  {:else if effectiveLayout === 'split'}
    <div class="flex items-center gap-2 px-4 py-2 border-b" style="border-color: var(--line-2)">
      <div class="hud">CORSIE VISIBILI</div>
      {#each $courses as c (c.id)}
        {@const on = visibleLaneIds.includes(c.id)}
        <button
          class="lane-status"
          style="color: {on ? 'var(--accent-running)' : 'var(--fg-3)'}; cursor: pointer"
          onclick={() => toggleSplitCourse(c.id)}
        >
          {#if on}<CircleDot size={12} />{:else}<Circle size={12} />{/if} {c.name}
        </button>
      {/each}
      <div class="hud ml-auto" style="color: var(--fg-3)">max 4 corsie</div>
    </div>
  {/if}

  <!-- Lane grid -->
  <div class="flex-1 p-3 min-h-0">
    {#if visibleCourses.length === 0}
      <div class="h-full flex items-center justify-center">
        <div class="text-center">
          <div class="hud-strong text-lg mb-3" style="color: var(--fg-2)">NESSUN PERCORSO</div>
          <div class="text-sm" style="color: var(--fg-3)">
            Configura l'operator_id in <span class="kbd">SETTINGS</span> e premi
            <span class="kbd">SINCRONIZZA</span>
          </div>
        </div>
      </div>
    {:else}
      <div class="grid gap-3 h-full {gridClass()}">
        {#each visibleCourses as c (c.id)}
          <LaneCard
            course={c}
            size={visibleCourses.length === 1 ? 'full' : 'compact'}
            active={c.id === ($activeCourseId ?? $courses[0]?.id)}
            onFocus={() => activeCourseId.set(c.id)}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>
