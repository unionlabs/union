import * as React from "react"
import ts from "shiki/langs/typescript.mjs"
import { shikiToMonaco } from "@shikijs/monaco"
import { createHighlighterCoreSync } from "shiki"
import tokyoNight from "shiki/themes/tokyo-night.mjs"
import type { FileSystemTree } from "@webcontainer/api"
import { createJavaScriptRegexEngine } from "shiki/engine/javascript"
import { default as MonacoEditor, type OnMount, useMonaco } from "@monaco-editor/react"

type OnMountParameters = Parameters<OnMount>

export type Packages = {
  url: string
  name: string
}

// Create the highlighter, it can be reused
const highlighter = createHighlighterCoreSync({
  langs: [ts],
  themes: [tokyoNight],
  engine: createJavaScriptRegexEngine()
})

export function Editor({
  options,
  dependencies
}: {
  options: {
    width?: string
    height?: string
    fontSize?: number
    className?: string
    initialCode?: string
    lineNumbers?: "on" | "off"
  }
  dependencies: Array<Packages>
}) {
  const { width, height, fontSize, className, initialCode, lineNumbers } = options
  const monaco = useMonaco()
  const [code, setCode] = React.useState(initialCode)
  const editorRef = React.useRef<monaco.editor.IStandaloneCodeEditor | null>(null)

  React.useEffect(() => {
    if (monaco) {
      // Register the languageIds first. Only registered languages will be highlighted.
      monaco.languages.register({ id: "typescript" })

      // Register the highlight provider for the languageIds
      shikiToMonaco(highlighter, monaco)
    }
    const wcFiles = localStorage.getItem("webcontainer:files")

    if (wcFiles) {
      const files = JSON.parse(wcFiles) as FileSystemTree
      // @ts-expect-error
      const content = (files["mod.ts"].file as any).contents.toString()
      setCode(`${content}\n`)
    }
  }, [monaco])

  return (
    <MonacoEditor
      onMount={(editor, _monaco) => {
        const wcFiles = localStorage.getItem("webcontainer:files")
        if (!wcFiles) {
          return handleEditorDidMount(code ? `${code}\n` : "", editor, _monaco, dependencies)
        }

        const files = JSON.parse(wcFiles) as FileSystemTree
        // @ts-expect-error
        const content = (files["mod.ts"].file as any).contents.toString()

        handleEditorDidMount(content.concat("\n"), editor, _monaco, dependencies)
      }}
      onChange={(value, _event) => setCode(value || "")}
      value={code?.concat(code.endsWith("\n") ? "" : "\n")}
      onValidate={markers => console.info("Validation", markers)}
      beforeMount={monaco => {
        monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
          noEmit: true,
          strict: true,
          allowJs: true,
          checkJs: true,
          skipLibCheck: true,
          esModuleInterop: true,
          isolatedModules: true,
          resolveJsonModule: true,
          allowNonTsExtensions: true,
          forceConsistentCasingInFileNames: true,
          jsx: monaco.languages.typescript.JsxEmit.React,
          module: monaco.languages.typescript.ModuleKind.ESNext,
          target: monaco.languages.typescript.ScriptTarget.ESNext,
          moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
          lib: [
            "lib.dom.d.ts",
            "lib.dom.iterable.d.ts",
            "lib.esnext.d.ts",
            "lib.esnext.full.d.ts",
            "lib.esnext.intl.d.ts",
            "lib.esnext.string.d.ts",
            "lib.esnext.weakref.d.ts",
            "lib.scripthost.d.ts",
            "lib.webworker.d.ts"
          ],
          // ../
          types: ["node", "react"]
        })
      }}
      width={width}
      height={height}
      language="typescript"
      className={className}
      defaultLanguage="typescript"
      options={{
        fontSize,
        lineNumbers,
        fontLigatures: true,
        disableLayerHinting: true,
        automaticLayout: true,
        padding: { top: 14 },
        scrollBeyondLastColumn: 0,
        minimap: { enabled: false },
        // fontFamily: "",
        scrollBeyondLastLine: false,
        scrollbar: { vertical: "hidden" }
      }}
    />
  )
}

function handleEditorDidMount(
  code: string,
  editor: OnMountParameters[0],
  monaco: OnMountParameters[1],
  dependenciesUrls: Array<Packages>
) {
  if (monaco) {
    monaco.languages.typescript.javascriptDefaults.setDiagnosticsOptions({
      noSyntaxValidation: false,
      noSemanticValidation: true
    })
    monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
      target: monaco.languages.typescript.ScriptTarget.Latest,
      allowNonTsExtensions: true,
      moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
      module: monaco.languages.typescript.ModuleKind.ESNext,
      noEmit: true,
      skipLibCheck: true,
      esModuleInterop: true,
      jsx: monaco.languages.typescript.JsxEmit.React,
      reactNamespace: "React",
      allowJs: true,
      types: ["node", "react"]
    })

    Promise.all(
      dependenciesUrls.map(({ url, name }) =>
        fetch(url).then(res =>
          res.text().then(content => ({
            content,
            name,
            url
          }))
        )
      )
    ).then(responses => {
      const libraries = responses.map(({ content, name }) => {
        return {
          content,
          filePath:
            name === "@types/node"
              ? `file:///node_modules/@types/node/index.d.ts`
              : `file:///node_modules/@types/${name.startsWith("@") ? name.slice(1).replace("/", "__") : name}/index.d.ts`
        }
      })
      console.info(monaco.languages.typescript.typescriptDefaults.getExtraLibs())
      libraries.map(library =>
        monaco.languages.typescript.typescriptDefaults.addExtraLib(
          library.content,
          library.filePath
        )
      )
    })
  }
  const model = monaco.editor.createModel(code, "typescript", monaco.Uri.file("mod.ts"))
  editor.setModel(model)
}
