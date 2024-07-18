import type { EntryFieldTypes } from "contentful"

export interface BlogPost {
  contentTypeId: "blog"
  fields: {
    date: EntryFieldTypes.Text
    slug: EntryFieldTypes.Text
    title: EntryFieldTypes.Text
    author: EntryFieldTypes.Text
    cover: EntryFieldTypes.AssetLink
    content: EntryFieldTypes.RichText
    description: EntryFieldTypes.Text
  }
}
