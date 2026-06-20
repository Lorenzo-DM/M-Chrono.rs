<script lang="ts">
    import { api } from "../api";
    import { config, isAuthenticated as isAuthStore, courses } from "../stores";
    import { themeMode } from "../theme";
    import type { ThemeMode } from "../theme";
    import Button from "../ui/Button.svelte";
    import { TriangleAlert, Check, X } from 'lucide-svelte';
    import SegmentedControl from "../ui/SegmentedControl.svelte";
    import DeviceLoginModal from "./DeviceLoginModal.svelte";
    import AthleteImportPanel from "./AthleteImportPanel.svelte";
    import AthleteFormModal from "./AthleteFormModal.svelte";
    import RaceSetupPanel from "./RaceSetupPanel.svelte";
    import CheckpointsPanel from "./CheckpointsPanel.svelte";
    import { isMuted, setMuted } from "../sound";
    import type { AppConfig, Athlete } from "../types";
    import { locale, SUPPORTED_LOCALES, t, i } from "../i18n";
    import type { Locale } from "../i18n";

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

    type SettingsTab = "general" | "race" | "athletes" | "sync";
    let tabs = $derived([
        { id: "general" as SettingsTab, label: $t.settings.tabs.general },
        { id: "race"    as SettingsTab, label: $t.settings.tabs.race },
        { id: "athletes"as SettingsTab, label: $t.settings.tabs.athletes },
        { id: "sync"    as SettingsTab, label: $t.settings.tabs.sync },
    ]);
    let tab = $state<SettingsTab>("general");

    let saving = $state(false);
    let saved = $state(false);
    let showLogin = $state(false);
    let authed = $state(false);
    let athletes = $state<Athlete[]>([]);
    let athleteFilter = $state("");
    let editingAthlete = $state<Athlete | null>(null);
    let showAthleteForm = $state(false);
    let athleteError = $state<string | null>(null);
    let soundOn = $state(!isMuted());

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
        if (!confirm(i($t.settings.athletes.confirmDelete, {
            firstName: a.first_name,
            lastName: a.last_name,
            bib: a.bib_number,
        }))) return;
        try {
            await api.deleteAthlete(a.id);
            await loadAthletes();
        } catch (e: any) {
            athleteError = e?.message ?? JSON.stringify(e);
        }
    }

    function toggleSound(on: boolean) {
        soundOn = on;
        setMuted(!on);
    }
</script>

