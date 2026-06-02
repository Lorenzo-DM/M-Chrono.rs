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

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose(false);
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="modal-backdrop" role="dialog">
  <div class="panel-2 w-full max-w-xl">
    <div class="px-5 py-3 flex items-center justify-between border-b" style="border-color: var(--line-2)">
      <div class="hud-strong" style="color: var(--accent-running)">ACCEDI VIA BROWSER</div>
      <button class="btn-base btn-ghost text-xs" onclick={() => onClose(false)}>ESC</button>
    </div>

    <div class="p-6">
      {#if !resp && !error}
        <div class="hud" style="color: var(--fg-2)">RICHIESTA DEVICE CODE…</div>
      {:else if error}
        <div class="hud mb-2" style="color: var(--accent-finish)">ERRORE</div>
        <div class="text-base" style="color: var(--fg-1)">{error}</div>
      {:else if resp}
        <div class="hud mb-2">VISITA URL</div>
        <button
          class="text-base mb-6 break-all underline text-left w-full"
          style="color: var(--accent-running)"
          onclick={openVerification}
        >
          {resp.verification_uri_complete ?? resp.verification_uri}
        </button>

        <div class="hud mb-2">INSERISCI CODICE</div>
        <div
          class="chronodial num text-7xl tracking-[0.15em] py-4 px-6 panel"
          data-state="running"
          style="text-align: center"
        >
          {resp.user_code}
        </div>

        <div class="flex items-center justify-between mt-6">
          <div class="hud">SCADENZA</div>
          <div class="num text-xl" style="color: {countdown > 60 ? 'var(--fg-0)' : 'var(--accent-finish)'}">
            {countdown}s
          </div>
        </div>
        <div class="hud mt-1" style="color: var(--fg-3)">in attesa di conferma…</div>
      {/if}

      <button class="btn-base mt-6 w-full py-3" onclick={() => onClose(false)}>ANNULLA</button>
    </div>
  </div>
</div>
