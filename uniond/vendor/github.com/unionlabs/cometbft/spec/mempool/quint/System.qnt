// -*- mode: Bluespec; -*-
module System{
    type Process = str
    type Tx = str
    type Block = Set[Tx]

    pure val PROCESSES=Set("alice", "bob", "charlie")
    pure val TXS=Set("T0", "T1", "T2")
}
