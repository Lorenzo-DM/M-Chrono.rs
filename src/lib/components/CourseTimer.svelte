<script lang="ts">
  import { api } from '../api';
  import { formatMsToHms } from '../format';

  let { courseId } = $props<{ courseId: number }>();
  let elapsed = $state(0);
  let started = $state(false);

  $effect(() => {
    let alive = true;
    const tick = async () => {
      if (!alive) return;
      try {
        const snap = await api.pollDisplay();
        const c = snap.courses.find(x => x.id === courseId);
        if (c) { elapsed = c.elapsed_ms ?? 0; started = c.started; }
      } catch {
        // ignore transient errors during polling
      }
    };
    tick();
    const id = window.setInterval(tick, 100);
    return () => { alive = false; clearInterval(id); };
  });
</script>

<div class="text-center py-8">
  <div class="text-[10rem] font-mono tabular-nums leading-none">
    {formatMsToHms(elapsed)}
  </div>
  <div class="opacity-70 mt-2">{started ? 'IN CORSO' : 'NON AVVIATO'}</div>
</div>
