import { describe, it, expect } from 'vitest';
import { formatMsToHms } from './format';

describe('formatMsToHms', () => {
  it('formats zero', () => expect(formatMsToHms(0)).toBe('00:00:00.000'));
  it('formats sub-second', () => expect(formatMsToHms(123)).toBe('00:00:00.123'));
  it('formats minutes/seconds', () => expect(formatMsToHms(65_500)).toBe('00:01:05.500'));
  it('formats hours', () => expect(formatMsToHms(3_600_000 + 1234)).toBe('01:00:01.234'));
  it('handles negative as zero', () => expect(formatMsToHms(-5)).toBe('00:00:00.000'));
});
