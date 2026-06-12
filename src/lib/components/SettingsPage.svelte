<script lang="ts">
    import { api } from "../api";
    import { config, isAuthenticated as isAuthStore, courses } from "../stores";
    import { themeMode } from "../theme";
    import type { ThemeMode } from "../theme";
    import Button from "../ui/Button.svelte";
    import SegmentedControl from "../ui/SegmentedControl.svelte";
    import DeviceLoginModal from "./DeviceLoginModal.svelte";
    import AthleteImportPanel from "./AthleteImportPanel.svelte";
    import AthleteFormModal from "./AthleteFormModal.svelte";
    import { save as saveDialog } from "@tauri-apps/plugin-dialog";
    import type { AppConfig, Athlete } from "../types";

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
        sync_enabled: $config?.sync_enabled ?? false,
    });

    let saving = $state(false);
    let saved = $state(false);
    let showLogin = $state(false);
    let authed = $state(false);
    let athletes = $state<Athlete[]>([]);
    let athleteFilter = $state("");
    let editingAthlete = $state<Athlete | null>(null);
    let showAthleteForm = $state(false);
    let athleteError = $state<string | null>(null);

    $effect(() => {
        api.isAuthenticated().then((b) => {
            authed = b;
            isAuthStore.set(b);
        });
    });

    $effect(() => {
        loadAthletes();
    });

    async function loadAthletes() {
        athletes = await api.getAllAthletes();
    }

    let courseNames = $derived(
        new Map($courses.map((c) => [c.id, c.name])),
    );

    let filteredAthletes = $derived(
        athletes.filter((a) => {
            const q = athleteFilter.trim().toLowerCase();
            if (!q) return true;
            return (
                String(a.bib_number).includes(q) ||
                a.first_name.toLowerCase().includes(q) ||
                a.last_name.toLowerCase().includes(q) ||
                (courseNames.get(a.course_id) ?? "").toLowerCase().includes(q)
            );
        }),
    );

    async function doLogout() {
        await api.logout();
        authed = false;
        isAuthStore.set(false);
    }

    async function toggleSync(enabled: boolean) {
        form.sync_enabled = enabled;
        const cfg = await api.updateConfig({ sync_enabled: enabled });
        config.set(cfg);
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
                sync_enabled: form.sync_enabled,
            });
            config.set(cfg);
            saved = true;
            setTimeout(() => (saved = false), 1800);
        } finally {
            saving = false;
        }
    }

    function editAthlete(a: Athlete) {
        editingAthlete = a;
        showAthleteForm = true;
    }

    async function removeAthlete(a: Athlete) {
        athleteError = null;
        if (!confirm(`Eliminare ${a.first_name} ${a.last_name} (pett. ${a.bib_number})?`)) return;
        try {
            await api.deleteAthlete(a.id);
            await loadAthletes();
        } catch (e: any) {
            athleteError = e?.message ?? JSON.stringify(e);
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
                    <span class="hud">NOME OPERATORE</span>
                    <input bind:value={form.operator_id} placeholder="PC-A" />
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

        <!-- Aspetto -->
        <section class="panel p-4 col-span-6">
            <div class="hud mb-3">ASPETTO</div>
            <SegmentedControl
                ariaLabel="Tema"
                options={[
                    { value: 'auto', label: 'Auto', title: 'Tema automatico' },
                    { value: 'light', label: 'Chiaro', title: 'Tema chiaro' },
                    { value: 'dark', label: 'Scuro', title: 'Tema scuro' },
                ]}
                value={$themeMode}
                onChange={(v) => themeMode.set(v as ThemeMode)}
            />
        </section>

        <!-- Athletes -->
        <section class="panel p-4 col-span-12">
            <div class="hud mb-4">ATLETI</div>
            <AthleteImportPanel onImported={loadAthletes} />

            {#if athleteError}
                <div class="hud mt-3" style="color: var(--accent-finish)">
                    ⚠ {athleteError}
                </div>
            {/if}

            {#if athletes.length > 0}
                <div class="mt-5 flex items-center gap-3">
                    <input
                        bind:value={athleteFilter}
                        placeholder="Cerca per pettorale, nome, percorso…"
                        class="flex-1 max-w-md"
                    />
                    <span class="hud" style="color: var(--fg-3)">
                        {filteredAthletes.length} / {athletes.length}
                    </span>
                </div>
                <div class="mt-3 max-h-80 overflow-auto border rounded"
                     style="border-color: var(--line-1)">
                    <table class="w-full text-sm">
                        <thead>
                            <tr class="hud text-left" style="background: var(--bg-2)">
                                <th class="px-3 py-2">PETT.</th>
                                <th class="px-3 py-2">NOME</th>
                                <th class="px-3 py-2">COGNOME</th>
                                <th class="px-3 py-2">PERCORSO</th>
                                <th class="px-3 py-2"></th>
                                <th class="px-3 py-2"></th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each filteredAthletes as a (a.id)}
                                <tr class="border-t" style="border-color: var(--line-1)">
                                    <td class="px-3 py-1.5 num" style="color: var(--fg-0)">{a.bib_number}</td>
                                    <td class="px-3 py-1.5" style="color: var(--fg-1)">{a.first_name}</td>
                                    <td class="px-3 py-1.5" style="color: var(--fg-1)">{a.last_name}</td>
                                    <td class="px-3 py-1.5" style="color: var(--fg-2)">
                                        {courseNames.get(a.course_id) ?? a.course_id}
                                    </td>
                                    <td class="px-3 py-1.5">
                                        {#if a.id < 0}
                                            <span class="hud" style="color: var(--fg-3)">LOCALE</span>
                                        {/if}
                                    </td>
                                    <td class="px-3 py-1.5 text-right whitespace-nowrap">
                                        <Button variant="ghost" size="sm" onclick={() => editAthlete(a)} title="Modifica">✎</Button>
                                        <Button variant="ghost" size="sm" onclick={() => removeAthlete(a)} title="Elimina">✕</Button>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {:else}
                <div class="hud mt-4" style="color: var(--fg-3)">
                    Nessun atleta. Importa un file o aggiungili manualmente.
                </div>
            {/if}
        </section>

        <!-- Sync -->
        <section class="panel p-4 col-span-12">
            <div class="flex items-center justify-between mb-4">
                <div class="hud">SINCRONIZZAZIONE BACKEND</div>
                <SegmentedControl
                    ariaLabel="Sincronizzazione"
                    options={[
                        { value: 'off', label: 'OFF', title: 'Solo locale' },
                        { value: 'on', label: 'ON', title: 'Sync con backend' },
                    ]}
                    value={form.sync_enabled ? 'on' : 'off'}
                    onChange={(v) => toggleSync(v === 'on')}
                />
            </div>

            {#if !form.sync_enabled}
                <div class="hud" style="color: var(--fg-3)">
                    L'app lavora solo in locale. Attiva per sincronizzare tempi
                    e atleti con il backend.
                </div>
            {:else}
                <div class="grid grid-cols-12 gap-4">
                    <!-- OIDC -->
                    <div class="col-span-6 flex flex-col gap-3">
                        <div class="hud">OIDC (ZITADEL)</div>
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

                    <!-- API -->
                    <div class="col-span-6 flex flex-col gap-3">
                        <div class="hud">API</div>
                        <label class="flex flex-col gap-1">
                            <span class="hud">BASE_URL</span>
                            <input
                                bind:value={form.api_base_url}
                                placeholder="https://api.example.com"
                            />
                        </label>
                        <label class="flex flex-col gap-1">
                            <span class="hud">SYNC_INTERVAL (sec)</span>
                            <input
                                type="number"
                                bind:value={form.sync_interval_secs}
                                min="1"
                            />
                        </label>
                    </div>

                    <!-- Auth -->
                    <div class="col-span-12 border-t pt-4" style="border-color: var(--line-1)">
                        <div class="hud mb-3">AUTENTICAZIONE</div>
                        {#if authed}
                            <div class="flex items-center gap-3">
                                <span class="dot-running"></span>
                                <span class="hud-strong" style="color: var(--accent-start)"
                                    >LOGIN ATTIVO</span
                                >
                                <Button onclick={doLogout}>LOGOUT</Button>
                            </div>
                        {:else}
                            <div class="flex items-center gap-3">
                                <span
                                    class="dot-idle"
                                    style="background: var(--accent-finish)"
                                ></span>
                                <span class="hud-strong" style="color: var(--accent-finish)"
                                    >NON AUTENTICATO</span
                                >
                                <Button variant="primary" onclick={() => (showLogin = true)}>ACCEDI</Button>
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}
        </section>

        <!-- Save button -->
        <div class="col-span-12 flex items-center gap-3">
            <Button
                variant="primary"
                disabled={saving}
                onclick={save}
                class="px-6 py-3"
            >
                {saving ? "SALVATAGGIO…" : "SALVA CONFIGURAZIONE"}
            </Button>
            {#if saved}
                <span class="hud" style="color: var(--accent-start)"
                    >✓ SALVATO</span
                >
            {/if}
        </div>

        <!-- Export -->
        <section class="panel p-4 col-span-12">
            <div class="hud mb-3">EXPORT</div>
            <Button onclick={doExport}>ESPORTA RISULTATI XLSX</Button>
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

{#if showAthleteForm}
    <AthleteFormModal
        athlete={editingAthlete}
        onClose={(savedOk) => {
            showAthleteForm = false;
            editingAthlete = null;
            if (savedOk) loadAthletes();
        }}
    />
{/if}
