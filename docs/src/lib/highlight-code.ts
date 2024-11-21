import { unified } from "unified"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import rehypeStringify from "rehype-stringify"
import { rehypePrettyCode } from "rehype-pretty-code"
import { transformerCopyButton } from "@rehype-pretty/transformers"

export async function highlightCode(code: string) {
  const file = await unified()
    .use(remarkParse)
    .use(remarkRehype)
    .use(rehypePrettyCode, {
      grid: true,
      keepBackground: true,
      theme: {
        dark: "houston",
        light: "github-light"
      },
      transformers: [
        transformerCopyButton({
          visibility: "always",
          feedbackDuration: 3_000
        })
      ]
    })
    .use(rehypeStringify)
    .process(code)

  return String(file)
}
