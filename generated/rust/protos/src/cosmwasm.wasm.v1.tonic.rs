// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    #[derive(Debug, Clone)]
    pub struct MsgClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MsgClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MsgClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> MsgClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            MsgClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn store_code(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgStoreCode>,
        ) -> std::result::Result<tonic::Response<super::MsgStoreCodeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/StoreCode");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "StoreCode"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn instantiate_contract(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgInstantiateContract>,
        ) -> std::result::Result<
            tonic::Response<super::MsgInstantiateContractResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/InstantiateContract");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Msg",
                "InstantiateContract",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn instantiate_contract2(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgInstantiateContract2>,
        ) -> std::result::Result<
            tonic::Response<super::MsgInstantiateContract2Response>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/InstantiateContract2");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Msg",
                "InstantiateContract2",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn execute_contract(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgExecuteContract>,
        ) -> std::result::Result<tonic::Response<super::MsgExecuteContractResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/ExecuteContract");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "ExecuteContract"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn migrate_contract(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgMigrateContract>,
        ) -> std::result::Result<tonic::Response<super::MsgMigrateContractResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/MigrateContract");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "MigrateContract"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_admin(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateAdmin>,
        ) -> std::result::Result<tonic::Response<super::MsgUpdateAdminResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/UpdateAdmin");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "UpdateAdmin"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn clear_admin(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgClearAdmin>,
        ) -> std::result::Result<tonic::Response<super::MsgClearAdminResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/ClearAdmin");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "ClearAdmin"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_instantiate_config(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateInstantiateConfig>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUpdateInstantiateConfigResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmwasm.wasm.v1.Msg/UpdateInstantiateConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Msg",
                "UpdateInstantiateConfig",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_params(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateParams>,
        ) -> std::result::Result<tonic::Response<super::MsgUpdateParamsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/UpdateParams");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "UpdateParams"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn sudo_contract(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSudoContract>,
        ) -> std::result::Result<tonic::Response<super::MsgSudoContractResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/SudoContract");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "SudoContract"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn pin_codes(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgPinCodes>,
        ) -> std::result::Result<tonic::Response<super::MsgPinCodesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/PinCodes");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "PinCodes"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn unpin_codes(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUnpinCodes>,
        ) -> std::result::Result<tonic::Response<super::MsgUnpinCodesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/UnpinCodes");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Msg", "UnpinCodes"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn store_and_instantiate_contract(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgStoreAndInstantiateContract>,
        ) -> std::result::Result<
            tonic::Response<super::MsgStoreAndInstantiateContractResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmwasm.wasm.v1.Msg/StoreAndInstantiateContract",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Msg",
                "StoreAndInstantiateContract",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn remove_code_upload_params_addresses(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgRemoveCodeUploadParamsAddresses>,
        ) -> std::result::Result<
            tonic::Response<super::MsgRemoveCodeUploadParamsAddressesResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmwasm.wasm.v1.Msg/RemoveCodeUploadParamsAddresses",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Msg",
                "RemoveCodeUploadParamsAddresses",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn add_code_upload_params_addresses(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAddCodeUploadParamsAddresses>,
        ) -> std::result::Result<
            tonic::Response<super::MsgAddCodeUploadParamsAddressesResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmwasm.wasm.v1.Msg/AddCodeUploadParamsAddresses",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Msg",
                "AddCodeUploadParamsAddresses",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn store_and_migrate_contract(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgStoreAndMigrateContract>,
        ) -> std::result::Result<
            tonic::Response<super::MsgStoreAndMigrateContractResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmwasm.wasm.v1.Msg/StoreAndMigrateContract",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Msg",
                "StoreAndMigrateContract",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_contract_label(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateContractLabel>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUpdateContractLabelResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Msg/UpdateContractLabel");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Msg",
                "UpdateContractLabel",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    #[derive(Debug, Clone)]
    pub struct QueryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            QueryClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn contract_info(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryContractInfoRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryContractInfoResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/ContractInfo");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Query", "ContractInfo"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn contract_history(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryContractHistoryRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryContractHistoryResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/ContractHistory");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Query", "ContractHistory"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn contracts_by_code(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryContractsByCodeRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryContractsByCodeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/ContractsByCode");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Query", "ContractsByCode"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_contract_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAllContractStateRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryAllContractStateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/AllContractState");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Query",
                "AllContractState",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn raw_contract_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryRawContractStateRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryRawContractStateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/RawContractState");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Query",
                "RawContractState",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn smart_contract_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QuerySmartContractStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QuerySmartContractStateResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/SmartContractState");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Query",
                "SmartContractState",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn code(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCodeRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryCodeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/Code");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Query", "Code"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn codes(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCodesRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryCodesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/Codes");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Query", "Codes"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn pinned_codes(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPinnedCodesRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryPinnedCodesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/PinnedCodes");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Query", "PinnedCodes"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryParamsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn contracts_by_creator(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryContractsByCreatorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryContractsByCreatorResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/ContractsByCreator");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmwasm.wasm.v1.Query",
                "ContractsByCreator",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn build_address(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryBuildAddressRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryBuildAddressResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmwasm.wasm.v1.Query/BuildAddress");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmwasm.wasm.v1.Query", "BuildAddress"));
            self.inner.unary(req, path, codec).await
        }
    }
}
