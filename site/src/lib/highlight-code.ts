import { unified } from "unified"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import rehypeStringify from "rehype-stringify"
import rehypePrettyCode from "rehype-pretty-code"
import monochromeTheme from "#/assets/theme/monochrome.json"

export async function highlightCode(code: string) {
  const file = await unified()
    .use(remarkParse)
    .use(remarkRehype)
    // @ts-expect-error
    .use(rehypePrettyCode, {
      theme: monochromeTheme
    })
    .use(rehypeStringify)
    .process(code)

  return String(file)
}
