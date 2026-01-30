import { type ClassValue, clsx } from "clsx"
import type { Snippet } from "svelte"
import type { HTMLAttributes } from "svelte/elements"
import type { TransitionConfig } from "svelte/transition"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null
}

export type WithoutChildren<T> = T extends { children?: unknown } ? Omit<T, "children">
  : T

export type WithoutChildrenOrChild<T> = T extends { children?: unknown; child?: unknown }
  ? Omit<T, "children" | "child">
  : T extends { children?: unknown } ? Omit<T, "children">
  : T extends { child?: unknown } ? Omit<T, "child">
  : T

// Alias for WithoutChildren (used by some components)
export type WithoutChild<T> = T extends { child?: unknown } ? Omit<T, "child">
  : T

type FlyAndScaleParams = {
  y?: number
  x?: number
  start?: number
  duration?: number
}

export const flyAndScale = (
  node: Element,
  params: FlyAndScaleParams = { y: -8, x: 0, start: 0.95, duration: 150 },
): TransitionConfig => {
  const style = getComputedStyle(node)
  const transform = style.transform === "none" ? "" : style.transform

  const scaleConversion = (
    valueA: number,
    scaleA: [number, number],
    scaleB: [number, number],
  ) => {
    const [minA, maxA] = scaleA
    const [minB, maxB] = scaleB

    const percentage = (valueA - minA) / (maxA - minA)
    const valueB = percentage * (maxB - minB) + minB

    return valueB
  }

  const styleToString = (
    style: Record<string, number | string | undefined>,
  ): string => {
    return Object.keys(style).reduce((str, key) => {
      if (style[key] === undefined) {
        return str
      }
      return str + `${key}:${style[key]};`
    }, "")
  }

  return {
    duration: params.duration ?? 200,
    delay: 0,
    css: (t) => {
      const y = scaleConversion(t, [0, 1], [params.y ?? 5, 0])
      const x = scaleConversion(t, [0, 1], [params.x ?? 0, 0])
      const scale = scaleConversion(t, [0, 1], [params.start ?? 0.95, 1])

      return styleToString({
        transform: `${transform} translate3d(${x}px, ${y}px, 0) scale(${scale})`,
        opacity: t,
      })
    },
    easing: (t) => t,
  }
}
