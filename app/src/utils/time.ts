export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor(seconds / 60) % 60
  seconds = seconds % 60

  return [hours, minutes, seconds].map((v) => (v < 10 ? '0' + v : v)).join(':')
}
