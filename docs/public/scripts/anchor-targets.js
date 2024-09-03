/**
 * Add target="_blank" to all external links
 */

window.addEventListener("DOMContentLoaded", () => {
  const externalLinks = document.querySelectorAll('a[href^="http"]')
  try {
    for (const link of externalLinks) {
      link.setAttribute("target", "_blank")
      link.setAttribute("rel", "noopener noreferrer")
    }
  } catch {
    /* empty */
  }
})
