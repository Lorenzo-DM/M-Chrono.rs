// Long-run smoke procedure (manual).
//
// This file is documentation, not an executable harness. Tauri commands
// require the running Tauri runtime; they cannot be invoked from plain
// Node. Run the procedure below from the running app's WebView devtools.
//
// Setup:
//   1. bun run tauri dev
//   2. In Settings, set operator_id (e.g. "PC-A") and authenticate.
//   3. Click "Sincronizza atleti/percorsi" to seed local data.
//   4. Navigate to a course and click START PERCORSO.
//
// In the running app, open devtools (right-click → Inspect → Console)
// and paste:
//
//   for (let bib = 1; bib <= 200; bib++) {
//     try {
//       await window.__TAURI__.core.invoke('finish_by_bib', { bib });
//     } catch (e) {
//       console.warn(`bib ${bib} failed:`, e);
//     }
//     await new Promise(r => setTimeout(r, 50));
//   }
//
// Expected outcome over 8 hours:
//   - Memory footprint of trailtrace-stopwatch in Activity Monitor/Task
//     Manager stays bounded (no monotonic growth across the run).
//   - cargo test still green.
//   - DB at ~/Library/Application Support/com.tauri.dev/race.db grows
//     monotonically; no corruption.
//   - logs/race.log.<date> rotates daily, no obvious errors.
//   - Sync queue empties after each network-restoration cycle.
//
// If running offline tests, disable Wi-Fi mid-run; the timer must
// continue to work. Re-enable Wi-Fi and verify push drains.

console.log('See file comments for procedure.');
