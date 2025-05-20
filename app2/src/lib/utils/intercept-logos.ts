/**
 * Intercepts all requests to raw.githubusercontent.com and rewrites them
 * to use the Cloudflare-cached logo CDN instead. This affects:
 * - fetch()
 * - XMLHttpRequest
 * - <img src="...">
 *
 * This function should be run once on the client (e.g., in a layout or onMount).
 * It is idempotent and guards against multiple executions via a global flag.
 */
export const interceptLogos = () => {
  const RAW = "https://raw.githubusercontent.com/"
  const CDN = "https://cache-logos.unionlabs.workers.dev/"

  // Ensure this runs only in the browser
  if (typeof window === "undefined") {
    return
  }

  // Prevent patching more than once
  if (window.__githubLogoProxyPatched) {
    return
  }
  window.__githubLogoProxyPatched = true

  /**
   * Patch global fetch to rewrite raw.githubusercontent.com URLs
   */
  const originalFetch = window.fetch
  window.fetch = (input, init) => {
    if (typeof input === "string" && input.startsWith(RAW)) {
      input = input.replace(RAW, CDN)
    } else if (input instanceof Request && input.url.startsWith(RAW)) {
      input = new Request(input.url.replace(RAW, CDN), input)
    }
    return originalFetch(input, init)
  }

  /**
   * Patch XMLHttpRequest.open to rewrite raw.githubusercontent.com URLs
   */
  const originalOpen = XMLHttpRequest.prototype.open
  XMLHttpRequest.prototype.open = function(
    method: string,
    url: string | URL,
    async = true,
    username?: string | null,
    password?: string | null,
  ) {
    if (typeof url === "string" && url.startsWith(RAW)) {
      url = url.replace(RAW, CDN)
    }
    return originalOpen.call(this, method, url, Boolean(async), username, password)
  }

  /**
   * Patch HTMLImageElement.setAttribute to rewrite <img src="..."> URLs
   */
  const originalSetAttr = HTMLImageElement.prototype.setAttribute
  HTMLImageElement.prototype.setAttribute = function(attr: string, value: string) {
    if (attr === "src" && typeof value === "string" && value.startsWith(RAW)) {
      value = value.replace(RAW, CDN)
    }
    return originalSetAttr.call(this, attr, value)
  }
}
