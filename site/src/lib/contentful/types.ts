import type { EntryFields, EntrySkeletonType } from "contentful"

export interface LandingPageFields {
  firstTitle: EntryFields.RichText
  firstText: EntryFields.RichText
  secondTitle: EntryFields.RichText
  secondText: EntryFields.RichText
  thirdTitle: EntryFields.RichText
  thirdText: EntryFields.RichText
  fourthTitle: EntryFields.RichText
  fourthText: EntryFields.RichText
}

export interface LearningPageFields {
  coverTitle: EntryFields.RichText
  coverText: EntryFields.RichText
  firstTitle: EntryFields.RichText
  firstText: EntryFields.RichText
  secondTitle: EntryFields.RichText
  secondText: EntryFields.RichText
  thirdTitle: EntryFields.RichText
  thirdText: EntryFields.RichText
  fourthTitle: EntryFields.RichText
  fourthText: EntryFields.RichText
  fifthTitle: EntryFields.RichText
  fifthText: EntryFields.RichText
  sixthTitle: EntryFields.RichText
  sixthText: EntryFields.RichText
  seventhTitle: EntryFields.RichText
  seventhText: EntryFields.RichText
  eightTitle: EntryFields.RichText
  eightText: EntryFields.RichText
  nineTitle: EntryFields.RichText
  nineText: EntryFields.RichText
}

export interface TeamPageFields {
  name: EntryFields.Text
  title: EntryFields.Text
  twitterHandle: EntryFields.Text
  profilePicture: EntryFields.AssetLink
  position: EntryFields.Number
}

export interface EcosystemFields {
  name: EntryFields.Text
  url: EntryFields.Text
  background: EntryFields.AssetLink
  logo: EntryFields.AssetLink
  category: Array<CategoryFields>
}

export interface CategoryFields {
  fields: {
    category: EntryFields.Text
  }
}

export interface EcosystemSkeleton extends EntrySkeletonType<EcosystemFields, "ecosystem"> {}

export interface CategorySkeleton extends EntrySkeletonType<CategoryFields, "category"> {}

export interface TeamPageSkeleton extends EntrySkeletonType<TeamPageFields, "team"> {}

export interface LandingPageSkeleton extends EntrySkeletonType<LandingPageFields, "landing"> {}

export interface LearningPageFields extends EntrySkeletonType<LearningPageFields, "learn"> {}

export type FetchError = string | null
