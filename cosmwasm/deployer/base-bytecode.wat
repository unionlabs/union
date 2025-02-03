;; bare minimum cosmwasm contract to be used for deterministic addresses
;; this will be deployed on-chain and instantiated with a salt, and then migrated to the desired contract code
;; compiled bytecode (207 bytes):
;; 0061736d0100000001110360037f7f7f017f60017f017f60017f000304030001020503010001074605066d656d6f7279020013696e746572666163655f76657273696f6e5f3800000b696e7374616e7469617465000008616c6c6f6361746500010a6465616c6c6f6361746500020a0f03040041330b0400413f0b0300010b0b4e010041010b487b226f6b223a7b226d65737361676573223a5b5d2c2261747472696275746573223a5b5d2c226576656e7473223a5b5d7d7d0100000032000000320000004b000000000200000002
(module
 ;; note: data must start at 1 otherwise cosmwasm complains about getting a 0 pointer
 (data (i32.const 1)
  ;; json(ContractResult::Ok(Response::default()))
  ;; note: the data key is omitted instead of being set to null since it deserializes the same
  "{\"ok\":{\"messages\":[],\"attributes\":[],\"events\":[]}}"

  ;; regions required for instantiate

  ;; response region
  "\01\00\00\00" ;; 4 bytes pointing to json data at address 1
  "\32\00\00\00" ;; 4 bytes for capacity (same as length)
  "\32\00\00\00" ;; 4 bytes for length (50)

  ;; alloc region
  "\4B\00\00\00" ;; 4 bytes pointing to empty data past this segment
  "\00\02\00\00" ;; 4 bytes for capacity (same as length)
  "\00\02"       ;; 4 bytes for length (512) (trailing zero bytes are omitted to reduce the blob size)

  ;; note: the max size required for an allocation on instantiate is 451 (for Env), but it's rounded up to 512 for safety
 )

 ;; required by cosmwasm
 (memory (export "memory") 1)

 ;; only checked for existence, never called
 (export "interface_version_8" (func 0))

 (func (export "instantiate") (param i32) (param i32) (param i32) (result i32)
  ;; return the pointer to the response region
  (i32.const 51)
 )

 (func (export "allocate") (param i32) (result i32)
  ;; return the pointer to the alloc region
  ;; this will be written to by the vm multiple times (overwriting previously written data), but the value is never read so it's fine
  (i32.const 63)
 )

 ;; noop, why bother deallocating
 (func (export "deallocate") (param i32))
)
