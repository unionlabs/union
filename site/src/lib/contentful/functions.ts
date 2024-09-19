import * as contentful from "contentful"
import { env } from "#/lib/constants/env.ts"
import { ContentfulLivePreview } from "@contentful/live-preview"
import { documentToHtmlString } from "@contentful/rich-text-html-renderer"
import { BLOCKS, INLINES, MARKS, type Document } from "@contentful/rich-text-types"

export type ConfigOptions = {
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
  const contentfulClient = contentful.createClient({
    space: env.CONTENTFUL_SPACE_ID,
    host: "preview.contentful.com",
    accessToken: env.CONTENTFUL_ACCESS_TOKEN
  })
  ContentfulLivePreview.init({
    locale,
    debugMode,
    enableLiveUpdates: true,
    enableInspectorMode: true
  })

  contentfulClient
    .getEntry(entryId)
    .then(entry => {
      fields.forEach(fieldId => {
        displayFieldData({ entry, fieldId, entryId })
        setupLivePreview({ entry, fieldId, entryId, subscriptions })
      })
    })
    .catch((error: any) => console.error(`[initializeContentfulLivePreview] error`, error))
}

export function setupLivePreview({
  entry,
  entryId,
  fieldId,
  subscriptions
}: {
  entry: contentful.Entry
  entryId: string
  fieldId: string
  subscriptions: Array<VoidFunction>
}) {
  const callback = (updatedData: any) => {
    console.info("updatedData", updatedData)
    const domElement = findElementByDataAttribute({ entryId, fieldId })
    if (!domElement) return
    console.info(domElement, updatedData.fields, updatedData.fields[fieldId])
    if (domElement && updatedData.fields && updatedData.fields[fieldId]) {
      console.info(updatedData.fields[fieldId])
      // Check if the content is text
      if (typeof updatedData.fields[fieldId] === "string") {
        domElement.textContent = updatedData.fields[fieldId]
      }
      console.info(updatedData.fields[fieldId].nodeType)
      // Check if the content is rich text
      if (updatedData.fields[fieldId].nodeType === "document") {
        const document = updatedData.fields[fieldId]
        if (!document) return
        domElement.innerHTML = documentToHtmlString(document, {
          preserveWhitespace: true,
          renderMark: {
            [MARKS["ITALIC"]]: text => `<em>${text}</em>`,
            [MARKS["UNDERLINE"]]: text => `<u>${text}</u>`,
            [MARKS["CODE"]]: text => `<code>${text}</code>`,
            [MARKS["STRIKETHROUGH"]]: text => `<s>${text}</s>`,
            [MARKS["SUBSCRIPT"]]: text => `<sub>${text}</sub>`,
            [MARKS["BOLD"]]: text => `<strong>${text}</strong>`,
            [MARKS["SUPERSCRIPT"]]: text => `<sup>${text}</sup>`
          },
          renderNode: {
            [BLOCKS["HEADING_1"]]: (node, next) => `<h1>${next(node.content)}</h1>`,
            [BLOCKS["HEADING_2"]]: (node, next) => `<h2>${next(node.content)}</h2>`,
            [BLOCKS["HEADING_3"]]: (node, next) => `<h3>${next(node.content)}</h3>`,
            [BLOCKS["HEADING_4"]]: (node, next) => `<h4>${next(node.content)}</h4>`,
            [BLOCKS["HEADING_5"]]: (node, next) => `<h5>${next(node.content)}</h5>`,
            [BLOCKS["HEADING_6"]]: (node, next) => `<h6>${next(node.content)}</h6>`,
            [BLOCKS["LIST_ITEM"]]: (node, next) => `<li>${next(node.content)}</li>`,
            [BLOCKS["TABLE"]]: (node, next) => `<table>${next(node.content)}</table>`,
            [BLOCKS["TABLE_ROW"]]: (node, next) => `<tr>${next(node.content)}</tr>`,
            [BLOCKS["TABLE_CELL"]]: (node, next) => `<td>${next(node.content)}</td>`,
            [BLOCKS["TABLE_HEADER_CELL"]]: (node, next) => `<th>${next(node.content)}</th>`,
            [BLOCKS["QUOTE"]]: (node, next) => `<blockquote>${next(node.content)}</blockquote>`,
            [BLOCKS["PARAGRAPH"]]: (node, next) =>
              `<p data-contentful-field-id="content" data-contentful-entry-id="${entryId}">${next(node.content)}</p>`,
            [BLOCKS["UL_LIST"]]: (node, next) => `<ul>${next(node.content)}</ul>`,
            [BLOCKS["OL_LIST"]]: (node, next) => `<ol>${next(node.content)}</ol>`,
            [BLOCKS["EMBEDDED_ASSET"]]: asset => {
              const dataFields = asset.data.target.fields
              const imageUrl = imageWithProtocol(dataFields.file.url)
              const { width, height } = dataFields.file.details.image
              return `<Image src="${imageUrl}" alt="${dataFields.title}" width={${width}} height={${height}} />`
            },
            [INLINES["HYPERLINK"]]: params => /* html */ `
                  <a target="_blank" href="${params.data.uri}" rel="noopener noreferrer">${(params.content.at(0) as any)?.value}</a>`,
            [INLINES["ENTRY_HYPERLINK"]]: params => /* html */ `
                    <a target="_blank" href="${params.data.uri}" rel="noopener noreferrer">${(params.content.at(0) as any)?.value}</a>`,
            [INLINES["ASSET_HYPERLINK"]]: params => /* html */ `
                    <a target="_blank" href="${params.data.uri}" rel="noopener noreferrer">${(params.content.at(0) as any)?.value}</a>`
          }
        })
      }
    }
  }
  const unsubscribe = ContentfulLivePreview.subscribe({
    callback,
    data: entry,
    locale: "en-US"
  })

  if (!subscriptions) throw new Error("subscriptions is undefined")
  subscriptions.push(unsubscribe)
}

