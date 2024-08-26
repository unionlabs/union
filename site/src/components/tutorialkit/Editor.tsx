import type {
  EditorUpdate,
  EditorDocument,
  ScrollPosition
} from "@tutorialkit/components-react/core"
import { useState } from "react"
import { useTheme } from "./hooks/use-theme.ts"
import CodeMirrorEditor from "@tutorialkit/components-react/core/CodeMirrorEditor"

export default function ExampleCodeMirrorEditor() {
  const { editorDocument, theme, onChange, onScroll } = useEditorDocument()

  return (
    <CodeMirrorEditor
      theme={theme}
      doc={editorDocument}
      onChange={onChange}
      onScroll={onScroll}
      debounceChange={500}
      debounceScroll={500}
      className="h-full text-sm"
    />
  )
}

function useEditorDocument() {
  const theme = useTheme()
  const [editorDocument, setEditorDocument] = useState<EditorDocument>(DEFAULT_DOCUMENT)

  function onChange({ content }: EditorUpdate) {
    setEditorDocument(prev => ({
      ...prev,
      value: content
    }))
  }

  function onScroll(scroll: ScrollPosition) {
    setEditorDocument(prev => ({
      ...prev,
      scroll
    }))
  }

  return {
    theme,
    editorDocument,
    onChange,
    onScroll
  }
}

const DEFAULT_DOCUMENT: EditorDocument = {
  filePath: "index.js",
  loading: false,
  value: 'function hello() {\n  console.log("Hello, world!");\n}\n\nhello();'
}
