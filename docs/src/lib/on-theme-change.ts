/**
 * Takes a callback function to be called when the theme changes
 * The callback gets the new theme as a string (optional)
 */

export type ThemeMode = "dark" | "light"

export function onThemeChange<T extends (theme: ThemeMode) => void>(callback: T): void {
  const systemPreference = window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light"
  const themeToggle = document.querySelector("starlight-rapide-theme-select button")
  if (!themeToggle) return
  themeToggle.addEventListener("click", _ => {
    const newTheme = document.documentElement.getAttribute("data-theme") || systemPreference
    callback(newTheme as ThemeMode)
  })
}
