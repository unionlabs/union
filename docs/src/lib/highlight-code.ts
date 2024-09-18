import { unified } from "unified"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import rehypeStringify from "rehype-stringify"
import { rehypePrettyCode } from "rehype-pretty-code"

// let highlighter: Awaited<ReturnType<typeof createHighlighter>> | undefined

export type BuiltinLang =
  | "ts"
  | "typescript"
  | "jsx"
  | "sh"
  | "plaintext"
  | "toml"
  | "json"
  | "yaml"
  | "bash"

export type HighlightArgs = {
  code: string
  lang?: BuiltinLang
}

export async function highlightCode({ code, lang = "typescript" }: HighlightArgs): Promise<string> {
  const file = await unified()
    .use(remarkParse)
    .use(remarkRehype)
    // .use(rehypeShiki, {
    //   theme: "houston",
    //   // transformers: [
    //   //   transformerTwoslash({
    //   //     renderer: rendererRich({ jsdoc: true }),
    //   //     twoslashOptions: {
    //   //       compilerOptions: {
    //   //         ...defaultCompilerOptions,
    //   //         noErrorTruncation: true,
    //   //         exactOptionalPropertyTypes: true
    //   //       }
    //   //     }
    //   //   }),
    //   //   transformerNotationDiff(),
    //   //   transformerMetaHighlight(),
    //   //   transformerNotationFocus(),
    //   //   transformerRenderWhitespace(),
    //   //   transformerNotationHighlight(),
    //   //   transformerMetaWordHighlight(),
    //   //   transformerNotationErrorLevel(),
    //   //   transformerCompactLineOptions(),
    //   //   transformerRemoveNotationEscape(),
    //   //   transformerNotationWordHighlight()
    //   // ]
    // })
    .use(rehypePrettyCode, {
      theme: "houston",
      grid: true,
      keepBackground: true
      // transformers: [transformerTwoslash({ renderer: rendererRich() })]
    })
    .use(rehypeStringify)
    .process(`\`\`\`${lang}\n${code}\n\`\`\``)
  return String(file)

  // highlighter ??= await createHighlighter({
  //   themes: ["houston"],
  //   langs: [lang, "typescript"]
  // })

  // return highlighter.codeToHtml(code, {
  //   lang,
  //   theme: "houston",
  //   transformers: [
  //     transformerTwoslash({
  //       renderer: rendererRich()
  //     }),
  //     transformerNotationDiff(),
  //     transformerMetaHighlight(),
  //     transformerNotationFocus(),
  //     transformerRenderWhitespace(),
  //     transformerNotationHighlight(),
  //     transformerMetaWordHighlight(),
  //     transformerNotationErrorLevel(),
  //     transformerCompactLineOptions(),
  //     transformerRemoveNotationEscape(),
  //     transformerNotationWordHighlight()
  //   ]
  // })
}
