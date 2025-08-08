import * as Fs from "node:fs"
import * as Path from "node:path"
import {
  Application,
  DeclarationReflection,
  PageKind,
  ReflectionKind,
  RendererEvent,
} from "typedoc"

/**
 * @type {import('typedoc-plugin-frontmatter').load}
 */
export function load(app) {
  console.log("loaded app", app)
  app.renderer.on(RendererEvent.BEGIN, (page) => {
    // We only care about “module” ( = file ) reflections
    // if (page.pageKind !== PageKind.Reflection) {
    //   return
    // }
    console.log("PAGE", JSON.stringify(page, null, 2))
    const model = page.model
    if (
      !(model instanceof DeclarationReflection)
      || !Array.isArray(model.sources)
      || model.sources.length === 0
    ) {
      return // not a page backed by a concrete source file
    }

    const src = model.sources[0]
    const fileName = src.fileName ?? src.fullFileName
    if (!fileName?.endsWith(".ts")) {
      return
    }

    const comment = model.comment
    const summaryText = comment?.summary?.map((p) => p.text).join("").trim() ?? ""

    const badgeBlock = comment?.blockTags?.find((t) => t.tag === "@badge")
    const badgeText = badgeBlock?.content?.map((p) => p.text).join("").trim() ?? "Example"

    Object.assign(page, {
      frontmatter: {
        title: Path.basename(fileName).replace(/\.ts$/, "").replace(/[-_]/g, " "),

        description: page.frontmatter?.description ?? summaryText,

        sidebar: {
          ...(page.frontmatter?.sidebar ?? {}),
          badge: { text: badgeText },
        },
      },
    })

    const summary = comment?.summary?.map(p => p.text).join("").trim() ?? ""
    page.frontmatter = {
      // keep everything the front‑matter plugin already added
      ...page.frontmatter,

      // pull <description> from the first paragraph of the doc‑comment
      description: summary,

      // custom badge from a @badge tag, if present
      sidebar: {
        badge: {
          text: comment?.blockTags?.find(t => t.tag === "@badge")?.content
            ?? "Example",
        },
      },
    }

    /* ---------- 2.  Replace Markdown body with a twoslash block ---- */
    const srcPath = page.model.sources?.[0]?.fileName
    const raw = readFileSync(srcPath, "utf8")
      // escape triple back‑ticks that might appear in code
      .replace(/```/g, "`​`​`")

    page.contents = `\`\`\`ts twoslash\n${raw}\n\`\`\`\n`
  })
}
