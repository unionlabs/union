import remarkToc from "remark-toc"
import rehypeSlug from "rehype-slug"
import { visit } from "unist-util-visit"
import remarkMathPlugin from "remark-math"
import rehypeKatexPlugin from "rehype-katex"
import rehypeMathjaxPlugin from "rehype-mathjax"
import remarkSmartypants from "remark-smartypants"
import type { AstroUserConfig } from "astro/config"
import { escapeHTML } from "astro/runtime/server/escape.js"
import rehypeAutolinkHeadings from "rehype-autolink-headings"
import { rehypeHeadingIds, type RemarkPlugin } from "@astrojs/markdown-remark"

type Markdown = AstroUserConfig["markdown"]

export const markdownConfiguration = {
  gfm: true,
  smartypants: false,
  remarkPlugins: [
    mermaid(),
    remarkMathPlugin,
    remarkSmartypants,
    [remarkToc, { heading: "contents", prefix: "toc-" }]
  ],
  rehypePlugins: [
    rehypeHeadingIds,
    rehypeSlug,
    [rehypeAutolinkHeadings, { behavior: "wrap" }],
    rehypeKatexPlugin,
    rehypeMathjaxPlugin
  ]
} satisfies Markdown

export function mermaid(): RemarkPlugin<Array<any>> {
  return () => tree => {
    visit(tree, "code", node => {
      if (node.lang !== "mermaid") return
      // @ts-expect-error
      node.type = "html"
      node.value = /* html */ `<div class="mermaid">${escapeHTML(node.value)}</div>`
    })
  }
}
