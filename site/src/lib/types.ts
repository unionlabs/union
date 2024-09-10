import type { EntryFieldTypes } from "contentful"
import { type ImageMetadata } from 'astro';

export type MaybePromise<T> = T | Promise<T>

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P]
}

export type NamedImage = {
  name: string, 
  url: string, 
  logo: ImageMetadata
}

export interface BlogPost {
  contentTypeId: "blog"
  fields: {
    date: EntryFieldTypes.Text
    slug: EntryFieldTypes.Text
    title: EntryFieldTypes.Text
    author: EntryFieldTypes.Text
    hidden: EntryFieldTypes.Boolean
    cover: EntryFieldTypes.AssetLink
    content: EntryFieldTypes.RichText
    description: EntryFieldTypes.Text
  }
}
