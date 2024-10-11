pragma solidity >=0.4.21 <0.6.0;

contract Test1 {
  function isSameAddress(address a, address b) public returns(bool){  //Simply add the two arguments and return
      if (a == b) return true;
      return false;
  }
}

contract OpCodes {

    Test1 test1;

    constructor() public {  //Constructor function
      test1 = new Test1();  //Create new "Test1" function
    }

   modifier onlyOwner(address _owner) {
      require(msg.sender == _owner);
      _;
   }
   // Add a todo to the list
   function test() public {

     //simple_instructions
     /*assembly { pop(sub(dup1, mul(dup1, dup1))) }*/

     //keywords
     assembly { pop(address) return(2, byte(2,1)) }

     //label_complex
     /*assembly { 7 abc: 8 eq jump(abc) jumpi(eq(7, 8), abc) pop }
     assembly { pop(jumpi(eq(7, 8), abc)) jump(abc) }*/

     //functional
     /*assembly { let x := 2 add(7, mul(6, x)) mul(7, 8) add =: x }*/

     //for_statement
     assembly { for { let i := 1 } lt(i, 5) { i := add(i, 1) } {} }
     assembly { for { let i := 6 } gt(i, 5) { i := add(i, 1) } {} }
     assembly { for { let i := 1 } slt(i, 5) { i := add(i, 1) } {} }
     assembly { for { let i := 6 } sgt(i, 5) { i := add(i, 1) } {} }

     //no_opcodes_in_strict
     assembly { pop(callvalue()) }

     //no_dup_swap_in_strict
     /*assembly { swap1() }*/

     //print_functional
     assembly { let x := mul(sload(0x12), 7) }

     //print_if
     assembly { if 2 { pop(mload(0)) }}

     //function_definitions_multiple_args
     assembly { function f(a, d){ mstore(a, d) } function g(a, d) -> x, y {}}

     //sstore
     assembly { function f(a, d){ sstore(a, d) } function g(a, d) -> x, y {}}

     //mstore8
     assembly { function f(a, d){ mstore8(a, d) } function g(a, d) -> x, y {}}

     //calldatacopy
     assembly {
       let a := mload(0x40)
       let b := add(a, 32)
       calldatacopy(a, 4, 32)
       /*calldatacopy(b, add(4, 32), 32)*/
       /*result := add(mload(a), mload(b))*/
     }

     //codecopy
     assembly {
       let a := mload(0x40)
       let b := add(a, 32)
       codecopy(a, 4, 32)
     }

     //codecopy
     assembly {
       let a := mload(0x40)
       let b := add(a, 32)
       extcodecopy(0, a, 4, 32)
     }

     //for_statement
     assembly { let x := calldatasize() for { let i := 0} lt(i, x) { i := add(i, 1) } { mstore(i, 2) } }

     //keccak256
     assembly { pop(keccak256(0,0)) }

     //returndatasize
     assembly { let r := returndatasize }

     //returndatacopy
     assembly { returndatacopy(64, 32, 0) }
     //byzantium vs const Constantinople
     //staticcall
     assembly { pop(staticcall(10000, 0x123, 64, 0x10, 128, 0x10)) }

     /*//create2 Constantinople
     assembly { pop(create2(10, 0x123, 32, 64)) }*/

     //create Constantinople
     assembly { pop(create(10, 0x123, 32)) }

     //shift Constantinople
     /*assembly { pop(shl(10, 32)) }
     assembly { pop(shr(10, 32)) }
     assembly { pop(sar(10, 32)) }*/


     //not
     assembly { pop( not(0x1f)) }

     //exp
     assembly { pop( exp(2, 226)) }

     //mod
     assembly { pop( mod(3, 9)) }

     //smod
     assembly { pop( smod(3, 9)) }

     //div
     assembly { pop( div(4, 2)) }

     //sdiv
     assembly { pop( sdiv(4, 2)) }

     //iszero
     assembly { pop(iszero(1)) }

     //and
     assembly { pop(and(2,3)) }

     //or
     assembly { pop(or(3,3)) }

     //xor
     assembly { pop(xor(3,3)) }

     //addmod
     assembly { pop(addmod(3,3,6)) }

     //mulmod
     assembly { pop(mulmod(3,3,3)) }

     //signextend
     assembly { pop(signextend(1, 10)) }

     //sha3
     assembly { pop(calldataload(0)) }

     //blockhash
     assembly { pop(blockhash(sub(number(), 1))) }

     //balance
     assembly { pop(balance(0x0)) }

     //caller
     assembly { pop(caller()) }

     //codesize
     assembly { pop(codesize()) }

     //extcodesize
     assembly { pop(extcodesize(0x1)) }

     //origin
     assembly { pop(origin()) }

     //gas
     assembly {  pop(gas())}

     //msize
     assembly {  pop(msize())}

     //pc
     assembly {  pop(pc())}

     //gasprice
     assembly {  pop(gasprice())}

     //coinbase
     assembly {  pop(coinbase())}

     //timestamp
     assembly {  pop(timestamp())}

     //number
     assembly {  pop(number())}

     //difficulty
     assembly {  pop(difficulty())}

     //gaslimit
     assembly {  pop(gaslimit())}

     //call
     address contractAddr = address(test1);
     bytes4 sig = bytes4(keccak256("isSameAddress(address,address)")); //Function signature
     address a = msg.sender;

     assembly {
         let x := mload(0x40)   //Find empty storage location using "free memory pointer"
         mstore(x,sig) //Place signature at beginning of empty storage
         mstore(add(x,0x04),a) // first address parameter. just after signature
         mstore(add(x,0x24),a) // 2nd address parameter - first padded. add 32 bytes (not 20 bytes)
         mstore(0x40,add(x,0x64)) // this is missing in other examples. Set free pointer before function call. so it is used by called function.
          // new free pointer position after the output values of the called function.

         let success := call(
                         5000, //5k gas
                         contractAddr, //To addr
                         0,    //No wei passed
                         x,    // Inputs are at location x
                         0x44, //Inputs size two padded, so 68 bytes
                         x,    //Store output over input
                         0x20) //Output is 32 bytes long
     }

     //callcode
     assembly {
         let x := mload(0x40)   //Find empty storage location using "free memory pointer"
         mstore(x,sig) //Place signature at beginning of empty storage
         mstore(add(x,0x04),a) // first address parameter. just after signature
         mstore(add(x,0x24),a) // 2nd address parameter - first padded. add 32 bytes (not 20 bytes)
         mstore(0x40,add(x,0x64)) // this is missing in other examples. Set free pointer before function call. so it is used by called function.
          // new free pointer position after the output values of the called function.

         let success := callcode(
                         5000, //5k gas
                         contractAddr, //To addr
                         0,    //No wei passed
                         x,    // Inputs are at location x
                         0x44, //Inputs size two padded, so 68 bytes
                         x,    //Store output over input
                         0x20) //Output is 32 bytes long
     }

     //delegatecall
     assembly {
         let x := mload(0x40)   //Find empty storage location using "free memory pointer"
         mstore(x,sig) //Place signature at beginning of empty storage
         mstore(add(x,0x04),a) // first address parameter. just after signature
         mstore(add(x,0x24),a) // 2nd address parameter - first padded. add 32 bytes (not 20 bytes)
         mstore(0x40,add(x,0x64)) // this is missing in other examples. Set free pointer before function call. so it is used by called function.
          // new free pointer position after the output values of the called function.

         let success := delegatecall(
                         5000, //5k gas
                         contractAddr, //To addr
                         x,    // Inputs are at location x
                         0x44, //Inputs size two padded, so 68 bytes
                         x,    //Store output over input
                         0x20) //Output is 32 bytes long
     }

     uint256 _id = 0x420042;

     //log0
     log0(
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20)
     );

     //log1
     log1(
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20),
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20)
     );

     //log2
     log2(
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20),
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20),
         bytes32(uint256(msg.sender))
     );

     //log3
     log3(
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20),
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20),
         bytes32(uint256(msg.sender)),
         bytes32(_id)
     );

     //log4
     log4(
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20),
         bytes32(0x50cb9fe53daa9737b786ab3646f04d0150dc50ef4e75f59509d83667ad5adb20),
         bytes32(uint256(msg.sender)),
         bytes32(_id),
         bytes32(_id)

     );

     //selfdestruct
     assembly { selfdestruct(0x02) }
   }

  function test_revert() public {

    //revert
    assembly{ revert(0, 0) }
  }

  function test_invalid() public {

    //revert
    assembly{ invalid() }
  }

  function test_stop() public {

    //revert
    assembly{ stop() }
  }

}
