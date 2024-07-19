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
