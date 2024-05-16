export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor(seconds / 60) % 60
  seconds = seconds % 60

  return [hours, minutes, seconds].map((v) => (v < 10 ? '0' + v : v)).join(':')
}

export function durationFromString(duration: string): number {
  const chunkNames = ['hours', 'minutes', 'seconds']
  const [seconds, minutes, hours] = duration
    .split(':')
    .reverse()
    .map((value: string, _index: number, array: string[]) => {
      if (array.length < 1 || array.length > 3) {
        throw new Error('Invalid duration')
      }
      const name = chunkNames.pop()
      const num = Number(value)
      if (isNaN(num) || num < 0 || num > 59) {
        throw new Error(`Invalid ${name}`)
      }
      return num
    })

  return (hours || 0) * 3600 + (minutes || 0) * 60 + seconds
}
