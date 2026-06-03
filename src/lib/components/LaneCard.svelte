<script lang="ts">
    import { onMount } from "svelte";
    import { api } from "../api";
    import { on } from "../events";
    import { formatMsToHms } from "../format";
    import { recentFinishes } from "../stores";
    import type { Course, PendingFinish } from "../types";

    let {
        course,
        size = "full",
        active = false,
        onFocus,
    } = $props<{
        course: Course;
        size?: "full" | "compact";
        active?: boolean;
        onFocus?: () => void;
    }>();

    let elapsed = $state(0);
    let started = $state(false);
    let pending = $state<PendingFinish[]>([]);
    let bib = $state("");
    let bibInput: HTMLInputElement | null = $state(null);
    let error = $state<string | null>(null);
    let flashing = $state(false);
    let busy = $state(false);

    let recent = $derived(
        $recentFinishes.filter((f) => f.course_id === course.id).slice(0, 3),
    );

    $effect(() => {
        let alive = true;
        const tick = async () => {
            if (!alive) return;
            try {
                const snap = await api.pollDisplay();
                const c = snap.courses.find((x) => x.id === course.id);
                if (c) {
                    elapsed = c.elapsed_ms ?? 0;
                    started = c.started;
                }
            } catch {
                // transient
            }
        };
        tick();
        const id = window.setInterval(tick, 100);
        return () => {
            alive = false;
            clearInterval(id);
        };
    });

    async function refreshPending() {
        try {
            pending = await api.getPendingFinishes(course.id);
        } catch {}
    }

    onMount(() => {
        refreshPending();
        let u1: (() => void) | null = null;
        let u2: (() => void) | null = null;
        on("pending:captured", () => refreshPending()).then((u) => {
            u1 = u;
        });
        on("athlete:finished", () => refreshPending()).then((u) => {
            u2 = u;
        });
        return () => {
            u1?.();
            u2?.();
        };
    });

    async function doStart() {
        busy = true;
        error = null;
        try {
            await api.startCourse(course.id);
        } catch (e: any) {
            error = e?.message ?? String(e);
        } finally {
            busy = false;
        }
    }

    async function doTap() {
        try {
            const p = await api.capturePending(course.id);
            pending = [...pending, p];
            flashing = true;
            setTimeout(() => (flashing = false), 700);
        } catch (e: any) {
            error = e?.message ?? String(e);
        }
    }

    async function doFinish(e: Event) {
        e.preventDefault();
        error = null;
        const n = parseInt(bib);
        if (!Number.isFinite(n)) {
            error = "pettorale non valido";
            return;
        }
        try {
            const t = await api.finishByBib(n);
            const ms = Date.now();
            recentFinishes.update((arr) =>
                [
                    {
                        timing_id: t.id,
                        course_id: t.course_id,
                        bib_number: n,
                        total_ms: t.total_time_ms,
                        operator_id: t.operator_id,
                        at_ms: ms,
                    },
                    ...arr,
                ].slice(0, 30),
            );
            bib = "";
            bibInput?.focus();
        } catch (e: any) {
            error = e?.message ?? String(e);
            bib = "";
            bibInput?.focus();
        }
    }

    function handleKey(e: KeyboardEvent) {
        if (!active) return;
        if (e.code === "Space" && document.activeElement !== bibInput) {
            e.preventDefault();
            doTap();
        }
    }
</script>

<svelte:window onkeydown={handleKey} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
    class="panel relative flex flex-col {flashing ? 'flash-capture' : ''}"
    style="border-color: {active
        ? 'var(--accent-running)'
        : 'var(--line-2)'}; transition: border-color 120ms"
    aria-label="Lane {course.name}"
    onclick={() => onFocus?.()}
