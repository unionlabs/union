use prost::Message;

pub trait CdcEncode {
    fn cdc_encode(self) -> Vec<u8>;
}

impl CdcEncode for String {
    fn cdc_encode(self) -> Vec<u8> {
        protos::google::protobuf::StringValue { value: self }.encode_to_vec()
    }
}

impl CdcEncode for i64 {
    fn cdc_encode(self) -> Vec<u8> {
        protos::google::protobuf::Int64Value { value: self }.encode_to_vec()
    }
}

impl CdcEncode for Vec<u8> {
    fn cdc_encode(self) -> Vec<u8> {
        protos::google::protobuf::BytesValue { value: self }.encode_to_vec()
    }
}
