const HASURA_ENDPOINT = "https://graphql.union.build"
const HASURA_ADMIN_SECRET = process.env.HASURA_ADMIN_SECRET || import.meta.env.HASURA_ADMIN_SECRET

const SOURCE = "hubble"
const tableNamesCSV = process.argv[2].split(",")

main().catch(error => {
  console.error(error)
  process.exit(1)
})

const getAllTablesAndViews = /* sql */ `
  SELECT table_name
  FROM "information_schema".tables
  WHERE table_schema IN ('v0_cosmos')
    AND (table_type = 'BASE TABLE' OR table_type = 'VIEW');
`

async function main() {
  const getAllTablesAndViews = await sqlQueries([])
  console.info(JSON.stringify(getAllTablesAndViews, undefined, 2))

  const createPermissions = await Promise.all(
    tableNamesCSV.map(table => createSelectPermissions({ table, role: "app" }))
  )

  console.info(JSON.stringify(createPermissions, undefined, 2))
}

async function createSelectPermissions({ table, role }: { table: string; role: string }) {
  const response = await fetch(`${HASURA_ENDPOINT}/v1/metadata`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Hasura-Role": "admin", // 'app
      "X-Hasura-Admin-Secret": HASURA_ADMIN_SECRET
    },
    body: JSON.stringify({
      type: "pg_create_select_permission",
      args: {
        table,
        role,
        source: "v0_cosmos",
        permission: {
          filter: {},
          columns: "*",
          allow_aggregations: true
        }
      }
    })
  })
  const data = await response.json()
  if (!response.ok) return console.error(`Request failed: ${JSON.stringify(data, undefined, 2)}`)
  return data
}

async function sqlQueries(queries: Array<string>) {
  const response = await fetch(`${HASURA_ENDPOINT}/v2/query`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Hasura-Admin-Secret": HASURA_ADMIN_SECRET
    },
    body: JSON.stringify({
      type: "bulk",
      source: SOURCE,
      args: queries.map(sql => ({
        type: "run_sql",
        args: { sql, cascade: false, read_only: true, source: "hubble" }
      }))
    })
  })

  const data = await response.json()
  if (!response.ok) return console.error(`Request failed: ${JSON.stringify(data, undefined, 2)}`)
  if (!Array.isArray(data))
    return console.error(`Expected array, got: ${JSON.stringify(data, undefined, 2)}`)
  const [result] = data as Array<{ result: Array<Array<string>> }>
  return result.result.map(([table]) => table)
}
