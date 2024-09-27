import { env } from "#/lib/constants/env.ts"

export const graphqlUrl = `https://graphql.contentful.com/content/v1/spaces/${env.CONTENTFUL_SPACE_ID}/environments/master`

export type RssItemsResponse = {
  data: {
    blogCollection: {
      items: Array<{
        _id: string
        date: string
        slug: string
        title: string
        author: string
        hidden: boolean
        description: string
      }>
    }
  }
}

export const rssItemsQuery = /* GraphQL */ `
  query RssItems($limit: Int = 100) {
    blogCollection(
      limit: $limit,
      order: date_DESC,
      where: { hidden: false }
    ) {
      items {
        _id
        date
        slug
        title
        author
        hidden
        description
      }
    }
  }
`

export const blogPostsQuery = /* GraphQL */ `
  query BlogPosts($limit: Int = 100) {
    blogCollection(
      limit: $limit,
      order: date_DESC,
    ) {
      items {
        _id
        date
        slug
        title
        author
        description
        cover {
          url
          width
          height
        }
        content { json }
      }
    }
  }
`

export const blogPostQuery = /* GraphQL */ `
  query BlogPost(
    $slug: String!,
    $preview: Boolean = true
  ) {
    blogCollection(
      limit: 1,
      preview: $preview,
      where: { slug: $slug },
    ) {
      items {
        date
        slug
        title
        author
        description
        cover {
          url
          width
          height
        }
        content { json }
      }
    }
  }
`
