/**
 * Takes a callback function to be called when the theme changes
 * The callback gets the new theme as a string (optional)
 */

export type ThemeMode = "dark" | "light"

export function onThemeChange<T extends (theme: ThemeMode) => void>(callback: T): void {
  const systemPreference = window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light"
  const themeToggle = document.querySelector("starlight-theme-select select")
  if (!themeToggle) return
  themeToggle.addEventListener("change", _ => {
    const newTheme = themeToggle.value === "auto" ? systemPreference : themeToggle.value
    callback(newTheme as ThemeMode)
  })
}
