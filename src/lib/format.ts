export function formatMsToHms(ms: number): string {
  if (!Number.isFinite(ms) || ms < 0) return '00:00:00.000';
  const total = Math.floor(ms);
  const millis = total % 1000;
  const totalSec = Math.floor(total / 1000);
  const sec = totalSec % 60;
  const totalMin = Math.floor(totalSec / 60);
  const min = totalMin % 60;
  const hrs = Math.floor(totalMin / 60);
  const pad = (n: number, w = 2) => n.toString().padStart(w, '0');
  return `${pad(hrs)}:${pad(min)}:${pad(sec)}.${pad(millis, 3)}`;
}
