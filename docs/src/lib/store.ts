import * as n from "nanostores"



export interface PreProcessedData<T> {
  data: T
  error: Error | null
}
