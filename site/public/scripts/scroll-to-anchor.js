/**
 * This script is used to scroll to anchors in the blog post page.
 */

/** @param {string} selector */
const isElementLoaded = async selector => {
  while (document.querySelector(selector) === null) {
    await new Promise(resolve => requestAnimationFrame(resolve))
  }
  return document.querySelector(selector)
}

window.addEventListener(
  "DOMContentLoaded",
  async _event => {
    try {
      const url = new URL(window.location.href)
      const anchorWrapper = await isElementLoaded(CSS.escape(url.hash).slice(1))
      if (anchorWrapper) anchorWrapper?.scrollIntoView({ behavior: "smooth" })
    } catch {
      /* empty */
    }
  },
  { once: true }
)
