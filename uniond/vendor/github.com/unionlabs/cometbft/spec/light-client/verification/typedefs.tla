--------------------------- MODULE typedefs ----------------------------
(*
 // a node id, just a string
 @typeAlias: node = Str;

 // a block header
 @typeAlias: header = {
   height: Int,
   time: Int,
   lastCommit: Set($node),
   VS: Set($node),
   NextVS: Set($node)
 };

 // the blockchain is a function of heights to blocks
 @typeAlias: blockchain = Int -> $header;

 // a light-client block header
 @typeAlias: lightHeader = {
   header: $header,
   Commits: Set($node)
 };
 *)
typedefs == TRUE
========================================================================
