import { unified } from "unified"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import rehypeStringify from "rehype-stringify"
import { rehypePrettyCode } from "rehype-pretty-code"

export async function highlightCode(code: string) {
  const file = await unified()
    .use(remarkParse)
    .use(remarkRehype)
    .use(rehypePrettyCode, {
      theme: "houston"
    })
    .use(rehypeStringify)
    .process(code)

  return String(file)
}
