import { toPrettyDateTimeFormat } from "$lib/utilities/date"

export const encodeTimestampSearchParam = (timestamp: string) =>
  `?timestamp=${toPrettyDateTimeFormat(timestamp)?.replaceAll("-", "").replaceAll(":", "").replaceAll(" ", "")}`

export const decodeTimestampSearchParam = (search: string) =>
  search
    .replace("?timestamp=", "")
    .replace(/(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})/, "$1-$2-$3 $4:$5:$6")
