import { isBrowser } from "./index.ts"

export function updateTheme({ activeTheme, path }: { activeTheme: string; path: string }) {
  if (!isBrowser) return
  document.body.classList.forEach(className => {
    if (className.match(/^theme.*/)) {
      document.body.classList.remove(className)
    }
  })

  const theme = path === "/themes" ? activeTheme : null
  if (theme) return document.body.classList.add(`theme-${theme}`)
}
