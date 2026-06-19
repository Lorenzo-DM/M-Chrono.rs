// Short synthesized cues via the Web Audio API — no asset files needed.
// Timing on a track is eyes-up work, so capture/assign get an audible tick.

export type Cue = 'capture' | 'assign' | 'error';

const TONES: Record<Cue, { freq: number; ms: number; type: OscillatorType }> = {
  capture: { freq: 880, ms: 70, type: 'square' },
  assign: { freq: 1320, ms: 90, type: 'sine' },
  error: { freq: 200, ms: 180, type: 'sawtooth' },
};

let ctx: AudioContext | null = null;
let muted = false;

const KEY = 'tt.sound.muted';
if (typeof localStorage !== 'undefined') {
  muted = localStorage.getItem(KEY) === '1';
}

export function isMuted(): boolean {
  return muted;
}

export function setMuted(v: boolean): void {
  muted = v;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(KEY, v ? '1' : '0');
  }
}

export function beep(cue: Cue): void {
  if (muted) return;
  try {
    if (!ctx) {
      const AC = window.AudioContext ?? (window as any).webkitAudioContext;
      if (!AC) return;
      ctx = new AC();
    }
    if (ctx.state === 'suspended') ctx.resume();
    const t = TONES[cue];
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = t.type;
    osc.frequency.value = t.freq;
    gain.gain.setValueAtTime(0.0001, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.15, ctx.currentTime + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + t.ms / 1000);
    osc.connect(gain).connect(ctx.destination);
    osc.start();
    osc.stop(ctx.currentTime + t.ms / 1000 + 0.02);
  } catch {
    // audio unavailable — silent no-op
  }
}
