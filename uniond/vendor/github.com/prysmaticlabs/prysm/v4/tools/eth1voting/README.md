# eth1voting

This tool can be used to query a Prysm node to print eth1voting information.

Flags:
```
  -beacon string
        gRPC address of the Prysm beacon node (default "127.0.0.1:4000")
  -genesis uint
        Genesis time. mainnet=1606824023, prater=1616508000 (default 1606824023)
```

Usage:
```
bazel run //tools/eth1voting -- -beacon=127.0.0.1:4000 -genesis=1606824023
```

Example response
```
Looking back from current epoch 71132 back to 71104                                                                                                           
Next period starts at epoch 71168 (3h50m1.542950367s)                                                                                                         
====Eth1Data Voting Report====                                                                                                                                
                                                                                                                                                              
Total votes: 732                                                                                                                                              
                                                                                                                                                              
Block Hashes                                                                                                                                                  
0xd3c1132b8cebb601872a32277af708ec0f026b74e814956b0f1456516234c48e=656                                                                                        
0x6b418e33ac6a7d181408af0306c475adff9ea28a43b97e1f85e82070988f9288=11                                                                                         
0x1957243971efcd4897e219769b235807f7c2fb726efb78917478416c7ecccf4a=1
0x8e13121c1e4e86c134cf89dbb43971fd7e17d5388a88b6e8cfd2271a403dd80f=1
0x38d0c96aa2d7dc31b26923637023f0ec1bfa2c661f6194f39bad8862b4a8d8a0=63

Deposit Roots
0x7145788308cae4edce32d919d55363b9bf33e26598ce08d591da1b973fa3f5bc=638
0x1cd4bea86b7d65ddd49541f828cf2e7b0068cd00bb7e4888aaf4630b11b15b15=11
0x3abb67c5bba4d0c3cbf7f2aacbd01b8fcf59840c8ce36f82e8d03bb51a987677=1
0xf44e4b56d4190d89d897e5641d0cea3a8adbfd419d1258a9180732f0a0caee26=64
0xdd868cee8ef3eaee5a6707fc2e68682a8d7d77fdc349cf1fd8f8f88303a1faa8=18

Deposit Counts
66629=638
66623=11
66624=1
66627=64
66628=18

Votes
deposit_root:"݆\x8c\xee\x8e\xf3\xea\xeeZg\x07\xfc.hh*\x8d}w\xfd\xc3I\xcf\x1f\xd8\xf8\xf8\x83\x03\xa1\xfa\xa8"  deposit_count:66628  block_hash:"\xd3\xc1\x13+\x
8c\xeb\xb6\x01\x87*2'z\xf7\x08\xec\x0f\x02kt\xe8\x14\x95k\x0f\x14VQb4Ď"=18
deposit_root:"qEx\x83\x08\xca\xe4\xed\xce2\xd9\x19\xd5Sc\xb9\xbf3\xe2e\x98\xce\x08Ց\xda\x1b\x97?\xa3\xf5\xbc"  deposit_count:66629  block_hash:"\xd3\xc1\x13+\
x8c\xeb\xb6\x01\x87*2'z\xf7\x08\xec\x0f\x02kt\xe8\x14\x95k\x0f\x14VQb4Ď"=638
deposit_root:"\x1cԾ\xa8k}e\xddԕA\xf8(\xcf.{\x00h\xcd\x00\xbb~H\x88\xaa\xf4c\x0b\x11\xb1[\x15"  deposit_count:66623  block_hash:"kA\x8e3\xacj}\x18\x14\x08\xaf\
x03\x06\xc4u\xad\xff\x9e\xa2\x8aC\xb9~\x1f\x85\xe8 p\x98\x8f\x92\x88"=11
deposit_root:":\xbbgŻ\xa4\xd0\xc3\xcb\xf7\xf2\xaa\xcb\xd0\x1b\x8f\xcfY\x84\x0c\x8c\xe3o\x82\xe8\xd0;\xb5\x1a\x98vw"  deposit_count:66624  block_hash:"\x19W$9q
\xef\xcdH\x97\xe2\x19v\x9b#X\x07\xf7\xc2\xfbrn\xfbx\x91txAl~\xcc\xcfJ"=1
deposit_root:"\xf4NKV\xd4\x19\r\x89ؗ\xe5d\x1d\x0c\xea:\x8a\xdb\xfdA\x9d\x12X\xa9\x18\x072\xf0\xa0\xca\xee&"  deposit_count:66627  block_hash:"\x8e\x13\x12\x1c\
x1eN\x86\xc14ω۴9q\xfd~\x17\xd58\x8a\x88\xb6\xe8\xcf\xd2'\x1a@=\xd8\x0f"=1
deposit_root:"\xf4NKV\xd4\x19\r\x89ؗ\xe5d\x1d\x0c\xea:\x8a\xdb\xfdA\x9d\x12X\xa9\x18\x072\xf0\xa0\xca\xee&"  deposit_count:66627  block_hash:"8\xd0\xc9j\xa2\xd
7\xdc1\xb2i#cp#\xf0\xec\x1b\xfa,f\x1fa\x94󛭈b\xb4\xa8ؠ"=63
```