export function findElementByDataAttribute({
  entryId,
  fieldId
}: {
  entryId: string
  fieldId: string
}) {
  if (typeof document === "undefined") return
  return document.querySelector(
    `[data-contentful-entry-id="${entryId}"][data-contentful-field-id="${fieldId}"]`
  )
}

export const imageWithProtocol = (url: string) => `https:${url}`

export function displayFieldData<T extends contentful.Entry>({
  entry,
  fieldId,
  entryId
}: {
  entry: T
  fieldId: string
  entryId: string
}) {
  const domElement = findElementByDataAttribute({ entryId, fieldId })

  if (!domElement) {
    console.error(`DOM element with entry ID "${entryId}" and field ID "${fieldId}" not found.`)
    return
  }

  if (typeof document === "undefined") return

  const field = entry.fields[fieldId]
  if (!field) {
    console.error(`Field with ID "${fieldId}" not found in entry with ID "${entryId}".`)
    return
  }
  if (typeof field === "string") {
    domElement.textContent = field
  }
  if ((field as Document)?.nodeType === "document") {
    domElement.innerHTML = documentToHtmlString(field as Document, {
      preserveWhitespace: true,
      renderMark: {
        [MARKS["ITALIC"]]: text => `<em>${text}</em>`,
        [MARKS["UNDERLINE"]]: text => `<u>${text}</u>`,
        [MARKS["CODE"]]: text => `<code>${text}</code>`,
        [MARKS["STRIKETHROUGH"]]: text => `<s>${text}</s>`,
        [MARKS["SUBSCRIPT"]]: text => `<sub>${text}</sub>`,
        [MARKS["BOLD"]]: text => `<strong>${text}</strong>`,
        [MARKS["SUPERSCRIPT"]]: text => `<sup>${text}</sup>`
      },
      renderNode: {
        [BLOCKS["HEADING_1"]]: (node, next) => {
          console.info(node)
          return `<h1>${next(node.content)}</h1>`
        },
        [BLOCKS["HEADING_2"]]: (node, next) => `<h2>
                <a href="#${next(node.content)}" id="${next(node.content)}">
                  ${next(node.content)}
                </a>
              </h2>`,
        [BLOCKS["HEADING_3"]]: (node, next) =>
          `<h3>
                <a href="#${next(node.content)}" id="${next(node.content)}">
                  ${next(node.content)}
                </a>
              </h3>`,
        [BLOCKS["HEADING_4"]]: (node, next) =>
          `<h4>
                <a href="#${next(node.content)}" id="${next(node.content)}">
                  ${next(node.content)}
                </a>
              </h4>`,
        [BLOCKS["HEADING_5"]]: (node, next) => `<h5>${next(node.content)}</h5>`,
        [BLOCKS["HEADING_6"]]: (node, next) => `<h6>${next(node.content)}</h6>`,
        [BLOCKS["LIST_ITEM"]]: (node, next) => `<li>${next(node.content)}</li>`,
        [BLOCKS["TABLE"]]: (node, next) => `<table>${next(node.content)}</table>`,
        [BLOCKS["TABLE_ROW"]]: (node, next) => `<tr>${next(node.content)}</tr>`,
        [BLOCKS["TABLE_CELL"]]: (node, next) => `<td>${next(node.content)}</td>`,
        [BLOCKS["TABLE_HEADER_CELL"]]: (node, next) => `<th>${next(node.content)}</th>`,
        [BLOCKS["QUOTE"]]: (node, next) => `<blockquote>${next(node.content)}</blockquote>`,
        [BLOCKS["PARAGRAPH"]]: (node, next) =>
          `<p data-contentful-field-id="content" data-contentful-entry-id="${entryId}">${next(node.content)}</p>`,
        [BLOCKS["UL_LIST"]]: (node, next) => `<ul>${next(node.content)}</ul>`,
        [BLOCKS["OL_LIST"]]: (node, next) => `<ol>${next(node.content)}</ol>`,
        [BLOCKS["EMBEDDED_ASSET"]]: asset => {
          const dataFields = asset.data.target.fields
          const imageUrl = imageWithProtocol(dataFields.file.url)
          const { width, height } = dataFields.file.details.image
          return `<img src="${imageUrl}?fm=avif&w=1366" alt="${dataFields.title}"></img>`
        },
        [INLINES["HYPERLINK"]]: params => /* html */ `
                  <a target="_blank" href="${params.data.uri}" rel="noopener noreferrer">${(params.content.at(0) as any)?.value}</a>`,
        [INLINES["ENTRY_HYPERLINK"]]: params => /* html */ `
                    <a target="_blank" href="${params.data.uri}" rel="noopener noreferrer">${(params.content.at(0) as any)?.value}</a>`,
        [INLINES["ASSET_HYPERLINK"]]: params => /* html */ `
                    <a target="_blank" href="${params.data.uri}" rel="noopener noreferrer">${(params.content.at(0) as any)?.value}</a>`
      }
    })
  }
}
