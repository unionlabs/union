export const ibcHandlerAbi = <const>[
  {
    type: "function",
    name: "COMMITMENT_PREFIX",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "capabilities",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "channels",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      },
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.State"
      },
      {
        name: "ordering",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Counterparty.Data",
        components: [
          {
            name: "port_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel_id",
            type: "string",
            internalType: "string"
          }
        ]
      },
      {
        name: "version",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientImpls",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientRegistry",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientTypes",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "commitments",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "connections",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "client_id",
        type: "string",
        internalType: "string"
      },
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreConnectionV1GlobalEnums.State"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreConnectionV1Counterparty.Data",
        components: [
          {
            name: "client_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "connection_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "prefix",
            type: "tuple",
            internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
            components: [
              {
                name: "key_prefix",
                type: "bytes",
                internalType: "bytes"
              }
            ]
          }
        ]
      },
      {
        name: "delay_period",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "createClient",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgCreateClient",
        components: [
          {
            name: "clientType",
            type: "string",
            internalType: "string"
          },
          {
            name: "clientStateBytes",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "consensusStateBytes",
            type: "bytes",
            internalType: "bytes"
          }
        ]
      }
    ],
    outputs: [
      {
        name: "clientId",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "getClient",
    inputs: [
      {
        name: "clientId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "contract ILightClient"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextChannelSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextClientSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextConnectionSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "registerClient",
    inputs: [
      {
        name: "clientType",
        type: "string",
        internalType: "string"
      },
      {
        name: "client",
        type: "address",
        internalType: "contract ILightClient"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "updateClient",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgUpdateClient",
        components: [
          {
            name: "clientId",
            type: "string",
            internalType: "string"
          },
          {
            name: "clientMessage",
            type: "bytes",
            internalType: "bytes"
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "event",
    name: "ClientCreated",
    inputs: [
      {
        name: "",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ClientRegistered",
    inputs: [
      {
        name: "",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "",
        type: "address",
        indexed: false,
        internalType: "address"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ClientUpdated",
    inputs: [
      {
        name: "",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "",
        type: "tuple",
        indexed: false,
        internalType: "struct IbcCoreClientV1Height.Data",
        components: [
          {
            name: "revision_number",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "revision_height",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      }
    ],
    anonymous: false
  },
  {
    type: "error",
    name: "ErrClientMustNotBeSelf",
    inputs: []
  },
  {
    type: "error",
    name: "ErrClientNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrClientTypeAlreadyExists",
    inputs: []
  },
  {
    type: "error",
    name: "ErrClientTypeNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrFailedToCreateClient",
    inputs: []
  },
  {
    type: "error",
    name: "ErrFailedToUpdateClient",
    inputs: []
  },
  {
    type: "function",
    name: "COMMITMENT_PREFIX",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "acknowledgePacket",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgPacketAcknowledgement",
        components: [
          {
            name: "packet",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Packet.Data",
            components: [
              {
                name: "sequence",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "source_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "source_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "data",
                type: "bytes",
                internalType: "bytes"
              },
              {
                name: "timeout_height",
                type: "tuple",
                internalType: "struct IbcCoreClientV1Height.Data",
                components: [
                  {
                    name: "revision_number",
                    type: "uint64",
                    internalType: "uint64"
                  },
                  {
                    name: "revision_height",
                    type: "uint64",
                    internalType: "uint64"
                  }
                ]
              },
              {
                name: "timeout_timestamp",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "acknowledgement",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proof",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "capabilities",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "channelCapabilityPath",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string"
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "pure"
  },
  {
    type: "function",
    name: "channels",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      },
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.State"
      },
      {
        name: "ordering",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Counterparty.Data",
        components: [
          {
            name: "port_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel_id",
            type: "string",
            internalType: "string"
          }
        ]
      },
      {
        name: "version",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientImpls",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientRegistry",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientTypes",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "commitments",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "connections",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "client_id",
        type: "string",
        internalType: "string"
      },
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreConnectionV1GlobalEnums.State"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreConnectionV1Counterparty.Data",
        components: [
          {
            name: "client_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "connection_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "prefix",
            type: "tuple",
            internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
            components: [
              {
                name: "key_prefix",
                type: "bytes",
                internalType: "bytes"
              }
            ]
          }
        ]
      },
      {
        name: "delay_period",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getClient",
    inputs: [
      {
        name: "clientId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "contract ILightClient"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextChannelSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextClientSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextConnectionSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "recvPacket",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgPacketRecv",
        components: [
          {
            name: "packet",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Packet.Data",
            components: [
              {
                name: "sequence",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "source_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "source_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "data",
                type: "bytes",
                internalType: "bytes"
              },
              {
                name: "timeout_height",
                type: "tuple",
                internalType: "struct IbcCoreClientV1Height.Data",
                components: [
                  {
                    name: "revision_number",
                    type: "uint64",
                    internalType: "uint64"
                  },
                  {
                    name: "revision_height",
                    type: "uint64",
                    internalType: "uint64"
                  }
                ]
              },
              {
                name: "timeout_timestamp",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "proof",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "sendPacket",
    inputs: [
      {
        name: "sourcePort",
        type: "string",
        internalType: "string"
      },
      {
        name: "sourceChannel",
        type: "string",
        internalType: "string"
      },
      {
        name: "timeoutHeight",
        type: "tuple",
        internalType: "struct IbcCoreClientV1Height.Data",
        components: [
          {
            name: "revision_number",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "revision_height",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      {
        name: "timeoutTimestamp",
        type: "uint64",
        internalType: "uint64"
      },
      {
        name: "data",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "timeoutPacket",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgPacketTimeout",
        components: [
          {
            name: "packet",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Packet.Data",
            components: [
              {
                name: "sequence",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "source_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "source_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "data",
                type: "bytes",
                internalType: "bytes"
              },
              {
                name: "timeout_height",
                type: "tuple",
                internalType: "struct IbcCoreClientV1Height.Data",
                components: [
                  {
                    name: "revision_number",
                    type: "uint64",
                    internalType: "uint64"
                  },
                  {
                    name: "revision_height",
                    type: "uint64",
                    internalType: "uint64"
                  }
                ]
              },
              {
                name: "timeout_timestamp",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "proof",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "nextSequenceRecv",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "writeAcknowledgement",
    inputs: [
      {
        name: "destinationPort",
        type: "string",
        internalType: "string"
      },
      {
        name: "destinationChannel",
        type: "string",
        internalType: "string"
      },
      {
        name: "sequence",
        type: "uint64",
        internalType: "uint64"
      },
      {
        name: "acknowledgement",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "event",
    name: "AcknowledgePacket",
    inputs: [
      {
        name: "packet",
        type: "tuple",
        indexed: false,
        internalType: "struct IbcCoreChannelV1Packet.Data",
        components: [
          {
            name: "sequence",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "source_port",
            type: "string",
            internalType: "string"
          },
          {
            name: "source_channel",
            type: "string",
            internalType: "string"
          },
          {
            name: "destination_port",
            type: "string",
            internalType: "string"
          },
          {
            name: "destination_channel",
            type: "string",
            internalType: "string"
          },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "timeout_height",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "timeout_timestamp",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      {
        name: "acknowledgement",
        type: "bytes",
        indexed: false,
        internalType: "bytes"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "RecvPacket",
    inputs: [
      {
        name: "packet",
        type: "tuple",
        indexed: false,
        internalType: "struct IbcCoreChannelV1Packet.Data",
        components: [
          {
            name: "sequence",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "source_port",
            type: "string",
            internalType: "string"
          },
          {
            name: "source_channel",
            type: "string",
            internalType: "string"
          },
          {
            name: "destination_port",
            type: "string",
            internalType: "string"
          },
          {
            name: "destination_channel",
            type: "string",
            internalType: "string"
          },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "timeout_height",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "timeout_timestamp",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "SendPacket",
    inputs: [
      {
        name: "sequence",
        type: "uint64",
        indexed: false,
        internalType: "uint64"
      },
      {
        name: "sourcePort",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "sourceChannel",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "timeoutHeight",
        type: "tuple",
        indexed: false,
        internalType: "struct IbcCoreClientV1Height.Data",
        components: [
          {
            name: "revision_number",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "revision_height",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      {
        name: "timeoutTimestamp",
        type: "uint64",
        indexed: false,
        internalType: "uint64"
      },
      {
        name: "data",
        type: "bytes",
        indexed: false,
        internalType: "bytes"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "TimeoutPacket",
    inputs: [
      {
        name: "packet",
        type: "tuple",
        indexed: false,
        internalType: "struct IbcCoreChannelV1Packet.Data",
        components: [
          {
            name: "sequence",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "source_port",
            type: "string",
            internalType: "string"
          },
          {
            name: "source_channel",
            type: "string",
            internalType: "string"
          },
          {
            name: "destination_port",
            type: "string",
            internalType: "string"
          },
          {
            name: "destination_channel",
            type: "string",
            internalType: "string"
          },
          {
            name: "data",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "timeout_height",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "timeout_timestamp",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "WriteAcknowledgement",
    inputs: [
      {
        name: "destinationPort",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "destinationChannel",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "sequence",
        type: "uint64",
        indexed: false,
        internalType: "uint64"
      },
      {
        name: "acknowledgement",
        type: "bytes",
        indexed: false,
        internalType: "bytes"
      }
    ],
    anonymous: false
  },
  {
    type: "error",
    name: "ErrAcknowledgementAlreadyExists",
    inputs: []
  },
  {
    type: "error",
    name: "ErrAcknowledgementIsEmpty",
    inputs: []
  },
  {
    type: "error",
    name: "ErrClientNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrDestinationAndCounterpartyChannelMismatch",
    inputs: []
  },
  {
    type: "error",
    name: "ErrDestinationAndCounterpartyPortMismatch",
    inputs: []
  },
  {
    type: "error",
    name: "ErrHeightTimeout",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidChannelState",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidConnectionState",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidPacketCommitment",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidProof",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidTimeoutHeight",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidTimeoutTimestamp",
    inputs: []
  },
  {
    type: "error",
    name: "ErrLatestHeightNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrLatestTimestampNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrModuleNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrNextSequenceMustBeGreaterThanTimeoutSequence",
    inputs: []
  },
  {
    type: "error",
    name: "ErrPacketAlreadyReceived",
    inputs: []
  },
  {
    type: "error",
    name: "ErrPacketCommitmentNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrPacketSequenceNextSequenceMismatch",
    inputs: []
  },
  {
    type: "error",
    name: "ErrPacketWithoutTimeout",
    inputs: []
  },
  {
    type: "error",
    name: "ErrSourceAndCounterpartyChannelMismatch",
    inputs: []
  },
  {
    type: "error",
    name: "ErrSourceAndCounterpartyPortMismatch",
    inputs: []
  },
  {
    type: "error",
    name: "ErrTimeoutHeightNotReached",
    inputs: []
  },
  {
    type: "error",
    name: "ErrTimeoutTimestampNotReached",
    inputs: []
  },
  {
    type: "error",
    name: "ErrTimestampTimeout",
    inputs: []
  },
  {
    type: "error",
    name: "ErrUnauthorized",
    inputs: []
  },
  {
    type: "error",
    name: "ErrUnknownChannelOrdering",
    inputs: []
  },
  {
    type: "function",
    name: "COMMITMENT_PREFIX",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "capabilities",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "channels",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      },
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.State"
      },
      {
        name: "ordering",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Counterparty.Data",
        components: [
          {
            name: "port_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel_id",
            type: "string",
            internalType: "string"
          }
        ]
      },
      {
        name: "version",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientImpls",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientRegistry",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientTypes",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "commitments",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "connectionOpenAck",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgConnectionOpenAck",
        components: [
          {
            name: "connectionId",
            type: "string",
            internalType: "string"
          },
          {
            name: "clientStateBytes",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "version",
            type: "tuple",
            internalType: "struct IbcCoreConnectionV1Version.Data",
            components: [
              {
                name: "identifier",
                type: "string",
                internalType: "string"
              },
              {
                name: "features",
                type: "string[]",
                internalType: "string[]"
              }
            ]
          },
          {
            name: "counterpartyConnectionID",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofTry",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofClient",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofConsensus",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "consensusHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "connectionOpenConfirm",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgConnectionOpenConfirm",
        components: [
          {
            name: "connectionId",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofAck",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "connectionOpenInit",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgConnectionOpenInit",
        components: [
          {
            name: "clientId",
            type: "string",
            internalType: "string"
          },
          {
            name: "counterparty",
            type: "tuple",
            internalType: "struct IbcCoreConnectionV1Counterparty.Data",
            components: [
              {
                name: "client_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "connection_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "prefix",
                type: "tuple",
                internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
                components: [
                  {
                    name: "key_prefix",
                    type: "bytes",
                    internalType: "bytes"
                  }
                ]
              }
            ]
          },
          {
            name: "delayPeriod",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "connectionOpenTry",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgConnectionOpenTry",
        components: [
          {
            name: "counterparty",
            type: "tuple",
            internalType: "struct IbcCoreConnectionV1Counterparty.Data",
            components: [
              {
                name: "client_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "connection_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "prefix",
                type: "tuple",
                internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
                components: [
                  {
                    name: "key_prefix",
                    type: "bytes",
                    internalType: "bytes"
                  }
                ]
              }
            ]
          },
          {
            name: "delayPeriod",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "clientId",
            type: "string",
            internalType: "string"
          },
          {
            name: "clientStateBytes",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "counterpartyVersions",
            type: "tuple[]",
            internalType: "struct IbcCoreConnectionV1Version.Data[]",
            components: [
              {
                name: "identifier",
                type: "string",
                internalType: "string"
              },
              {
                name: "features",
                type: "string[]",
                internalType: "string[]"
              }
            ]
          },
          {
            name: "proofInit",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofClient",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofConsensus",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "consensusHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "connections",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "client_id",
        type: "string",
        internalType: "string"
      },
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreConnectionV1GlobalEnums.State"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreConnectionV1Counterparty.Data",
        components: [
          {
            name: "client_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "connection_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "prefix",
            type: "tuple",
            internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
            components: [
              {
                name: "key_prefix",
                type: "bytes",
                internalType: "bytes"
              }
            ]
          }
        ]
      },
      {
        name: "delay_period",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getClient",
    inputs: [
      {
        name: "clientId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "contract ILightClient"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextChannelSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextClientSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextConnectionSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "event",
    name: "ConnectionOpenAck",
    inputs: [
      {
        name: "connectionId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ConnectionOpenConfirm",
    inputs: [
      {
        name: "connectionId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ConnectionOpenInit",
    inputs: [
      {
        name: "connectionId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ConnectionOpenTry",
    inputs: [
      {
        name: "connectionId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "error",
    name: "ErrClientNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrConnectionAlreadyExists",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidConnectionState",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidProof",
    inputs: []
  },
  {
    type: "error",
    name: "ErrNoCounterpartyVersion",
    inputs: []
  },
  {
    type: "error",
    name: "ErrUnsupportedVersion",
    inputs: []
  },
  {
    type: "error",
    name: "ErrValidateSelfClient",
    inputs: []
  },
  {
    type: "constructor",
    inputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "COMMITMENT_PREFIX",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "UPGRADE_INTERFACE_VERSION",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "acknowledgePacket",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgPacketAcknowledgement",
        components: [
          {
            name: "packet",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Packet.Data",
            components: [
              {
                name: "sequence",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "source_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "source_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "data",
                type: "bytes",
                internalType: "bytes"
              },
              {
                name: "timeout_height",
                type: "tuple",
                internalType: "struct IbcCoreClientV1Height.Data",
                components: [
                  {
                    name: "revision_number",
                    type: "uint64",
                    internalType: "uint64"
                  },
                  {
                    name: "revision_height",
                    type: "uint64",
                    internalType: "uint64"
                  }
                ]
              },
              {
                name: "timeout_timestamp",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "acknowledgement",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proof",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "capabilities",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "channelCapabilityPath",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string"
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "pure"
  },
  {
    type: "function",
    name: "channelCloseConfirm",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelCloseConfirm",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channelId",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofInit",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelCloseInit",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelCloseInit",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channelId",
            type: "string",
            internalType: "string"
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelOpenAck",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelOpenAck",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channelId",
            type: "string",
            internalType: "string"
          },
          {
            name: "counterpartyVersion",
            type: "string",
            internalType: "string"
          },
          {
            name: "counterpartyChannelId",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofTry",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelOpenConfirm",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelOpenConfirm",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channelId",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofAck",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelOpenInit",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelOpenInit",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Channel.Data",
            components: [
              {
                name: "state",
                type: "uint8",
                internalType: "enum IbcCoreChannelV1GlobalEnums.State"
              },
              {
                name: "ordering",
                type: "uint8",
                internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
              },
              {
                name: "counterparty",
                type: "tuple",
                internalType: "struct IbcCoreChannelV1Counterparty.Data",
                components: [
                  {
                    name: "port_id",
                    type: "string",
                    internalType: "string"
                  },
                  {
                    name: "channel_id",
                    type: "string",
                    internalType: "string"
                  }
                ]
              },
              {
                name: "connection_hops",
                type: "string[]",
                internalType: "string[]"
              },
              {
                name: "version",
                type: "string",
                internalType: "string"
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelOpenTry",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelOpenTry",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Channel.Data",
            components: [
              {
                name: "state",
                type: "uint8",
                internalType: "enum IbcCoreChannelV1GlobalEnums.State"
              },
              {
                name: "ordering",
                type: "uint8",
                internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
              },
              {
                name: "counterparty",
                type: "tuple",
                internalType: "struct IbcCoreChannelV1Counterparty.Data",
                components: [
                  {
                    name: "port_id",
                    type: "string",
                    internalType: "string"
                  },
                  {
                    name: "channel_id",
                    type: "string",
                    internalType: "string"
                  }
                ]
              },
              {
                name: "connection_hops",
                type: "string[]",
                internalType: "string[]"
              },
              {
                name: "version",
                type: "string",
                internalType: "string"
              }
            ]
          },
          {
            name: "counterpartyVersion",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofInit",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channels",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      },
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.State"
      },
      {
        name: "ordering",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Counterparty.Data",
        components: [
          {
            name: "port_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel_id",
            type: "string",
            internalType: "string"
          }
        ]
      },
      {
        name: "version",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientImpls",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientRegistry",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientTypes",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "commitments",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "connectionOpenAck",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgConnectionOpenAck",
        components: [
          {
            name: "connectionId",
            type: "string",
            internalType: "string"
          },
          {
            name: "clientStateBytes",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "version",
            type: "tuple",
            internalType: "struct IbcCoreConnectionV1Version.Data",
            components: [
              {
                name: "identifier",
                type: "string",
                internalType: "string"
              },
              {
                name: "features",
                type: "string[]",
                internalType: "string[]"
              }
            ]
          },
          {
            name: "counterpartyConnectionID",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofTry",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofClient",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofConsensus",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "consensusHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "connectionOpenConfirm",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgConnectionOpenConfirm",
        components: [
          {
            name: "connectionId",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofAck",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "connectionOpenInit",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgConnectionOpenInit",
        components: [
          {
            name: "clientId",
            type: "string",
            internalType: "string"
          },
          {
            name: "counterparty",
            type: "tuple",
            internalType: "struct IbcCoreConnectionV1Counterparty.Data",
            components: [
              {
                name: "client_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "connection_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "prefix",
                type: "tuple",
                internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
                components: [
                  {
                    name: "key_prefix",
                    type: "bytes",
                    internalType: "bytes"
                  }
                ]
              }
            ]
          },
          {
            name: "delayPeriod",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "connectionOpenTry",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgConnectionOpenTry",
        components: [
          {
            name: "counterparty",
            type: "tuple",
            internalType: "struct IbcCoreConnectionV1Counterparty.Data",
            components: [
              {
                name: "client_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "connection_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "prefix",
                type: "tuple",
                internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
                components: [
                  {
                    name: "key_prefix",
                    type: "bytes",
                    internalType: "bytes"
                  }
                ]
              }
            ]
          },
          {
            name: "delayPeriod",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "clientId",
            type: "string",
            internalType: "string"
          },
          {
            name: "clientStateBytes",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "counterpartyVersions",
            type: "tuple[]",
            internalType: "struct IbcCoreConnectionV1Version.Data[]",
            components: [
              {
                name: "identifier",
                type: "string",
                internalType: "string"
              },
              {
                name: "features",
                type: "string[]",
                internalType: "string[]"
              }
            ]
          },
          {
            name: "proofInit",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofClient",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofConsensus",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "consensusHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "connections",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "client_id",
        type: "string",
        internalType: "string"
      },
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreConnectionV1GlobalEnums.State"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreConnectionV1Counterparty.Data",
        components: [
          {
            name: "client_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "connection_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "prefix",
            type: "tuple",
            internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
            components: [
              {
                name: "key_prefix",
                type: "bytes",
                internalType: "bytes"
              }
            ]
          }
        ]
      },
      {
        name: "delay_period",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "createClient",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgCreateClient",
        components: [
          {
            name: "clientType",
            type: "string",
            internalType: "string"
          },
          {
            name: "clientStateBytes",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "consensusStateBytes",
            type: "bytes",
            internalType: "bytes"
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "getChannel",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string"
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Channel.Data",
        components: [
          {
            name: "state",
            type: "uint8",
            internalType: "enum IbcCoreChannelV1GlobalEnums.State"
          },
          {
            name: "ordering",
            type: "uint8",
            internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
          },
          {
            name: "counterparty",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Counterparty.Data",
            components: [
              {
                name: "port_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "channel_id",
                type: "string",
                internalType: "string"
              }
            ]
          },
          {
            name: "connection_hops",
            type: "string[]",
            internalType: "string[]"
          },
          {
            name: "version",
            type: "string",
            internalType: "string"
          }
        ]
      },
      {
        name: "",
        type: "bool",
        internalType: "bool"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getClient",
    inputs: [
      {
        name: "clientId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "contract ILightClient"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getClientState",
    inputs: [
      {
        name: "clientId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bytes",
        internalType: "bytes"
      },
      {
        name: "",
        type: "bool",
        internalType: "bool"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getConnection",
    inputs: [
      {
        name: "connectionId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "tuple",
        internalType: "struct IbcCoreConnectionV1ConnectionEnd.Data",
        components: [
          {
            name: "client_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "versions",
            type: "tuple[]",
            internalType: "struct IbcCoreConnectionV1Version.Data[]",
            components: [
              {
                name: "identifier",
                type: "string",
                internalType: "string"
              },
              {
                name: "features",
                type: "string[]",
                internalType: "string[]"
              }
            ]
          },
          {
            name: "state",
            type: "uint8",
            internalType: "enum IbcCoreConnectionV1GlobalEnums.State"
          },
          {
            name: "counterparty",
            type: "tuple",
            internalType: "struct IbcCoreConnectionV1Counterparty.Data",
            components: [
              {
                name: "client_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "connection_id",
                type: "string",
                internalType: "string"
              },
              {
                name: "prefix",
                type: "tuple",
                internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
                components: [
                  {
                    name: "key_prefix",
                    type: "bytes",
                    internalType: "bytes"
                  }
                ]
              }
            ]
          },
          {
            name: "delay_period",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      {
        name: "",
        type: "bool",
        internalType: "bool"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getConsensusState",
    inputs: [
      {
        name: "clientId",
        type: "string",
        internalType: "string"
      },
      {
        name: "height",
        type: "tuple",
        internalType: "struct IbcCoreClientV1Height.Data",
        components: [
          {
            name: "revision_number",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "revision_height",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      }
    ],
    outputs: [
      {
        name: "consensusStateBytes",
        type: "bytes",
        internalType: "bytes"
      },
      {
        name: "",
        type: "bool",
        internalType: "bool"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getHashedPacketAcknowledgementCommitment",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string"
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string"
      },
      {
        name: "sequence",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      },
      {
        name: "",
        type: "bool",
        internalType: "bool"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getHashedPacketCommitment",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string"
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string"
      },
      {
        name: "sequence",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      },
      {
        name: "",
        type: "bool",
        internalType: "bool"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getNextSequenceSend",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string"
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "hasPacketReceipt",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string"
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string"
      },
      {
        name: "sequence",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bool",
        internalType: "bool"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "initialize",
    inputs: [
      {
        name: "ibcClient",
        type: "address",
        internalType: "address"
      },
      {
        name: "ibcConnection",
        type: "address",
        internalType: "address"
      },
      {
        name: "ibcChannel",
        type: "address",
        internalType: "address"
      },
      {
        name: "ibcPacket",
        type: "address",
        internalType: "address"
      },
      {
        name: "admin",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "nextChannelSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextClientSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextConnectionSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "owner",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "paused",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "bool",
        internalType: "bool"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "proxiableUUID",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "recvPacket",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgPacketRecv",
        components: [
          {
            name: "packet",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Packet.Data",
            components: [
              {
                name: "sequence",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "source_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "source_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "data",
                type: "bytes",
                internalType: "bytes"
              },
              {
                name: "timeout_height",
                type: "tuple",
                internalType: "struct IbcCoreClientV1Height.Data",
                components: [
                  {
                    name: "revision_number",
                    type: "uint64",
                    internalType: "uint64"
                  },
                  {
                    name: "revision_height",
                    type: "uint64",
                    internalType: "uint64"
                  }
                ]
              },
              {
                name: "timeout_timestamp",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "proof",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "registerClient",
    inputs: [
      {
        name: "clientType",
        type: "string",
        internalType: "string"
      },
      {
        name: "client",
        type: "address",
        internalType: "contract ILightClient"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "renounceOwnership",
    inputs: [],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "sendPacket",
    inputs: [
      {
        name: "sourcePort",
        type: "string",
        internalType: "string"
      },
      {
        name: "sourceChannel",
        type: "string",
        internalType: "string"
      },
      {
        name: "timeoutHeight",
        type: "tuple",
        internalType: "struct IbcCoreClientV1Height.Data",
        components: [
          {
            name: "revision_number",
            type: "uint64",
            internalType: "uint64"
          },
          {
            name: "revision_height",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      },
      {
        name: "timeoutTimestamp",
        type: "uint64",
        internalType: "uint64"
      },
      {
        name: "data",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "timeoutPacket",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgPacketTimeout",
        components: [
          {
            name: "packet",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Packet.Data",
            components: [
              {
                name: "sequence",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "source_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "source_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_port",
                type: "string",
                internalType: "string"
              },
              {
                name: "destination_channel",
                type: "string",
                internalType: "string"
              },
              {
                name: "data",
                type: "bytes",
                internalType: "bytes"
              },
              {
                name: "timeout_height",
                type: "tuple",
                internalType: "struct IbcCoreClientV1Height.Data",
                components: [
                  {
                    name: "revision_number",
                    type: "uint64",
                    internalType: "uint64"
                  },
                  {
                    name: "revision_height",
                    type: "uint64",
                    internalType: "uint64"
                  }
                ]
              },
              {
                name: "timeout_timestamp",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "proof",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          },
          {
            name: "nextSequenceRecv",
            type: "uint64",
            internalType: "uint64"
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "transferOwnership",
    inputs: [
      {
        name: "newOwner",
        type: "address",
        internalType: "address"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "updateClient",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgUpdateClient",
        components: [
          {
            name: "clientId",
            type: "string",
            internalType: "string"
          },
          {
            name: "clientMessage",
            type: "bytes",
            internalType: "bytes"
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "upgradeToAndCall",
    inputs: [
      {
        name: "newImplementation",
        type: "address",
        internalType: "address"
      },
      {
        name: "data",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [],
    stateMutability: "payable"
  },
  {
    type: "function",
    name: "writeAcknowledgement",
    inputs: [
      {
        name: "destinationPortId",
        type: "string",
        internalType: "string"
      },
      {
        name: "destinationChannel",
        type: "string",
        internalType: "string"
      },
      {
        name: "sequence",
        type: "uint64",
        internalType: "uint64"
      },
      {
        name: "acknowledgement",
        type: "bytes",
        internalType: "bytes"
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "event",
    name: "Initialized",
    inputs: [
      {
        name: "version",
        type: "uint64",
        indexed: false,
        internalType: "uint64"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "OwnershipTransferred",
    inputs: [
      {
        name: "previousOwner",
        type: "address",
        indexed: true,
        internalType: "address"
      },
      {
        name: "newOwner",
        type: "address",
        indexed: true,
        internalType: "address"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "Paused",
    inputs: [
      {
        name: "account",
        type: "address",
        indexed: false,
        internalType: "address"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "Unpaused",
    inputs: [
      {
        name: "account",
        type: "address",
        indexed: false,
        internalType: "address"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "Upgraded",
    inputs: [
      {
        name: "implementation",
        type: "address",
        indexed: true,
        internalType: "address"
      }
    ],
    anonymous: false
  },
  {
    type: "error",
    name: "AddressEmptyCode",
    inputs: [
      {
        name: "target",
        type: "address",
        internalType: "address"
      }
    ]
  },
  {
    type: "error",
    name: "ERC1967InvalidImplementation",
    inputs: [
      {
        name: "implementation",
        type: "address",
        internalType: "address"
      }
    ]
  },
  {
    type: "error",
    name: "ERC1967NonPayable",
    inputs: []
  },
  {
    type: "error",
    name: "EnforcedPause",
    inputs: []
  },
  {
    type: "error",
    name: "ErrClientNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ExpectedPause",
    inputs: []
  },
  {
    type: "error",
    name: "FailedInnerCall",
    inputs: []
  },
  {
    type: "error",
    name: "InvalidInitialization",
    inputs: []
  },
  {
    type: "error",
    name: "NotInitializing",
    inputs: []
  },
  {
    type: "error",
    name: "OwnableInvalidOwner",
    inputs: [
      {
        name: "owner",
        type: "address",
        internalType: "address"
      }
    ]
  },
  {
    type: "error",
    name: "OwnableUnauthorizedAccount",
    inputs: [
      {
        name: "account",
        type: "address",
        internalType: "address"
      }
    ]
  },
  {
    type: "error",
    name: "UUPSUnauthorizedCallContext",
    inputs: []
  },
  {
    type: "error",
    name: "UUPSUnsupportedProxiableUUID",
    inputs: [
      {
        name: "slot",
        type: "bytes32",
        internalType: "bytes32"
      }
    ]
  },
  {
    type: "function",
    name: "COMMITMENT_PREFIX",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "capabilities",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "channelCapabilityPath",
    inputs: [
      {
        name: "portId",
        type: "string",
        internalType: "string"
      },
      {
        name: "channelId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "pure"
  },
  {
    type: "function",
    name: "channelCloseConfirm",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelCloseConfirm",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channelId",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofInit",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelCloseInit",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelCloseInit",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channelId",
            type: "string",
            internalType: "string"
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelOpenAck",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelOpenAck",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channelId",
            type: "string",
            internalType: "string"
          },
          {
            name: "counterpartyVersion",
            type: "string",
            internalType: "string"
          },
          {
            name: "counterpartyChannelId",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofTry",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelOpenConfirm",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelOpenConfirm",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channelId",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofAck",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelOpenInit",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelOpenInit",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Channel.Data",
            components: [
              {
                name: "state",
                type: "uint8",
                internalType: "enum IbcCoreChannelV1GlobalEnums.State"
              },
              {
                name: "ordering",
                type: "uint8",
                internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
              },
              {
                name: "counterparty",
                type: "tuple",
                internalType: "struct IbcCoreChannelV1Counterparty.Data",
                components: [
                  {
                    name: "port_id",
                    type: "string",
                    internalType: "string"
                  },
                  {
                    name: "channel_id",
                    type: "string",
                    internalType: "string"
                  }
                ]
              },
              {
                name: "connection_hops",
                type: "string[]",
                internalType: "string[]"
              },
              {
                name: "version",
                type: "string",
                internalType: "string"
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channelOpenTry",
    inputs: [
      {
        name: "msg_",
        type: "tuple",
        internalType: "struct IBCMsgs.MsgChannelOpenTry",
        components: [
          {
            name: "portId",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel",
            type: "tuple",
            internalType: "struct IbcCoreChannelV1Channel.Data",
            components: [
              {
                name: "state",
                type: "uint8",
                internalType: "enum IbcCoreChannelV1GlobalEnums.State"
              },
              {
                name: "ordering",
                type: "uint8",
                internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
              },
              {
                name: "counterparty",
                type: "tuple",
                internalType: "struct IbcCoreChannelV1Counterparty.Data",
                components: [
                  {
                    name: "port_id",
                    type: "string",
                    internalType: "string"
                  },
                  {
                    name: "channel_id",
                    type: "string",
                    internalType: "string"
                  }
                ]
              },
              {
                name: "connection_hops",
                type: "string[]",
                internalType: "string[]"
              },
              {
                name: "version",
                type: "string",
                internalType: "string"
              }
            ]
          },
          {
            name: "counterpartyVersion",
            type: "string",
            internalType: "string"
          },
          {
            name: "proofInit",
            type: "bytes",
            internalType: "bytes"
          },
          {
            name: "proofHeight",
            type: "tuple",
            internalType: "struct IbcCoreClientV1Height.Data",
            components: [
              {
                name: "revision_number",
                type: "uint64",
                internalType: "uint64"
              },
              {
                name: "revision_height",
                type: "uint64",
                internalType: "uint64"
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "nonpayable"
  },
  {
    type: "function",
    name: "channels",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      },
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.State"
      },
      {
        name: "ordering",
        type: "uint8",
        internalType: "enum IbcCoreChannelV1GlobalEnums.Order"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreChannelV1Counterparty.Data",
        components: [
          {
            name: "port_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "channel_id",
            type: "string",
            internalType: "string"
          }
        ]
      },
      {
        name: "version",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientImpls",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientRegistry",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "clientTypes",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "commitments",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    outputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "bytes32"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "connections",
    inputs: [
      {
        name: "",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "client_id",
        type: "string",
        internalType: "string"
      },
      {
        name: "state",
        type: "uint8",
        internalType: "enum IbcCoreConnectionV1GlobalEnums.State"
      },
      {
        name: "counterparty",
        type: "tuple",
        internalType: "struct IbcCoreConnectionV1Counterparty.Data",
        components: [
          {
            name: "client_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "connection_id",
            type: "string",
            internalType: "string"
          },
          {
            name: "prefix",
            type: "tuple",
            internalType: "struct IbcCoreCommitmentV1MerklePrefix.Data",
            components: [
              {
                name: "key_prefix",
                type: "bytes",
                internalType: "bytes"
              }
            ]
          }
        ]
      },
      {
        name: "delay_period",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "getClient",
    inputs: [
      {
        name: "clientId",
        type: "string",
        internalType: "string"
      }
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "contract ILightClient"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextChannelSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextClientSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "function",
    name: "nextConnectionSequence",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint64",
        internalType: "uint64"
      }
    ],
    stateMutability: "view"
  },
  {
    type: "event",
    name: "ChannelCloseConfirm",
    inputs: [
      {
        name: "channelId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "portId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ChannelCloseInit",
    inputs: [
      {
        name: "channelId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "portId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ChannelOpenAck",
    inputs: [
      {
        name: "channelId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "portId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ChannelOpenConfirm",
    inputs: [
      {
        name: "channelId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "portId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ChannelOpenInit",
    inputs: [
      {
        name: "channelId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "connectionId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "portId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "counterpartyPortId",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "event",
    name: "ChannelOpenTry",
    inputs: [
      {
        name: "channelId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "connectionId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "portId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "counterpartyPortId",
        type: "string",
        indexed: false,
        internalType: "string"
      },
      {
        name: "version",
        type: "string",
        indexed: false,
        internalType: "string"
      }
    ],
    anonymous: false
  },
  {
    type: "error",
    name: "ErrCapabilityAlreadyClaimed",
    inputs: []
  },
  {
    type: "error",
    name: "ErrClientNotFound",
    inputs: []
  },
  {
    type: "error",
    name: "ErrConnNotSingleHop",
    inputs: []
  },
  {
    type: "error",
    name: "ErrConnNotSingleVersion",
    inputs: []
  },
  {
    type: "error",
    name: "ErrCounterpartyChannelNotEmpty",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidChannelState",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidConnectionState",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidHexAddress",
    inputs: []
  },
  {
    type: "error",
    name: "ErrInvalidProof",
    inputs: []
  },
  {
    type: "error",
    name: "ErrUnsupportedFeature",
    inputs: []
  }
]
