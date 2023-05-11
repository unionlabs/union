*April 12th, 2023*

In this update, Protobuf definitions have been included for Interchain Security v1 CCV within
the `ibc_proto::interchain_security::ccv` module.

It should also be noted that the return type of `Protobuf::encode{,_length_delimited}_vec`
has been modified from `Result<Vec<u8>, Error>` to `Vec<u8>`.

Furthermore, the version of `tonic` has been raised from 0.8 to 0.9.
