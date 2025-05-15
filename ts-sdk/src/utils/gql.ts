import { Array as A, Match as M, Option as O, pipe } from "effect"
import { TadaDocumentNode } from "gql.tada"

export const documentNodeToAnnotations = (doc: TadaDocumentNode<any, any, any>) =>
  pipe(
    M.value(doc),
    M.when(
      { definitions: M.defined },
      ({ definitions }) =>
        pipe(
          definitions,
          A.filter((x) => x.kind === "OperationDefinition"),
          A.map(x => ({
            name: x.name?.value ?? "NO_NAME",
            op: x.operation,
          })),
        ),
    ),
    M.option,
    O.map(a => {
      let ops: string[] = []
      let names: string[] = []

      a.forEach(x => {
        ops.push(x.op)
        names.push(x.name)
      })

      return { ops, names }
    }),
  )
