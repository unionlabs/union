import { derived } from "svelte/store"
import type { CreateQueryResult } from "@tanstack/svelte-query"
import type { Readable } from 'svelte'

export function raise(message?: string, opts?: ErrorOptions): never {
  throw new Error(message, opts)
}

export const noThrow = async <T>(x: Promise<T>): Promise<T | undefined> => x.catch(() => undefined)

export const noThrowSync = <T>(callback: T): T | undefined => {
  try {
    return callback
  } catch {
    return undefined
  }
}

// remove duplicates from an array of objects by a key
export const removeArrayDuplicates = <T>(array: Array<T>, key: keyof T): Array<T> =>
  array.reduce(
    (accumulator, current) => {
      if (!accumulator.find(item => item[key] === current[key])) {
        accumulator.push(current)
      }
      return accumulator
    },
    [] as Array<T>
  )

export const elementHasFocus = (element: Element | null): element is HTMLElement =>
  element === document.activeElement

export const sleep = async (ms: number): Promise<void> =>
  new Promise(resolve => setTimeout(resolve, ms))

export const generateRandomInteger = (min: number, max: number) =>
  Math.floor(Math.random() * (max - min + 1)) + min

export const repeatArray = <T>(array: Array<T>, times: number): Array<T> =>
  Array.from({ length: times }).flatMap(() => array)

export function debounce<T extends (...args: Array<any>) => void>(
  handler: T,
  delay = 500
): (...args: Parameters<T>) => void {
  let id: number
  return (...args: Parameters<T>) => {
    window.clearTimeout(id)
    id = window.setTimeout(handler, delay, ...args)
  }
}

export function throttle(func: Function, limit: number) {
  let inThrottle: boolean
  return (...args: Array<any>) => {
    if (!inThrottle) {
      func(...args)
      inThrottle = true
      setTimeout(() => (inThrottle = false), limit)
    }
  }
}

export async function fetcher<T>(url: string, options?: RequestInit) {
  const response = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
      ...options?.headers
    }
  })
  if (!response.ok) {
    raise(
      `\n ${response.status} - Failed to fetch from ${url}:\n ${
        response.statusText
      }\n ${await response.text()}\n`
    )
  }
  const data = (await response.json()) as T
  return data
}


// TODO: unfortunately does not work, typescript is not smart enough to infer the type, learn how to properly type this
export const readableData = <T>(queryResult: CreateQueryResult<T>): Readable<T> => derived(queryResult, ($queryResult) =>  $queryResult.data  as T)
