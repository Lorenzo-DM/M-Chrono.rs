<script lang="ts">
    import { api } from "../api";
    import { config, isAuthenticated as isAuthStore, courses } from "../stores";
    import DeviceLoginModal from "./DeviceLoginModal.svelte";
    import { save as saveDialog } from "@tauri-apps/plugin-dialog";
    import type { AppConfig } from "../types";

    let { onBack: _onBack }: { onBack?: () => void } = $props();

    let form = $state<AppConfig>({
        oidc_issuer_url: $config?.oidc_issuer_url ?? "",
        oidc_client_id: $config?.oidc_client_id ?? "",
        oidc_scopes:
            $config?.oidc_scopes ?? "openid profile email offline_access",
        api_base_url: $config?.api_base_url ?? "",
        sync_interval_secs: $config?.sync_interval_secs ?? 10,
        operator_id: $config?.operator_id ?? "",
        dedup_window_ms: $config?.dedup_window_ms ?? 2000,
        dedup_warn_delta_ms: $config?.dedup_warn_delta_ms ?? 500,
    });

    let saving = $state(false);
    let saved = $state(false);
    let showLogin = $state(false);
    let authed = $state(false);
    let syncStatus = $state<string | null>(null);

    $effect(() => {
        api.isAuthenticated().then((b) => {
            authed = b;
            isAuthStore.set(b);
        });
    });

    async function doLogout() {
        await api.logout();
        authed = false;
        isAuthStore.set(false);
    }

    async function save() {
        saving = true;
        saved = false;
        try {
            const cfg = await api.updateConfig({
                oidc_issuer_url: form.oidc_issuer_url,
                oidc_client_id: form.oidc_client_id,
                oidc_scopes: form.oidc_scopes,
                api_base_url: form.api_base_url,
                sync_interval_secs: Number(form.sync_interval_secs),
                operator_id: form.operator_id,
                dedup_window_ms: Number(form.dedup_window_ms),
                dedup_warn_delta_ms: Number(form.dedup_warn_delta_ms),
            });
            config.set(cfg);
            saved = true;
            setTimeout(() => (saved = false), 1800);
        } finally {
            saving = false;
        }
    }

    async function doSyncData() {
        syncStatus = "…";
        try {
            const s = await api.fetchRemoteData();
            syncStatus = `OK · ${s.courses_count} percorsi · ${s.athletes_count} atleti`;
            courses.set(await api.getCourses());
        } catch (e: any) {
            syncStatus = `ERRORE · ${e?.message ?? e}`;
        }
    }

    async function doExport() {
        const date = new Date().toISOString().slice(0, 10);
        const path = await saveDialog({
            defaultPath: `risultati_${date}.xlsx`,
            filters: [{ name: "Excel", extensions: ["xlsx"] }],
        });
        if (!path) return;
        try {
            const s = await api.exportResultsXlsx(path);
            alert(
                `Esportati ${s.athletes_count} atleti su ${s.courses_count} percorsi`,
            );
        } catch (e: any) {
            alert(e?.message ?? String(e));
        }
    }
</script>

