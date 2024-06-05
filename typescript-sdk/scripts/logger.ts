import { createConsola } from "consola"

export const consola = createConsola({
  formatOptions: {
    date: true,
    colors: true
  }
})

export function timestamp() {
  const d = new Date()
  const [date] = d.toISOString().split("T")
  const [time] = d.toTimeString().split(" ")
  return `${date}--${time?.replace(/:/g, "-")}`
}