<div class="p-6 max-w-5xl mx-auto">
    <div class="flex items-center justify-between mb-5">
        <div>
            <div class="hud" style="color: var(--fg-3)">{$t.settings.subtitle}</div>
            <h2 class="hud-strong text-2xl mt-1" style="color: var(--fg-0)">
                {$t.settings.title}
            </h2>
        </div>
    </div>

    <!-- Sub-tabs -->
    <div class="settings-tabs" role="tablist" aria-label={$t.settings.subtitle}>
        {#each tabs as t_ (t_.id)}
            <button
                class="settings-tab"
                role="tab"
                aria-selected={tab === t_.id}
                data-active={tab === t_.id}
                onclick={() => (tab = t_.id)}
            >
                {t_.label}
            </button>
        {/each}
    </div>

    {#snippet saveFooter()}
        <div class="flex items-center gap-3 pt-1">
            <Button
                variant="primary"
                disabled={saving}
                onclick={save}
                class="px-6 py-3"
            >
                {saving ? $t.common.saving : $t.settings.saveConfig}
            </Button>
            {#if saved}
                <span class="hud" style="color: var(--accent-start)"><Check size={14} /> {$t.common.saved}</span>
            {/if}
        </div>
    {/snippet}

    <!-- ============ GENERALE ============ -->
    {#if tab === "general"}
    <div class="grid grid-cols-12 gap-4">
        <section class="panel p-4 col-span-6">
            <div class="hud mb-4">{$t.settings.general.sectionTitle}</div>
            <div class="flex flex-col gap-3">
                <label class="flex flex-col gap-1">
                    <span class="hud">{$t.settings.general.operatorName}</span>
                    <input bind:value={form.operator_id} placeholder="PC-A" />
                </label>
                <div class="grid grid-cols-2 gap-3">
                    <label class="flex flex-col gap-1">
                        <span class="hud">{$t.settings.general.dedupWindow}</span>
                        <input type="number" bind:value={form.dedup_window_ms} min="0" />
                    </label>
                    <label class="flex flex-col gap-1">
                        <span class="hud">{$t.settings.general.dedupWarnDelta}</span>
                        <input type="number" bind:value={form.dedup_warn_delta_ms} min="0" />
                    </label>
                </div>
            </div>
        </section>

        <section class="panel p-4 col-span-6">
            <div class="hud mb-3">{$t.settings.appearance.sectionTitle}</div>
            <SegmentedControl
                ariaLabel={$t.settings.appearance.theme}
                options={[
                    { value: 'auto',  label: $t.settings.appearance.themeAuto,  title: $t.settings.appearance.themeAutoTitle },
                    { value: 'light', label: $t.settings.appearance.themeLight, title: $t.settings.appearance.themeLightTitle },
                    { value: 'dark',  label: $t.settings.appearance.themeDark,  title: $t.settings.appearance.themeDarkTitle },
                ]}
                value={$themeMode}
                onChange={(v) => themeMode.set(v as ThemeMode)}
            />
            <div class="hud mt-4 mb-2">{$t.settings.appearance.sound}</div>
            <SegmentedControl
                ariaLabel={$t.settings.appearance.sound}
                options={[
                    { value: 'on',  label: $t.common.on,  title: $t.settings.appearance.soundOnTitle },
                    { value: 'off', label: $t.common.off, title: $t.settings.appearance.soundOffTitle },
                ]}
                value={soundOn ? 'on' : 'off'}
                onChange={(v) => toggleSound(v === 'on')}
            />
            <div class="hud mt-4 mb-2">{$t.settings.appearance.language}</div>
            <select
                value={$locale}
                onchange={(e) => locale.set((e.target as HTMLSelectElement).value as Locale)}
            >
                {#each SUPPORTED_LOCALES as l (l.code)}
                    <option value={l.code}>{l.nativeName}</option>
                {/each}
            </select>
        </section>

        <div class="col-span-12">{@render saveFooter()}</div>
    </div>
    {/if}

    <!-- ============ GARA ============ -->
    {#if tab === "race"}
    <div class="grid grid-cols-12 gap-4">
        <section class="panel p-4 col-span-12">
            <div class="hud mb-4">{$t.settings.race.sectionTitle}</div>
            <RaceSetupPanel />
        </section>

        <section class="panel p-4 col-span-12">
            <div class="hud mb-4">{$t.settings.race.checkpointsSectionTitle}</div>
            <CheckpointsPanel />
        </section>
    </div>
    {/if}

    <!-- ============ ATLETI ============ -->
    {#if tab === "athletes"}
    <div class="grid grid-cols-12 gap-4">
        <section class="panel p-4 col-span-12">
            <div class="hud mb-4">{$t.settings.athletes.sectionTitle}</div>
            <AthleteImportPanel onImported={loadAthletes} />

            {#if athleteError}
                <div class="hud mt-3" style="color: var(--accent-finish)">
                    <TriangleAlert size={14} /> {athleteError}
                </div>
            {/if}

            {#if athletes.length > 0}
                <div class="mt-5 flex items-center gap-3">
                    <input
                        bind:value={athleteFilter}
                        placeholder={$t.settings.athletes.searchPlaceholder}
                        class="flex-1 max-w-md"
                    />
                    <span class="hud" style="color: var(--fg-3)">
                        {i($t.settings.athletes.countLabel, { filtered: filteredAthletes.length, total: athletes.length })}
                    </span>
                </div>
                <div class="mt-3 max-h-80 overflow-auto border rounded"
                     style="border-color: var(--line-1)">
                    <table class="w-full text-sm">
                        <thead>
                            <tr class="hud text-left" style="background: var(--bg-2)">
                                <th class="px-3 py-2">{$t.settings.athletes.columns.bib}</th>
                                <th class="px-3 py-2">{$t.settings.athletes.columns.firstName}</th>
                                <th class="px-3 py-2">{$t.settings.athletes.columns.lastName}</th>
                                <th class="px-3 py-2">{$t.settings.athletes.columns.category}</th>
                                <th class="px-3 py-2">{$t.settings.athletes.columns.course}</th>
                                <th class="px-3 py-2"></th>
                                <th class="px-3 py-2"></th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each filteredAthletes as a (a.id)}
                                <tr class="border-t" style="border-color: var(--line-1)">
                                    <td class="px-3 py-1.5 num" style="color: var(--fg-0)">{a.bib_number}</td>
                                    <td class="px-3 py-1.5" style="color: var(--fg-1)">
                                        {#if a.anonymous}
                                            <span class="hud" style="color: var(--accent-pending)">{$t.settings.athletes.anonymous}</span>
                                        {:else}{a.first_name}{/if}
                                    </td>
                                    <td class="px-3 py-1.5" style="color: var(--fg-1)">{a.last_name}</td>
                                    <td class="px-3 py-1.5" style="color: var(--fg-2)">{a.category ?? ''}</td>
                                    <td class="px-3 py-1.5" style="color: var(--fg-2)">
                                        {courseNames.get(a.course_id) ?? a.course_id}
                                    </td>
                                    <td class="px-3 py-1.5">
                                        {#if a.id < 0}
                                            <span class="hud" style="color: var(--fg-3)">{$t.settings.athletes.localLabel}</span>
                                        {/if}
                                    </td>
                                    <td class="px-3 py-1.5 text-right whitespace-nowrap">
                                        <Button variant="ghost" size="sm" onclick={() => editAthlete(a)} title={$t.common.edit}>✎</Button>
                                        <Button variant="ghost" size="sm" onclick={() => removeAthlete(a)} title={$t.common.delete}><X size={14} /></Button>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {:else}
                <div class="hud mt-4" style="color: var(--fg-3)">
                    {$t.settings.athletes.noAthletes}
                </div>
            {/if}
        </section>
    </div>
    {/if}

    <!-- ============ SYNC ============ -->
    {#if tab === "sync"}
    <div class="grid grid-cols-12 gap-4">
        <section class="panel p-4 col-span-12">
            <div class="flex items-center justify-between mb-4">
                <div class="hud">{$t.settings.sync.sectionTitle}</div>
                <SegmentedControl
                    ariaLabel={$t.settings.sync.sectionTitle}
                    options={[
                        { value: 'off', label: $t.settings.sync.toggleOffLabel, title: $t.settings.sync.toggleOffTitle },
                        { value: 'on',  label: $t.settings.sync.toggleOnLabel,  title: $t.settings.sync.toggleOnTitle },
                    ]}
                    value={form.sync_enabled ? 'on' : 'off'}
                    onChange={(v) => toggleSync(v === 'on')}
                />
            </div>

            {#if !form.sync_enabled}
                <div class="hud" style="color: var(--fg-3)">
                    {$t.settings.sync.offlineDescription}
                </div>
            {:else}
                <div class="grid grid-cols-12 gap-4">
                    <div class="col-span-6 flex flex-col gap-3">
                        <div class="hud">{$t.settings.sync.oidcSection}</div>
                        <label class="flex flex-col gap-1">
                            <span class="hud">ISSUER_URL</span>
                            <input bind:value={form.oidc_issuer_url} placeholder="https://example.zitadel.cloud" />
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

                    <div class="col-span-6 flex flex-col gap-3">
                        <div class="hud">{$t.settings.sync.apiSection}</div>
                        <label class="flex flex-col gap-1">
                            <span class="hud">BASE_URL</span>
                            <input bind:value={form.api_base_url} placeholder="https://api.example.com" />
                        </label>
                        <label class="flex flex-col gap-1">
                            <span class="hud">{$t.settings.sync.syncIntervalLabel}</span>
                            <input type="number" bind:value={form.sync_interval_secs} min="1" />
                        </label>
                    </div>

                    <div class="col-span-12 border-t pt-4" style="border-color: var(--line-1)">
                        <div class="hud mb-3">{$t.settings.sync.authSection}</div>
                        {#if authed}
                            <div class="flex items-center gap-3">
                                <span class="dot-running"></span>
                                <span class="hud-strong" style="color: var(--accent-start)">{$t.settings.sync.loginActive}</span>
                                <Button onclick={doLogout}>{$t.settings.sync.logout}</Button>
                            </div>
                        {:else}
                            <div class="flex items-center gap-3">
                                <span class="dot-idle" style="background: var(--accent-finish)"></span>
                                <span class="hud-strong" style="color: var(--accent-finish)">{$t.settings.sync.notAuthenticated}</span>
                                <Button variant="primary" onclick={() => (showLogin = true)}>{$t.settings.sync.login}</Button>
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}
        </section>

        <div class="col-span-12">{@render saveFooter()}</div>
    </div>
    {/if}
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

<style>
    .settings-tabs {
        display: flex;
        gap: 0.25rem;
        padding: 0.25rem;
        margin-bottom: 1.5rem;
        background: var(--bg-2);
        border: 1px solid var(--line-1);
        border-radius: var(--radius-md);
        overflow-x: auto;
    }
    .settings-tab {
        flex: 1;
        min-width: max-content;
        padding: 0.55rem 1rem;
        font-family: "IBM Plex Mono", ui-monospace, monospace;
        font-size: 0.8rem;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--fg-3);
        background: transparent;
        border: none;
        border-radius: var(--radius-sm);
        cursor: pointer;
        transition:
            color 120ms ease,
            background 120ms ease,
            box-shadow 120ms ease;
    }
    .settings-tab:hover {
        color: var(--fg-1);
    }
    .settings-tab[data-active="true"] {
        color: var(--fg-0);
        background: var(--bg-0);
        box-shadow: inset 0 -2px 0 var(--accent-running);
    }
</style>
