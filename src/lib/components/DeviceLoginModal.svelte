<script lang="ts">
  import { api } from '../api';
  import { on } from '../events';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { onMount } from 'svelte';
  import type { DeviceCodeResponse } from '../types';

  let { onClose }: { onClose: (success: boolean) => void } = $props();
  let resp = $state<DeviceCodeResponse | null>(null);
  let error = $state<string | null>(null);
  let countdown = $state(0);
  let intervalId: number | null = null;
  let unsubA: (() => void) | null = null;
  let unsubF: (() => void) | null = null;

  onMount(() => {
    (async () => {
      try {
        resp = await api.startDeviceLogin();
        countdown = resp.expires_in;
        intervalId = window.setInterval(() => {
          countdown = Math.max(0, countdown - 1);
        }, 1000);
        on('auth:success', () => {
          if (intervalId !== null) clearInterval(intervalId);
          onClose(true);
        }).then(u => { unsubA = u; });
        on('auth:failed', (p: any) => {
          if (intervalId !== null) clearInterval(intervalId);
          error = p?.reason ?? 'login failed';
        }).then(u => { unsubF = u; });
      } catch (e: any) {
        error = e?.message ?? String(e);
      }
    })();
    return () => {
      if (intervalId !== null) clearInterval(intervalId);
      unsubA?.();
      unsubF?.();
    };
  });

  function openVerification() {
    if (!resp) return;
    openUrl(resp.verification_uri_complete ?? resp.verification_uri);
  }
</script>

<div class="fixed inset-0 bg-black/90 flex items-center justify-center" role="dialog">
  <div class="bg-zinc-900 p-8 max-w-xl w-full text-center">
    <h3 class="text-3xl mb-4">Accedi via browser</h3>
    {#if !resp && !error}
      <p>Caricamento…</p>
    {:else if error}
      <p class="text-red-400 text-xl">{error}</p>
    {:else if resp}
      <p class="opacity-80">Visita:</p>
      <button class="text-2xl underline mb-4 break-all" onclick={openVerification}>
        {resp.verification_uri}
      </button>
      <p class="opacity-80">e inserisci il codice:</p>
      <div class="text-6xl font-mono tracking-widest my-6">{resp.user_code}</div>
      <p class="opacity-60">Scade in {countdown}s</p>
    {/if}
    <button class="btn preset-tonal mt-4" onclick={() => onClose(false)}>Annulla</button>
  </div>
</div>
