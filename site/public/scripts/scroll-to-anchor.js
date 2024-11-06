/**
 * This script is used to scroll to anchors in the blog post page.
 */

window.addEventListener(
  "DOMContentLoaded",
  event => {
    setTimeout(() => {
      try {
        const url = new URL(window.location.href)
        const anchorWrapper = document.querySelector(CSS.escape(url.hash).slice(1))
        if (anchorWrapper) anchorWrapper?.scrollIntoView({ behavior: "smooth" })
      } catch {
        /* empty */
      }
    }, 500)
  },
  { once: true }
)
