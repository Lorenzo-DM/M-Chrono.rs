const MAX_LANES = 4;
const DEFAULT_LANES = 2;

/**
 * Resolve the lanes actually shown in split mode from the persisted selection.
 * - drops ids whose course no longer exists
 * - falls back to the first courses when the selection is empty
 * - caps the result at the maximum number of lanes
 * Idempotent: normalizing a normalized list returns the same list.
 */
export function normalizeVisibleLanes(allIds: number[], visible: number[]): number[] {
  if (allIds.length === 0) return [];
  const valid = visible.filter(id => allIds.includes(id));
  if (valid.length === 0) {
    return allIds.slice(0, Math.min(DEFAULT_LANES, allIds.length));
  }
  return valid.slice(0, MAX_LANES);
}

/**
 * Toggle a lane in the visible set. Adds when absent (capped at the max,
 * dropping the oldest), removes when present, but never removes the last lane
 * so split mode always shows something.
 */
export function toggleLane(current: number[], id: number): number[] {
  if (current.includes(id)) {
    if (current.length <= 1) return current;
    return current.filter(x => x !== id);
  }
  return [...current, id].slice(-MAX_LANES);
}
