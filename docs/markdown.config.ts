import {
  transformerNotationDiff,
  transformerMetaHighlight,
  transformerNotationFocus,
  transformerMetaWordHighlight,
  transformerNotationHighlight,
  transformerNotationErrorLevel,
  transformerNotationWordHighlight
} from "@shikijs/transformers"
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
import { rendererRich, transformerTwoslash } from "@shikijs/twoslash"
import { rehypeHeadingIds, type RemarkPlugin, type ShikiConfig } from "@astrojs/markdown-remark"

type Markdown = AstroUserConfig["markdown"]

export const shikiConfig = {
  themes: {
    light: "min-light",
    dark: "houston"
  },
  defaultColor: "dark",
  transformers: [
    transformerTwoslash({
      explicitTrigger: /\btwoslash\b/,
      renderer: rendererRich({ jsdoc: true })
    }),
    transformerNotationDiff(),
    transformerMetaHighlight(),
    transformerNotationFocus(),
    transformerMetaWordHighlight(),
    transformerNotationHighlight(),
    transformerNotationErrorLevel(),
    transformerNotationWordHighlight()
  ]
} satisfies ShikiConfig

export const markdownConfiguration = {
  gfm: true,
  shikiConfig,
  smartypants: false,
  syntaxHighlight: "shiki",
  remarkRehype: {
    clobberPrefix: "union-docs-",
    passThrough: ["code", "root"]
  },
  remarkPlugins: [
    mermaid(),
    remarkMathPlugin,
    remarkSmartypants as RemarkPlugin,
    [remarkToc, { heading: "contents", prefix: "toc-" }]
  ],
  rehypePlugins: [
    rehypeSlug,
    rehypeHeadingIds,
    rehypeKatexPlugin,
    rehypeMathjaxPlugin,
    [rehypeAutolinkHeadings, { behavior: "wrap" }]
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
