/**
 * detect which browser is being used
 */

import { readable } from "svelte/store"

const browsers = ["safari", "arc", "chrome", "brave", "firefox", "edge", "opera"] as const
type Browser = (typeof browsers)[number]

export interface BrowserDetection {
  detected: boolean
  browser: Browser | undefined
}

export function detectBrowser(): BrowserDetection {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return { detected: false, browser: undefined }
  }

  if (isArc()) return { detected: true, browser: "arc" }
  if (isEdge()) return { detected: true, browser: "edge" }
  if (isBrave()) return { detected: true, browser: "brave" }
  if (isOpera()) return { detected: true, browser: "opera" }
  if (isSafari()) return { detected: true, browser: "safari" }
  if (isChrome()) return { detected: true, browser: "chrome" }
  if (isFirefox()) return { detected: true, browser: "firefox" }

  return { detected: false, browser: undefined }
}

function isChrome(): boolean {
  if (typeof window === "undefined" || isOpera() || isEdge()) return false
  if (typeof window?.chrome?.app === "object") return true
  return window?.navigator?.userAgent?.indexOf("Chrome") > -1
}

function isEdge(): boolean {
  if (!window?.navigator?.userAgent) return false
  return window.navigator.userAgent.indexOf("Edg") > -1
}

function isFirefox(): boolean {
  // @ts-expect-error
  if (typeof InstallTrigger !== "undefined") return true
  if (navigator?.userAgent?.indexOf("Firefox") !== -1) return true
  return false
}

function isBrave(): boolean {
  if (typeof navigator?.brave !== "object") return false
  if (typeof navigator?.brave.isBrave === "function") return true
  return false
}

function isArc(): boolean {
  return (
    getComputedStyle(document.documentElement)
      //
      .getPropertyValue("--arc-palette-title").length > 0
  )
}

function isSafari(): boolean {
  const userAgent = navigator.userAgent.toLowerCase()
  const iOS =
    userAgent.match(/Macintosh/i) || userAgent.match(/iPad/i) || userAgent.match(/iPhone/i)
  const webket = userAgent.match(/WebKit/i)
  const iOSSafari =
    iOS &&
    webket &&
    !userAgent.match(/CriOS/i) &&
    !userAgent.match(/EdgiOS/i) &&
    !userAgent.match(/Chrome/i) &&
    !userAgent.match(/Edg/i)
  return Boolean(iOSSafari)
}

function isOpera(): boolean {
  return Boolean(window?.opera || window?.opr)
}

export const currentBrowser = readable<BrowserDetection>(detectBrowser())
