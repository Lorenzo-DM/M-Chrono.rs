import { describe, it, expect } from 'vitest';
import { normalizeVisibleLanes, toggleLane } from './splitLanes';

describe('normalizeVisibleLanes', () => {
  it('defaults to the first two courses when nothing is selected', () => {
    expect(normalizeVisibleLanes([1, 2, 3, 4], [])).toEqual([1, 2]);
  });

  it('keeps a single explicit selection without snapping back to two', () => {
    expect(normalizeVisibleLanes([1, 2, 3, 4], [3])).toEqual([3]);
  });

  it('prunes ids whose course no longer exists', () => {
    expect(normalizeVisibleLanes([1, 2], [2, 9])).toEqual([2]);
  });

  it('caps the visible set at four', () => {
    expect(normalizeVisibleLanes([1, 2, 3, 4, 5], [1, 2, 3, 4, 5])).toEqual([1, 2, 3, 4]);
  });

  it('returns empty when there are no courses', () => {
    expect(normalizeVisibleLanes([], [1, 2])).toEqual([]);
  });
});

describe('toggleLane', () => {
  it('adds an unselected lane', () => {
    expect(toggleLane([1, 2], 3)).toEqual([1, 2, 3]);
  });

  it('removes a selected lane', () => {
    expect(toggleLane([1, 2], 1)).toEqual([2]);
  });

  it('keeps at least one lane visible (cannot turn off the last)', () => {
    expect(toggleLane([2], 2)).toEqual([2]);
  });

  it('caps at four, dropping the oldest', () => {
    expect(toggleLane([1, 2, 3, 4], 5)).toEqual([2, 3, 4, 5]);
  });
});
