import * as React from "react"
import { default as MonacoEditor, useMonaco } from "@monaco-editor/react"

export function Editor({
  width = "800px",
  height = "300px",
  fontSize = 24,
  lineNumbers = "off"
}: {
  width?: string
  height?: string
  fontSize?: number
  lineNumbers?: "on" | "off"
}) {
  const [code, setCode] = React.useState("export const foo: string = 'bar'")
  const monaco = useMonaco()

  React.useEffect(() => {
    if (monaco) {
      //
    }
  }, [monaco])

  return (
    <MonacoEditor
      value={code}
      onChange={(value = "") => {
        console.info("Code changed", value)
        setCode(value)
      }}
      width={width}
      height={height}
      theme="vs-dark"
      language="typescript"
      options={{
        fontSize,
        lineNumbers,
        fontLigatures: true,
        padding: { top: 14 },
        scrollBeyondLastColumn: 0,
        fontFamily: "IBM Plex Mono",
        scrollBeyondLastLine: false,
        scrollbar: { vertical: "hidden" }
      }}
      defaultLanguage="typescript"
    />
  )
}
