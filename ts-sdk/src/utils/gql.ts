import { Array as A, flow, Option as O, pipe } from "effect"
import { TadaDocumentNode } from "gql.tada"

/** @internal */
export const operationNamesFromDocumentNode = <T extends TadaDocumentNode<any, any>>(doc: T) =>
  pipe(
    doc.definitions,
    A.filter(x => x.kind === "OperationDefinition"),
    A.map(flow((x) => x.name?.value, O.fromNullable)),
    A.getSomes,
  )
