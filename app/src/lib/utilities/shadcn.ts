import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"
import { cubicOut } from "svelte/easing"
import type { TransitionConfig } from "svelte/transition"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

type FlyAndScaleParameters = {
  y?: number
  x?: number
  start?: number
  duration?: number
}

export const flyAndScale = (
  node: Element,
  parameters: FlyAndScaleParameters = {
    y: -8,
    x: 0,
    start: 0.95,
    duration: 150
  }
): TransitionConfig => {
  const style = getComputedStyle(node)
  const transform = style.transform === "none" ? "" : style.transform

  const scaleConversion = (valueA: number, scaleA: [number, number], scaleB: [number, number]) => {
    const [minA, maxA] = scaleA
    const [minB, maxB] = scaleB

    const percentage = (valueA - minA) / (maxA - minA)
    const valueB = percentage * (maxB - minB) + minB

    return valueB
  }

  const styleToString = (style: Record<string, number | string | undefined>): string => {
    return Object.keys(style).reduce((string_, key) => {
      if (style[key] === undefined) return string_
      return string_ + `${key}:${style[key]};`
    }, "")
  }

  return {
    duration: parameters.duration ?? 200,
    delay: 0,
    css: t => {
      const y = scaleConversion(t, [0, 1], [parameters.y ?? 5, 0])
      const x = scaleConversion(t, [0, 1], [parameters.x ?? 0, 0])
      const scale = scaleConversion(t, [0, 1], [parameters.start ?? 0.95, 1])

      return styleToString({
        transform: `${transform} translate3d(${x}px, ${y}px, 0) scale(${scale})`,
        opacity: t
      })
    },
    easing: cubicOut
  }
}
