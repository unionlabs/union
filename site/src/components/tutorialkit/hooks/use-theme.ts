import { useState, useEffect } from "react"

export function useTheme() {
  const [theme, setTheme] = useState<"dark" | "light">(getThemeFromRoot())

  useEffect(() => {
    const observer = new MutationObserver(() => {
      setTheme(getThemeFromRoot())
    })

    observer.observe(window.document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"]
    })

    return () => observer.disconnect()
  }, [])

  return theme
}

function getThemeFromRoot() {
  return (
    (globalThis.document?.documentElement.getAttribute("data-theme") as "dark" | "light") ?? "light"
  )
}
