import { contentfulClient } from "./client.ts"
import { ContentfulLivePreview } from "@contentful/live-preview"

type ConfigOptions = {
  locale: string
  entryId: string
  debugMode: boolean
  fields: Array<string>
  subscriptions: Array<VoidFunction>
}

export function initializeContentfulLivePreview({
  locale,
  fields,
  entryId,
  debugMode,
  subscriptions
}: ConfigOptions) {
  ContentfulLivePreview.init({
    locale,
    debugMode,
    enableLiveUpdates: true,
    enableInspectorMode: true,
    targetOrigin: "https://app.contentful.com"
  })

  contentfulClient
    .getEntry(entryId)
    .then(entry => {
      console.info("[initializeContentfulLivePreview.contentful]", entry)
      fields.forEach(fieldId => {
        displayFieldData({ entry, client: contentfulClient, fieldId, entryId })
        setupLivePreview({ entry, fieldId, entryId, subscriptions })
      })
    })
    .catch(error => console.error(`[initializeContentfulLivePreview]`, error))
}

export function setupLivePreview({
  entry,
  entryId,
  fieldId,
  subscriptions
}: {
  entry: any
  entryId: string
  fieldId: string
  subscriptions: Array<VoidFunction>
}) {
  const callback = (updatedData: any) => {
    const domElement = findElementByDataAttribute({ entryId, fieldId })
    if (domElement && updatedData.fields && updatedData.fields[fieldId]) {
      // Check if the content is text
      if (typeof updatedData.fields[fieldId] === "string") {
        domElement.textContent = updatedData.fields[fieldId]
      }
    }
  }
  const unsubscribe = ContentfulLivePreview.subscribe({
    callback,
    data: entry,
    locale: "en-US"
  })

  subscriptions.push(unsubscribe)
}

function findElementByDataAttribute({ entryId, fieldId }: { entryId: string; fieldId: string }) {
  if (typeof document === "undefined") return
  return document.querySelector(`[data-entry-id="${entryId}"][data-field-id="${fieldId}"]`)
}

function displayFieldData({
  entry,
  client: _client,
  fieldId,
  entryId
}: {
  entry: any
  client: any
  fieldId: string
  entryId: string
}) {
  const domElement = findElementByDataAttribute({ entryId, fieldId })

  if (!domElement) {
    console.error(`DOM element with entry ID "${entryId}" and field ID "${fieldId}" not found.`)
    return
  }

  if (typeof document === "undefined") return
  domElement.textContent = entry.fields[fieldId]
}
