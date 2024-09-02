import {
  transformerNotationDiff,
  transformerNotationFocus,
  transformerMetaHighlight,
  transformerRenderWhitespace,
  transformerNotationHighlight,
  transformerMetaWordHighlight,
  transformerCompactLineOptions,
  transformerNotationErrorLevel,
  transformerRemoveNotationEscape,
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
import { defaultHoverInfoProcessor, rendererRich, transformerTwoslash } from "@shikijs/twoslash"
import { rehypeHeadingIds, type RemarkPlugin } from "@astrojs/markdown-remark"

type Markdown = AstroUserConfig["markdown"]

export const markdownConfiguration = {
  gfm: true,
  smartypants: false,
  syntaxHighlight: "shiki",
  shikiConfig: {
    theme: "tokyo-night",
    transformers: [
      transformerTwoslash({
        throws: false,
        renderer: rendererRich({
          jsdoc: true,
          errorRendering: "line",
          processHoverInfo: info =>
            defaultHoverInfoProcessor(info).replaceAll(/_shikijs_core\w*/g, "")
        })
      }),
      transformerNotationDiff(),
      transformerNotationFocus(),
      transformerMetaHighlight(),
      transformerRenderWhitespace(),
      transformerNotationHighlight(),
      transformerMetaWordHighlight(),
      transformerCompactLineOptions(),
      transformerNotationErrorLevel(),
      transformerRemoveNotationEscape(),
      transformerNotationWordHighlight(),
      {
        name: "shiki:inline-decorations",
        preprocess(code, options) {
          const reg = /^\/\/ @decorations:(.*)\n/
          code = code.replace(reg, (_match, decorations) => {
            options.decorations ||= []
            // @ts-expect-error
            options.decorations.push(...JSON.parse(decorations))
            return ""
          })
          return code
        }
      }
    ]
  },
  remarkPlugins: [
    mermaid(),
    remarkMathPlugin,
    remarkSmartypants as RemarkPlugin,
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
