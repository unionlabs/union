// https://stackoverflow.com/a/17415677
export function toIsoString(date: Date) {
  const pad = (num: number) => (num < 10 ? "0" : "") + num

  // @ts-ignore
  return (
    // biome-ignore lint/style/useTemplate: would be illegible
    date.getFullYear() +
    "-" +
    pad(date.getMonth() + 1) +
    "-" +
    pad(date.getDate()) +
    "T" +
    pad(date.getHours()) +
    ":" +
    pad(date.getMinutes()) +
    ":" +
    pad(date.getSeconds())
  )
}
