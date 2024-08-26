import { useState } from "react"
import FileTree from "@tutorialkit/components-react/core/FileTree"

export default function ExampleFileTree() {
  const [selectedFile, setSelectedFile] = useState(FILES[0])

  return (
    <FileTree
      files={FILES}
      hideRoot={true}
      className="my-file-tree"
      selectedFile={selectedFile}
      onFileSelect={setSelectedFile}
      hiddenFiles={["package-lock.json"]}
    />
  )
}

const FILES = [
  "/src/index.js",
  "/src/index.html",
  "/src/assets/logo.svg",
  "/package-lock.json",
  "/package.json",
  "/vite.config.js"
]
