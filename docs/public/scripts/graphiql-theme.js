window.addEventListener("DOMContentLoaded", () => {
  try {
    const themeSwitchElement = document.querySelector("starlight-rapide-theme-select")
    if (!themeSwitchElement) return

    const switchButtonElement = themeSwitchElement.children.item(0)
    if (!switchButtonElement) return

    switchButtonElement.addEventListener("click", () => {
      const newTheme = document.documentElement.getAttribute("data-theme")
      const oldTheme = newTheme === "light" ? "dark" : "light"
      if (!newTheme) return

      if (document.body.classList.contains(`graphiql-${oldTheme}`)) {
        document.body.classList.replace(`graphiql-${oldTheme}`, `graphiql-${newTheme}`)
      } else document.body.classList.add(`graphiql-${newTheme}`)

      if (document.documentElement.classList.contains(oldTheme)) {
        document.documentElement.classList.replace(oldTheme, newTheme)
      } else document.documentElement.classList.add(newTheme)
    })
  } catch {
    /* empty */
  }
})
