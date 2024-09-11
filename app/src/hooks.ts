import type { Reroute } from "@sveltejs/kit"

const redirects: Array<[path: RegExp, to: string]> = [
  [/^\/explorer\/?$/, "/"],
  [/^\/explorer\/transfers\/?$/, "/"],
  [/^\/explorer\/transfers\/.*$/, "/*"]
]

/**
 * the reroute hook returns a string, the path to redirect to
 */
export const reroute = (event => {
  const url = new URL(event.url)
  const pathname = url.pathname
  console.info(`current pathname ${url.pathname}`)

  for (const [path, to] of redirects) {
    const match = path.exec(pathname)
    console.info(`hooks.ts ${path.toString()} ${match}`)
    if (match && !to.endsWith("*")) return to

    if (match && to.endsWith("*")) {
      // redirect to the value of the last path segment in the `*` in the path
      const toPath = to
        .replace("/*", match[match.length - 1])
        .split("/")
        .at(-1)
      console.info(`redirecting to ${toPath}`)
      return `/${toPath}`
    }
  }
}) satisfies Reroute