<div class="p-6 max-w-5xl mx-auto">
    <div class="flex items-center justify-between mb-6">
        <div>
            <div class="hud" style="color: var(--fg-3)">CONFIGURAZIONE</div>
            <h2 class="hud-strong text-2xl mt-1" style="color: var(--fg-0)">
                SETTINGS
            </h2>
        </div>
    </div>

    <div class="grid grid-cols-12 gap-4">
        <!-- General -->
        <section class="panel p-4 col-span-6">
            <div class="hud mb-4">GENERALE</div>
            <div class="flex flex-col gap-3">
                <label class="flex flex-col gap-1">
                    <span class="hud">OPERATOR_ID</span>
                    <input bind:value={form.operator_id} placeholder="PC-A" />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="hud">SYNC_INTERVAL (sec)</span>
                    <input
                        type="number"
                        bind:value={form.sync_interval_secs}
                        min="1"
                    />
                </label>
                <div class="grid grid-cols-2 gap-3">
                    <label class="flex flex-col gap-1">
                        <span class="hud">DEDUP WINDOW (ms)</span>
                        <input
                            type="number"
                            bind:value={form.dedup_window_ms}
                            min="0"
                        />
                    </label>
                    <label class="flex flex-col gap-1">
                        <span class="hud">DEDUP WARN Δ (ms)</span>
                        <input
                            type="number"
                            bind:value={form.dedup_warn_delta_ms}
                            min="0"
                        />
                    </label>
                </div>
            </div>
        </section>

        <!-- OIDC -->
        <section class="panel p-4 col-span-6">
            <div class="hud mb-4">OIDC (ZITADEL)</div>
            <div class="flex flex-col gap-3">
                <label class="flex flex-col gap-1">
                    <span class="hud">ISSUER_URL</span>
                    <input
                        bind:value={form.oidc_issuer_url}
                        placeholder="https://example.zitadel.cloud"
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="hud">CLIENT_ID</span>
                    <input bind:value={form.oidc_client_id} />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="hud">SCOPES</span>
                    <input bind:value={form.oidc_scopes} />
                </label>
            </div>
        </section>

        <!-- API base -->
        <section class="panel p-4 col-span-12">
            <div class="hud mb-4">API</div>
            <label class="flex flex-col gap-1">
                <span class="hud">BASE_URL</span>
                <input
                    bind:value={form.api_base_url}
                    placeholder="https://api.example.com"
                />
            </label>
        </section>

        <!-- Save button -->
        <div class="col-span-12 flex items-center gap-3">
            <button
                class="btn-base btn-primary px-6 py-3"
                disabled={saving}
                onclick={save}
            >
                {saving ? "SALVATAGGIO…" : "SALVA CONFIGURAZIONE"}
            </button>
            {#if saved}
                <span class="hud" style="color: var(--accent-start)"
                    >✓ SALVATO</span
                >
            {/if}
        </div>

        <!-- Auth -->
        <section class="panel p-4 col-span-6">
            <div class="hud mb-3">AUTENTICAZIONE</div>
            {#if authed}
                <div class="flex items-center gap-3">
                    <span class="dot-running"></span>
                    <span class="hud-strong" style="color: var(--accent-start)"
                        >LOGIN ATTIVO</span
                    >
                </div>
                <button class="btn-base mt-4" onclick={doLogout}>LOGOUT</button>
            {:else}
                <div class="flex items-center gap-3">
                    <span
                        class="dot-idle"
                        style="background: var(--accent-finish)"
                    ></span>
                    <span class="hud-strong" style="color: var(--accent-finish)"
                        >NON AUTENTICATO</span
                    >
                </div>
                <button
                    class="btn-base btn-primary mt-4"
                    onclick={() => (showLogin = true)}>ACCEDI</button
                >
            {/if}
        </section>

        <!-- Data -->
        <section class="panel p-4 col-span-6">
            <div class="hud mb-3">DATI GARA</div>
            <button class="btn-base btn-primary" onclick={doSyncData}
                >SINCRONIZZA ATLETI/PERCORSI</button
            >
            {#if syncStatus}
                <div class="hud mt-3" style="color: var(--fg-1)">
                    {syncStatus}
                </div>
            {/if}
        </section>

        <!-- Export -->
        <section class="panel p-4 col-span-12">
            <div class="hud mb-3">EXPORT</div>
            <button class="btn-base" onclick={doExport}
                >ESPORTA RISULTATI XLSX</button
            >
        </section>
    </div>
</div>

{#if showLogin}
    <DeviceLoginModal
        onClose={(ok) => {
            showLogin = false;
            if (ok) {
                authed = true;
                isAuthStore.set(true);
            }
        }}
    />
{/if}
