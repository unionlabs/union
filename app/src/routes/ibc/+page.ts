import type { PageLoad } from "./$types.ts"

export const load = (async context => ({
  channels: [
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0x479499E2247991318dC208EF3709C8b73b76ABe7",
        channel_id: "channel-8"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union135ht30a9had32hpl7mgsl5c85hywf0z3avr534j62gl05xses8lsjt8nv9",
      channel_id: "channel-15"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: { port_id: "ucs01-relay", channel_id: "channel-0" },
      connection_hops: ["connection-0"],
      version: "ucs01-0",
      port_id: "wasm.union14pfzjnvzacqsmgjyf0avksc8cr70hsyt5epzcp66tmjpswf8sq8sn5meuy",
      channel_id: "channel-0"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: { port_id: "413fc3995bc3a8f2e58122768406cf26", channel_id: "channel-8" },
      connection_hops: ["connection-0"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union15l8pewpq2hh5hny0mvk3rgmxajmet8843mfz53ewp2t20rgusuyqfupx39",
      channel_id: "channel-6"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0x7Afc022bbbc2Bc7Fd01F598A21Bc8d484A56139e",
        channel_id: "channel-7"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union164n0dtcwhjfg8ds2vjezh6xdmyzqk6lw8na6zdthmhtmeuhyd5qqys8m9w",
      channel_id: "channel-14"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0xE1d41460253EF763705495d9ED86155f69f858a7",
        channel_id: "channel-5"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1660l29h522aaxkz0pvejfvgwuzr0rxywpw7lalxrtea4aaeztzjs2sflm6",
      channel_id: "channel-12"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: { port_id: "94fcbcee6cf9589b109768b393410418", channel_id: "channel-5" },
      connection_hops: ["connection-0"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union17p0jtnhkh2p7gwnsrjam8qq7gc6wpuj09kxg4gjzy454wn2gq98qnpazku",
      channel_id: "channel-3"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: { port_id: "f1ba910c57bf8178613a558a8da54e11", channel_id: "channel-6" },
      connection_hops: ["connection-0"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union17p0jtnhkh2p7gwnsrjam8qq7gc6wpuj09kxg4gjzy454wn2gq98qnpazku",
      channel_id: "channel-4"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: { port_id: "03a8670eaa1faab88fb2c064b5b2d960", channel_id: "channel-7" },
      connection_hops: ["connection-0"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union17p0jtnhkh2p7gwnsrjam8qq7gc6wpuj09kxg4gjzy454wn2gq98qnpazku",
      channel_id: "channel-5"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0xCD166c5E45b7343B770ebDe91A2E403C8D0a9E7c",
        channel_id: "channel-6"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1eeuxzqhhzpdxcj3309xd553zr9u25w567es3pmajvrlmytuz3r3s7cg7jr",
      channel_id: "channel-13"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0xcE6f7170f87b10659dAB8d85e0E5cD1Fea9910eb",
        channel_id: "channel-9"
      },
      connection_hops: ["connection-2"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1fs6lt476jqsntd0aspc7keedgdyfker3gkqj6qr67yxhw73y84ps9x00m2",
      channel_id: "channel-16"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0xc65f0C6415A0c7033796aef897B6ed345a4F1a4c",
        channel_id: "channel-3"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1j5qn0kju40rdd3r3tsz3zwrfmafmnxy5gw5dy95g0ly2gjpwfwfqcdktw5",
      channel_id: "channel-10"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: { port_id: "626943814afbb34c3680c95cbd712b7f", channel_id: "channel-3" },
      connection_hops: ["connection-0"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1jgz53x9c3a33fd00t56cr69vpnrysycet4h0kcymprl7hec7q7ksa9tpph",
      channel_id: "channel-1"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0xeb2eb555451eba91e026b6a6608aa04ed262c6fd",
        channel_id: "channel-4"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1pnmrq6qc9jl26urkl7pfqmlu7y29ddxsu5yflchpyw6g7akulzls7halvk",
      channel_id: "channel-11"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0xe948d15121f77220815981a456ed4439e22de266",
        channel_id: "channel-0"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1py2d2kucttwyz444pspkhhzupyqwdywy9xtt7cp8e9zh8qgfvvls4dljmy",
      channel_id: "channel-7"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: { port_id: "de76c596f1cf9785809d17b5c2fb80b8", channel_id: "channel-4" },
      connection_hops: ["connection-0"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1uj86kw7ekaj67kqyp9lcvf98ukrmvlufg9zh0aga8l4wfajkvazqtzsgh8",
      channel_id: "channel-2"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0x8ca1E8968cf2356d1b319ffb034fFe35C15d7d24",
        channel_id: "channel-2"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1w65jz7a77zpp4u3393twt3w7vjclpqekhamflyjdtwfp7u5mrkcqul8lz7",
      channel_id: "channel-9"
    },
    {
      state: "STATE_OPEN",
      ordering: "ORDER_UNORDERED",
      counterparty: {
        port_id: "0x8adf0dcb22e2ddc3cb570548198b40682b702445",
        channel_id: "channel-1"
      },
      connection_hops: ["connection-1"],
      version: "ucs00-pingpong-1",
      port_id: "wasm.union1ydsptvkxa9azenfypzdllhtuatrwrztpw65x3ggpj5eavs5hksaqfc8823",
      channel_id: "channel-8"
    }
  ],
  connections: [
    {
      id: "connection-localhost",
      client_id: "09-localhost",
      versions: [{ identifier: "1", features: ["ORDER_ORDERED", "ORDER_UNORDERED"] }],
      state: "STATE_OPEN",
      counterparty: {
        client_id: "09-localhost",
        connection_id: "connection-localhost",
        prefix: { key_prefix: "aWJj" }
      },
      delay_period: "0"
    }
  ]
})) satisfies PageLoad
