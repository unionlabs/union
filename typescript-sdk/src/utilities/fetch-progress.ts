import type { MaybePromise } from "../types.ts"

/**
 * WIP
 */
export async function onResponse({
  response,
  onProgress
}: {
  response: Response
  onProgress: (progress: number) => MaybePromise<void>
}) {
  console.info(`Response status: ${response.status}`)

  const reader = response.body?.getReader()
  if (!reader) return

  // Ensure the header name matches exactly as it was set
  const contentLengthHeader = response.headers.get("content-length")
  const length = contentLengthHeader ? +Number.parseInt(contentLengthHeader) : undefined

  let receivedLength = 0
  let chunks: Array<Uint8Array> = []

  while (true) {
    console.info("Reading...")

    const read = await reader.read()
    if (read.done || !read.value) break

    receivedLength += read.value.length
    chunks.push(read.value)

    // Calculate progress as a percentage if length is defined
    const progress = length ? (receivedLength / length) * 100 : receivedLength
    await onProgress(progress)

    console.info(`Received ${receivedLength} of ${length} bytes`)
  }
}
