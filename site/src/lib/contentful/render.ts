import { BLOCKS, MARKS } from "@contentful/rich-text-types"
import { documentToHtmlString } from "@contentful/rich-text-html-renderer"

const defaultOptions = {
  renderMark: {
    [MARKS.BOLD]: (text: string) => `<span class="text-accent-500">${text}</span>`
  },
  renderNode: {
    [BLOCKS.PARAGRAPH]: (node: any, next: any) => `<p>${next(node.content)}</p>`,
    [BLOCKS.DOCUMENT]: (node: any, next: any) => next(node.content).join("")
  }
}

export const renderRichText = (content: any, customOptions: any = {}) => {
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
