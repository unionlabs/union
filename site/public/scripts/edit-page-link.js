/**
 * "Edit page" link attaches path to link but we want only the discord link
 */

window.addEventListener("DOMContentLoaded", () => {
  const editPageLinkElements = document.querySelectorAll(
    'footer a[href*="discord"]'
  );
  try {
    for (const link of editPageLinkElements) {
      if (!link) continue;
      link.setAttribute("href", "https://discord.union.build");
    }
  } catch {
    /* empty */
  }
});
