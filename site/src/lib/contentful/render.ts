import { documentToHtmlString, type Options } from "@contentful/rich-text-html-renderer"
import { BLOCKS, type Document, MARKS } from "@contentful/rich-text-types"
import type { TermsFields } from "./types.ts"

const defaultOptions: Partial<Options> = {
  renderMark: {
    [MARKS.BOLD]: (text: string) => `<span class="text-accent-500">${text}</span>`,
  },
  renderNode: {
    [BLOCKS.PARAGRAPH]: (node: any, next: any) => `<p>${next(node.content)}</p>`,
    [BLOCKS.DOCUMENT]: (node: any, next: any) => next(node.content).join(""),
  },
}

export const renderRichText = (content: Document, customOptions: Partial<Options> = {}) => {
  const options = { ...defaultOptions, ...customOptions }
  return documentToHtmlString(content, options)
}

export const renderTitle = (title: any) => {
  return title.content
    .map((block: any) => {
      if (block.nodeType === BLOCKS.PARAGRAPH) {
        return block.content
          .map((node: any) => {
            if (node.marks.some((mark: any) => mark.type === MARKS.BOLD)) {
              return `<span class="text-accent-500">${node.value}</span>`
            }
            return node.value
          })
          .join("")
      }
      return ""
    })
    .join("<br>")
}

export const renderTerms = (data: TermsFields) =>
  renderRichText(
    data.copy,
    {
      preserveWhitespace: true,
      renderMark: {
        [MARKS.UNDERLINE]: (text) => `<span class="underline">${text}</span>`,
        [MARKS.ITALIC]: (text) => `<span class="italic">${text}</span>`,
      },
      renderNode: {
        [BLOCKS.HEADING_1]: (node, next) =>
          `<h1 class="text-xl mt-4 mb-4">${next(node.content)}</h1>`,
        [BLOCKS.HEADING_2]: (node, next) =>
          `<h2 class="text-lg mt-3 mb-3 font-bold">${next(node.content)}</h2>`,
        [BLOCKS.PARAGRAPH]: (node, next) =>
          `<p class="mb-4 text-justify">${next(node.content)}</p>`,
      },
    },
  )
