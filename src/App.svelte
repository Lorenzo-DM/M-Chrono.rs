<script lang="ts">
  let booted = $state(false);
  $effect(() => { booted = true; });
</script>

<main class="min-h-screen flex items-center justify-center">
  <h1 class="text-5xl font-bold">{booted ? 'Trail Race Timing' : 'Loading...'}</h1>
</main>
