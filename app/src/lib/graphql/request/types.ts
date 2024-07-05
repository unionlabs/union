import type { DocumentNode } from 'graphql'

export type RequestDocument = string | DocumentNode
export type RemoveIndex<T> = {
  [K in keyof T as string extends K ? never : number extends K ? never : K]: T[K]
}
export interface DocumentTypeDecoration<TResult, TVariables> {
  /**
   * This type is used to ensure that the variables you pass in to the query are assignable to Variables
   * and that the Result is assignable to whatever you pass your result to. The method is never actually
   * implemented, but the type is valid because we list it as optional
   */
  __apiType?: (variables: TVariables) => TResult
}

export type Variables = object

export interface TypedDocumentNode<
  TResult = {
    [key: string]: any
  },
  TVariables = {
    [key: string]: any
  }
> extends DocumentNode,
    DocumentTypeDecoration<TResult, TVariables> {}

export type VariablesAndRequestHeadersArgs<V extends Variables> = V extends Record<any, never> // do we have explicitly no variables allowed?
  ? [variables?: V, requestHeaders?: HeadersInit]
  : keyof RemoveIndex<V> extends never // do we get an empty variables object?
    ? [variables?: V, requestHeaders?: HeadersInit]
    : [variables: V, requestHeaders?: HeadersInit]
