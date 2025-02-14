import { useState, useEffect } from "react"
import { ApolloClient, InMemoryCache, gql } from "@apollo/client"

const client = new ApolloClient({
  uri: "https://hubble-green.hasura.app/v1/graphql",
  cache: new InMemoryCache(),
  defaultOptions: {
    watchQuery: {
      fetchPolicy: "no-cache",
      errorPolicy: "ignore"
    },
    query: {
      fetchPolicy: "no-cache",
      errorPolicy: "all"
    }
  }
})

function App() {
  const [clients, setClients] = useState([])
  const [chains, setChains] = useState([])
  useEffect(() => {
    const load = async () => {
      const {
        data: { v1_ibc_union_client_heights_max, v1_ibc_union_chains }
      } = await client.query({
        query: gql`
                        query MyQuery {
                            v1_ibc_union_client_heights_max {
                                client_chain {
                                    chain_id
                                }
                                counterparty_chain {
                                    chain_id
                                }
                                client_id
                                max_counterparty_height
                            }
                            v1_ibc_union_chains {
                                index_status {
                                    chain_id
                                    height
                                }
                            }
                        }
                    `
      })
      setChains(v1_ibc_union_chains)
      setClients(
        Object.groupBy(v1_ibc_union_client_heights_max, client => client.client_chain.chain_id)
      )
    }
    const id = setInterval(() => {
      load()
    }, 2000)
    return () => clearInterval(id)
  }, [])
  const length = Object.keys(clients).length
  return (
    <>
      <h1>bul merkat</h1>
      <div class="table-responsive">
        <table class="center">
          <caption>clients</caption>
          <thead>
            <tr>
              <th colspan="2" rowspan="2"></th>
              <th colspan={length}>tracker</th>
            </tr>
            <tr>
              {Object.keys(clients).map(chain => (
                <th key={chain}>{chain}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Object.entries(clients).map(([counterpartyChainId, _], counterpartyIndex) => (
              <tr key={counterpartyIndex}>
                {counterpartyIndex === 0 && (
                  <th key="first" rowspan={length}>
                    <div>tracked</div>
                  </th>
                )}
                <th key={counterpartyIndex}>{counterpartyChainId}</th>
                {Object.entries(clients).map(([chainId, values]) => {
                  if (chainId === counterpartyChainId) {
                    return <td key="{counterpartyChainId}-{chainId}" className="bg-default"></td>
                  }
                  const counterpartyChain = chains.find(chain => {
                    return chain.index_status.chain_id === counterpartyChainId
                  })
                  const client = values.find(
                    value => value.counterparty_chain.chain_id === counterpartyChainId
                  )
                  return (
                    <td key={chainId} className="bg-muted">
                      <i className="smaller success">{client.client_id}</i>
                      <br></br>
                      <span className="small">
                        {counterpartyChain.index_status.height - client.max_counterparty_height}
                      </span>
                      <br></br>
                      <span>{client.max_counterparty_height}</span>
                      <br></br>
                      <span>{counterpartyChain.index_status.height}</span>
                    </td>
                  )
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  )
}

export default App
