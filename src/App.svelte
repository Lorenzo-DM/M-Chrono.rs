<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from './lib/api';
  import { courses, config } from './lib/stores';
  import Header from './lib/components/Header.svelte';

  type View = { kind: 'home' } | { kind: 'course'; id: number } | { kind: 'settings' };
  let view = $state<View>({ kind: 'home' });
  let booted = $state(false);

  onMount(async () => {
    const [c, cfg] = await Promise.all([api.getCourses(), api.getConfig()]);
    courses.set(c);
    config.set(cfg);
    booted = true;
  });
</script>

{#if !booted}
  <main class="min-h-screen flex items-center justify-center">
    <p class="text-3xl">Caricamento…</p>
  </main>
{:else}
  <Header onSettings={() => view = { kind: 'settings' }} />
  <main class="p-6">
    {#if view.kind === 'home'}
      <h2 class="text-3xl mb-4">Percorsi</h2>
      {#if $courses.length === 0}
        <p class="opacity-70">Nessun percorso. Configura operator_id e sincronizza dati da Settings.</p>
      {:else}
        <div class="grid grid-cols-2 gap-4">
          {#each $courses as c (c.id)}
            <button class="card preset-tonal p-6 text-left" onclick={() => view = { kind: 'course', id: c.id }}>
              <div class="text-2xl font-semibold">{c.name}</div>
              <div class="text-sm opacity-70">
                {c.started_at_ms ? 'In corso' : 'Non avviato'}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    {:else if view.kind === 'course'}
      <p>Course page placeholder for {view.id}</p>
      <button class="btn preset-tonal" onclick={() => view = { kind: 'home' }}>← Home</button>
    {:else}
      <p>Settings page placeholder</p>
      <button class="btn preset-tonal" onclick={() => view = { kind: 'home' }}>← Home</button>
    {/if}
  </main>
{/if}
