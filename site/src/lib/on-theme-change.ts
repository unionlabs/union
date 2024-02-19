/**
 * Takes a callback function to be called when the theme changes
 * The callback gets the new theme as a string (optional)
 */

export function onThemeChange<T extends (theme: "dark" | "light") => void>(
  callback: T
): void {
  const themeToggle = document.querySelector("starlight-theme-select select");
  if (!themeToggle) return;
  themeToggle.addEventListener("change", (_event) => {
    callback(themeToggle.value as "dark" | "light");
  });
}
