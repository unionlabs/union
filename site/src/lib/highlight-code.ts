import monochromeTheme from "#/assets/theme/monochrome.json"
import { rehypePrettyCode } from "rehype-pretty-code"
import rehypeStringify from "rehype-stringify"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import { unified } from "unified"

export async function highlightCode(code: string) {
  const file = await unified()
    .use(remarkParse)
    .use(remarkRehype)
    // @ts-expect-error
    .use(rehypePrettyCode, {
      theme: monochromeTheme,
    })
    .use(rehypeStringify)
    .process(code)

  return String(file)
}
