import { map } from "nanostores"

export interface Query {
  result: string
}
export const $queryStore = map<Query>({
  result: ""
})
