import { writable } from "svelte/store"

const pad = (num: number): string => num.toString().padStart(2, "0")

const formatTime = (date: Date): string => {
  const hours = pad(date.getHours())
  const minutes = pad(date.getMinutes())
  const seconds = pad(date.getSeconds())
  return `${hours}:${minutes}:${seconds}`
}

export const userTime = writable(formatTime(new Date()))

const updateClock = (): void => {
  setInterval(() => {
    userTime.set(formatTime(new Date()))
  }, 1000)
}

updateClock()