>
    <!-- Lane header strip -->
    <div
        class="flex items-center justify-between px-4 py-2 border-b"
        style="border-color: var(--line-2)"
    >
        <div class="flex items-center gap-3">
            <div class={started ? "dot-running" : "dot-idle"}></div>
            <div class="hud-strong text-sm" style="color: var(--fg-0)">
                {course.name}
            </div>
            <div
                class="lane-status"
                style="color: {started
                    ? 'var(--accent-running)'
                    : 'var(--fg-2)'}"
            >
                {started ? "RUNNING" : "STANDBY"}
            </div>
        </div>
        <div class="hud">
            <span>PND</span>
            <span class="num ml-1" style="color: var(--accent-pending)"
                >{pending.length}</span
            >
        </div>
    </div>

    <!-- Chrono dial -->
    <div
        class="relative flex-1 flex flex-col items-center justify-center py-{size ===
        'full'
            ? 12
            : 6}"
    >
        <div
            class="chronodial num"
            data-state={started ? "running" : "idle"}
            style="font-size: {size === 'full'
                ? 'clamp(6rem, 14vw, 14rem)'
                : 'clamp(3rem, 8vw, 7rem)'}"
        >
            {formatMsToHms(elapsed)}
        </div>
        {#if course.distance_m}
            <div class="hud mt-3">
                {(course.distance_m / 1000).toFixed(1)} KM
            </div>
        {/if}
    </div>

    <!-- Action row -->
    <div
        class="grid grid-cols-12 gap-2 p-3 border-t"
        style="border-color: var(--line-2)"
    >
        {#if !started}
            <button
                class="btn-base btn-accent-start col-span-12 py-4 text-base"
                disabled={busy}
                onclick={doStart}
            >
                ▶ START PERCORSO
            </button>
        {:else}
            <!-- TAP — primary capture button -->
            <button
                class="btn-base btn-accent-tap col-span-5 py-4 text-base"
                onclick={doTap}
            >
                TAP
                <span
                    class="kbd ml-2"
                    style="background:transparent; color:inherit; border-color:currentColor; opacity:0.6"
                    >␣</span
                >
            </button>

            <!-- Bib input + finish -->
            <form onsubmit={doFinish} class="col-span-7 flex gap-2">
                <input
                    bind:this={bibInput}
                    bind:value={bib}
                    type="number"
                    inputmode="numeric"
                    placeholder="PETTORALE"
                    class="flex-1 text-2xl tabular num"
                    autocomplete="off"
                />
                <button type="submit" class="btn-base btn-primary px-5"
                    >↵</button
                >
            </form>
        {/if}
    </div>

    <!-- Footer: last finish + pending count + recent ticker (full size only) -->
    {#if size === "full"}
        <div
            class="grid grid-cols-12 border-t"
            style="border-color: var(--line-2)"
        >
            <!-- recent ticker -->
            <div
                class="col-span-7 border-r"
                style="border-color: var(--line-2)"
            >
                <div class="hud px-3 pt-2 pb-1">ULTIMI ARRIVI</div>
                {#if recent.length === 0}
                    <div class="px-3 py-3 text-sm" style="color: var(--fg-3)">
                        —
                    </div>
                {:else}
                    {#each recent as f (f.timing_id)}
                        <div class="ticker-row">
                            <span
                                class="num"
                                style="color: var(--accent-running); font-weight: 700"
                                >#{f.bib_number ?? "?"}</span
                            >
                            <span class="num" style="color: var(--fg-0)"
                                >{formatMsToHms(f.total_ms ?? 0)}</span
                            >
                            <span class="hud ml-auto">{f.operator_id}</span>
                        </div>
                    {/each}
                {/if}
            </div>

            <!-- pending preview -->
            <div class="col-span-5">
                <div class="hud px-3 pt-2 pb-1">CODA ASSEGNAZIONE</div>
                {#if pending.length === 0}
                    <div class="px-3 py-3 text-sm" style="color: var(--fg-3)">
                        vuota
                    </div>
                {:else}
                    {#each pending.slice(0, 3) as p (p.id)}
                        <div class="ticker-row">
                            <span
                                class="hud"
                                style="color: var(--accent-pending)"
                                >#{p.id}</span
                            >
                            <span class="num" style="color: var(--fg-1)"
                                >{formatMsToHms(
                                    p.finish_timestamp_ms % 86_400_000,
                                )}</span
                            >
                        </div>
                    {/each}
                    {#if pending.length > 3}
                        <div class="hud px-3 pb-2" style="color: var(--fg-3)">
                            +{pending.length - 3} altri
                        </div>
                    {/if}
                {/if}
            </div>
        </div>
    {/if}

    {#if error}
        <div
            class="px-3 py-2 text-sm border-t"
            style="color: var(--accent-finish); border-color: var(--line-2)"
        >
            {error}
        </div>
    {/if}
</div>
