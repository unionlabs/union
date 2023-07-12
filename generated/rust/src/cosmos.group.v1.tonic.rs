// @generated
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
            D: std::convert::TryInto<tonic::transport::Endpoint>,
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
        pub async fn group_info(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupInfoRequest>,
        ) -> Result<tonic::Response<super::QueryGroupInfoResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/GroupInfo");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn group_policy_info(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupPolicyInfoRequest>,
        ) -> Result<tonic::Response<super::QueryGroupPolicyInfoResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/GroupPolicyInfo");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn group_members(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupMembersRequest>,
        ) -> Result<tonic::Response<super::QueryGroupMembersResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/GroupMembers");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn groups_by_admin(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupsByAdminRequest>,
        ) -> Result<tonic::Response<super::QueryGroupsByAdminResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/GroupsByAdmin");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn group_policies_by_group(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupPoliciesByGroupRequest>,
        ) -> Result<tonic::Response<super::QueryGroupPoliciesByGroupResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/GroupPoliciesByGroup");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn group_policies_by_admin(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupPoliciesByAdminRequest>,
        ) -> Result<tonic::Response<super::QueryGroupPoliciesByAdminResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/GroupPoliciesByAdmin");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryProposalRequest>,
        ) -> Result<tonic::Response<super::QueryProposalResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/Proposal");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn proposals_by_group_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryProposalsByGroupPolicyRequest>,
        ) -> Result<tonic::Response<super::QueryProposalsByGroupPolicyResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.group.v1.Query/ProposalsByGroupPolicy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn vote_by_proposal_voter(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryVoteByProposalVoterRequest>,
        ) -> Result<tonic::Response<super::QueryVoteByProposalVoterResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/VoteByProposalVoter");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn votes_by_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryVotesByProposalRequest>,
        ) -> Result<tonic::Response<super::QueryVotesByProposalResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/VotesByProposal");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn votes_by_voter(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryVotesByVoterRequest>,
        ) -> Result<tonic::Response<super::QueryVotesByVoterResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/VotesByVoter");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn groups_by_member(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupsByMemberRequest>,
        ) -> Result<tonic::Response<super::QueryGroupsByMemberResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/GroupsByMember");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn tally_result(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTallyResultRequest>,
        ) -> Result<tonic::Response<super::QueryTallyResultResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/TallyResult");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn groups(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupsRequest>,
        ) -> Result<tonic::Response<super::QueryGroupsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Query/Groups");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod query_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServer.
    #[async_trait]
    pub trait Query: Send + Sync + 'static {
        async fn group_info(
            &self,
            request: tonic::Request<super::QueryGroupInfoRequest>,
        ) -> Result<tonic::Response<super::QueryGroupInfoResponse>, tonic::Status>;
        async fn group_policy_info(
            &self,
            request: tonic::Request<super::QueryGroupPolicyInfoRequest>,
        ) -> Result<tonic::Response<super::QueryGroupPolicyInfoResponse>, tonic::Status>;
        async fn group_members(
            &self,
            request: tonic::Request<super::QueryGroupMembersRequest>,
        ) -> Result<tonic::Response<super::QueryGroupMembersResponse>, tonic::Status>;
        async fn groups_by_admin(
            &self,
            request: tonic::Request<super::QueryGroupsByAdminRequest>,
        ) -> Result<tonic::Response<super::QueryGroupsByAdminResponse>, tonic::Status>;
        async fn group_policies_by_group(
            &self,
            request: tonic::Request<super::QueryGroupPoliciesByGroupRequest>,
        ) -> Result<tonic::Response<super::QueryGroupPoliciesByGroupResponse>, tonic::Status>;
        async fn group_policies_by_admin(
            &self,
            request: tonic::Request<super::QueryGroupPoliciesByAdminRequest>,
        ) -> Result<tonic::Response<super::QueryGroupPoliciesByAdminResponse>, tonic::Status>;
        async fn proposal(
            &self,
            request: tonic::Request<super::QueryProposalRequest>,
        ) -> Result<tonic::Response<super::QueryProposalResponse>, tonic::Status>;
        async fn proposals_by_group_policy(
            &self,
            request: tonic::Request<super::QueryProposalsByGroupPolicyRequest>,
        ) -> Result<tonic::Response<super::QueryProposalsByGroupPolicyResponse>, tonic::Status>;
        async fn vote_by_proposal_voter(
            &self,
            request: tonic::Request<super::QueryVoteByProposalVoterRequest>,
        ) -> Result<tonic::Response<super::QueryVoteByProposalVoterResponse>, tonic::Status>;
        async fn votes_by_proposal(
            &self,
            request: tonic::Request<super::QueryVotesByProposalRequest>,
        ) -> Result<tonic::Response<super::QueryVotesByProposalResponse>, tonic::Status>;
        async fn votes_by_voter(
            &self,
            request: tonic::Request<super::QueryVotesByVoterRequest>,
        ) -> Result<tonic::Response<super::QueryVotesByVoterResponse>, tonic::Status>;
        async fn groups_by_member(
            &self,
            request: tonic::Request<super::QueryGroupsByMemberRequest>,
        ) -> Result<tonic::Response<super::QueryGroupsByMemberResponse>, tonic::Status>;
        async fn tally_result(
            &self,
            request: tonic::Request<super::QueryTallyResultRequest>,
        ) -> Result<tonic::Response<super::QueryTallyResultResponse>, tonic::Status>;
        async fn groups(
            &self,
            request: tonic::Request<super::QueryGroupsRequest>,
        ) -> Result<tonic::Response<super::QueryGroupsResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct QueryServer<T: Query> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Query> QueryServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServer<T>
    where
        T: Query,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/cosmos.group.v1.Query/GroupInfo" => {
                    #[allow(non_camel_case_types)]
                    struct GroupInfoSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryGroupInfoRequest> for GroupInfoSvc<T> {
                        type Response = super::QueryGroupInfoResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryGroupInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).group_info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GroupInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/GroupPolicyInfo" => {
                    #[allow(non_camel_case_types)]
                    struct GroupPolicyInfoSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryGroupPolicyInfoRequest>
                        for GroupPolicyInfoSvc<T>
                    {
                        type Response = super::QueryGroupPolicyInfoResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryGroupPolicyInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).group_policy_info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GroupPolicyInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/GroupMembers" => {
                    #[allow(non_camel_case_types)]
                    struct GroupMembersSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryGroupMembersRequest> for GroupMembersSvc<T> {
                        type Response = super::QueryGroupMembersResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryGroupMembersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).group_members(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GroupMembersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/GroupsByAdmin" => {
                    #[allow(non_camel_case_types)]
                    struct GroupsByAdminSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryGroupsByAdminRequest>
                        for GroupsByAdminSvc<T>
                    {
                        type Response = super::QueryGroupsByAdminResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryGroupsByAdminRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).groups_by_admin(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GroupsByAdminSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/GroupPoliciesByGroup" => {
                    #[allow(non_camel_case_types)]
                    struct GroupPoliciesByGroupSvc<T: Query>(pub Arc<T>);
                    impl<T: Query>
                        tonic::server::UnaryService<super::QueryGroupPoliciesByGroupRequest>
                        for GroupPoliciesByGroupSvc<T>
                    {
                        type Response = super::QueryGroupPoliciesByGroupResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryGroupPoliciesByGroupRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).group_policies_by_group(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GroupPoliciesByGroupSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/GroupPoliciesByAdmin" => {
                    #[allow(non_camel_case_types)]
                    struct GroupPoliciesByAdminSvc<T: Query>(pub Arc<T>);
                    impl<T: Query>
                        tonic::server::UnaryService<super::QueryGroupPoliciesByAdminRequest>
                        for GroupPoliciesByAdminSvc<T>
                    {
                        type Response = super::QueryGroupPoliciesByAdminResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryGroupPoliciesByAdminRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).group_policies_by_admin(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GroupPoliciesByAdminSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/Proposal" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryProposalRequest> for ProposalSvc<T> {
                        type Response = super::QueryProposalResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryProposalRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).proposal(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/ProposalsByGroupPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalsByGroupPolicySvc<T: Query>(pub Arc<T>);
                    impl<T: Query>
                        tonic::server::UnaryService<super::QueryProposalsByGroupPolicyRequest>
                        for ProposalsByGroupPolicySvc<T>
                    {
                        type Response = super::QueryProposalsByGroupPolicyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryProposalsByGroupPolicyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).proposals_by_group_policy(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProposalsByGroupPolicySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/VoteByProposalVoter" => {
                    #[allow(non_camel_case_types)]
                    struct VoteByProposalVoterSvc<T: Query>(pub Arc<T>);
                    impl<T: Query>
                        tonic::server::UnaryService<super::QueryVoteByProposalVoterRequest>
                        for VoteByProposalVoterSvc<T>
                    {
                        type Response = super::QueryVoteByProposalVoterResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryVoteByProposalVoterRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).vote_by_proposal_voter(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VoteByProposalVoterSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/VotesByProposal" => {
                    #[allow(non_camel_case_types)]
                    struct VotesByProposalSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryVotesByProposalRequest>
                        for VotesByProposalSvc<T>
                    {
                        type Response = super::QueryVotesByProposalResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryVotesByProposalRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).votes_by_proposal(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VotesByProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/VotesByVoter" => {
                    #[allow(non_camel_case_types)]
                    struct VotesByVoterSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryVotesByVoterRequest> for VotesByVoterSvc<T> {
                        type Response = super::QueryVotesByVoterResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryVotesByVoterRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).votes_by_voter(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VotesByVoterSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/GroupsByMember" => {
                    #[allow(non_camel_case_types)]
                    struct GroupsByMemberSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryGroupsByMemberRequest>
                        for GroupsByMemberSvc<T>
                    {
                        type Response = super::QueryGroupsByMemberResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryGroupsByMemberRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).groups_by_member(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GroupsByMemberSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/TallyResult" => {
                    #[allow(non_camel_case_types)]
                    struct TallyResultSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryTallyResultRequest> for TallyResultSvc<T> {
                        type Response = super::QueryTallyResultResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryTallyResultRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).tally_result(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TallyResultSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Query/Groups" => {
                    #[allow(non_camel_case_types)]
                    struct GroupsSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryGroupsRequest> for GroupsSvc<T> {
                        type Response = super::QueryGroupsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryGroupsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).groups(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GroupsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Query> Clone for QueryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Query> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Query> tonic::server::NamedService for QueryServer<T> {
        const NAME: &'static str = "cosmos.group.v1.Query";
    }
}
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
            D: std::convert::TryInto<tonic::transport::Endpoint>,
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
        pub async fn create_group(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateGroup>,
        ) -> Result<tonic::Response<super::MsgCreateGroupResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/CreateGroup");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_group_members(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateGroupMembers>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupMembersResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/UpdateGroupMembers");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_group_admin(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateGroupAdmin>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupAdminResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/UpdateGroupAdmin");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_group_metadata(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateGroupMetadata>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupMetadataResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/UpdateGroupMetadata");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_group_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateGroupPolicy>,
        ) -> Result<tonic::Response<super::MsgCreateGroupPolicyResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/CreateGroupPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_group_with_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateGroupWithPolicy>,
        ) -> Result<tonic::Response<super::MsgCreateGroupWithPolicyResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/CreateGroupWithPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_group_policy_admin(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateGroupPolicyAdmin>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupPolicyAdminResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/UpdateGroupPolicyAdmin");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_group_policy_decision_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateGroupPolicyDecisionPolicy>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupPolicyDecisionPolicyResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.group.v1.Msg/UpdateGroupPolicyDecisionPolicy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_group_policy_metadata(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateGroupPolicyMetadata>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupPolicyMetadataResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.group.v1.Msg/UpdateGroupPolicyMetadata",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn submit_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSubmitProposal>,
        ) -> Result<tonic::Response<super::MsgSubmitProposalResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/SubmitProposal");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn withdraw_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgWithdrawProposal>,
        ) -> Result<tonic::Response<super::MsgWithdrawProposalResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/WithdrawProposal");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn vote(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgVote>,
        ) -> Result<tonic::Response<super::MsgVoteResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/Vote");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn exec(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgExec>,
        ) -> Result<tonic::Response<super::MsgExecResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/Exec");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn leave_group(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgLeaveGroup>,
        ) -> Result<tonic::Response<super::MsgLeaveGroupResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.group.v1.Msg/LeaveGroup");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod msg_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MsgServer.
    #[async_trait]
    pub trait Msg: Send + Sync + 'static {
        async fn create_group(
            &self,
            request: tonic::Request<super::MsgCreateGroup>,
        ) -> Result<tonic::Response<super::MsgCreateGroupResponse>, tonic::Status>;
        async fn update_group_members(
            &self,
            request: tonic::Request<super::MsgUpdateGroupMembers>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupMembersResponse>, tonic::Status>;
        async fn update_group_admin(
            &self,
            request: tonic::Request<super::MsgUpdateGroupAdmin>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupAdminResponse>, tonic::Status>;
        async fn update_group_metadata(
            &self,
            request: tonic::Request<super::MsgUpdateGroupMetadata>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupMetadataResponse>, tonic::Status>;
        async fn create_group_policy(
            &self,
            request: tonic::Request<super::MsgCreateGroupPolicy>,
        ) -> Result<tonic::Response<super::MsgCreateGroupPolicyResponse>, tonic::Status>;
        async fn create_group_with_policy(
            &self,
            request: tonic::Request<super::MsgCreateGroupWithPolicy>,
        ) -> Result<tonic::Response<super::MsgCreateGroupWithPolicyResponse>, tonic::Status>;
        async fn update_group_policy_admin(
            &self,
            request: tonic::Request<super::MsgUpdateGroupPolicyAdmin>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupPolicyAdminResponse>, tonic::Status>;
        async fn update_group_policy_decision_policy(
            &self,
            request: tonic::Request<super::MsgUpdateGroupPolicyDecisionPolicy>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupPolicyDecisionPolicyResponse>, tonic::Status>;
        async fn update_group_policy_metadata(
            &self,
            request: tonic::Request<super::MsgUpdateGroupPolicyMetadata>,
        ) -> Result<tonic::Response<super::MsgUpdateGroupPolicyMetadataResponse>, tonic::Status>;
        async fn submit_proposal(
            &self,
            request: tonic::Request<super::MsgSubmitProposal>,
        ) -> Result<tonic::Response<super::MsgSubmitProposalResponse>, tonic::Status>;
        async fn withdraw_proposal(
            &self,
            request: tonic::Request<super::MsgWithdrawProposal>,
        ) -> Result<tonic::Response<super::MsgWithdrawProposalResponse>, tonic::Status>;
        async fn vote(
            &self,
            request: tonic::Request<super::MsgVote>,
        ) -> Result<tonic::Response<super::MsgVoteResponse>, tonic::Status>;
        async fn exec(
            &self,
            request: tonic::Request<super::MsgExec>,
        ) -> Result<tonic::Response<super::MsgExecResponse>, tonic::Status>;
        async fn leave_group(
            &self,
            request: tonic::Request<super::MsgLeaveGroup>,
        ) -> Result<tonic::Response<super::MsgLeaveGroupResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct MsgServer<T: Msg> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Msg> MsgServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MsgServer<T>
    where
        T: Msg,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/cosmos.group.v1.Msg/CreateGroup" => {
                    #[allow(non_camel_case_types)]
                    struct CreateGroupSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgCreateGroup> for CreateGroupSvc<T> {
                        type Response = super::MsgCreateGroupResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgCreateGroup>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_group(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateGroupSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/UpdateGroupMembers" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateGroupMembersSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgUpdateGroupMembers>
                        for UpdateGroupMembersSvc<T>
                    {
                        type Response = super::MsgUpdateGroupMembersResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgUpdateGroupMembers>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_group_members(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateGroupMembersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/UpdateGroupAdmin" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateGroupAdminSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgUpdateGroupAdmin> for UpdateGroupAdminSvc<T> {
                        type Response = super::MsgUpdateGroupAdminResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgUpdateGroupAdmin>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_group_admin(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateGroupAdminSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/UpdateGroupMetadata" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateGroupMetadataSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgUpdateGroupMetadata>
                        for UpdateGroupMetadataSvc<T>
                    {
                        type Response = super::MsgUpdateGroupMetadataResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgUpdateGroupMetadata>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_group_metadata(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateGroupMetadataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/CreateGroupPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct CreateGroupPolicySvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgCreateGroupPolicy> for CreateGroupPolicySvc<T> {
                        type Response = super::MsgCreateGroupPolicyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgCreateGroupPolicy>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_group_policy(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateGroupPolicySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/CreateGroupWithPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct CreateGroupWithPolicySvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgCreateGroupWithPolicy>
                        for CreateGroupWithPolicySvc<T>
                    {
                        type Response = super::MsgCreateGroupWithPolicyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgCreateGroupWithPolicy>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).create_group_with_policy(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateGroupWithPolicySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/UpdateGroupPolicyAdmin" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateGroupPolicyAdminSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgUpdateGroupPolicyAdmin>
                        for UpdateGroupPolicyAdminSvc<T>
                    {
                        type Response = super::MsgUpdateGroupPolicyAdminResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgUpdateGroupPolicyAdmin>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).update_group_policy_admin(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateGroupPolicyAdminSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/UpdateGroupPolicyDecisionPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateGroupPolicyDecisionPolicySvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg>
                        tonic::server::UnaryService<super::MsgUpdateGroupPolicyDecisionPolicy>
                        for UpdateGroupPolicyDecisionPolicySvc<T>
                    {
                        type Response = super::MsgUpdateGroupPolicyDecisionPolicyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgUpdateGroupPolicyDecisionPolicy>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).update_group_policy_decision_policy(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateGroupPolicyDecisionPolicySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/UpdateGroupPolicyMetadata" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateGroupPolicyMetadataSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgUpdateGroupPolicyMetadata>
                        for UpdateGroupPolicyMetadataSvc<T>
                    {
                        type Response = super::MsgUpdateGroupPolicyMetadataResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgUpdateGroupPolicyMetadata>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).update_group_policy_metadata(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateGroupPolicyMetadataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/SubmitProposal" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitProposalSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgSubmitProposal> for SubmitProposalSvc<T> {
                        type Response = super::MsgSubmitProposalResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgSubmitProposal>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).submit_proposal(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubmitProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/WithdrawProposal" => {
                    #[allow(non_camel_case_types)]
                    struct WithdrawProposalSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgWithdrawProposal> for WithdrawProposalSvc<T> {
                        type Response = super::MsgWithdrawProposalResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgWithdrawProposal>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).withdraw_proposal(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WithdrawProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/Vote" => {
                    #[allow(non_camel_case_types)]
                    struct VoteSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgVote> for VoteSvc<T> {
                        type Response = super::MsgVoteResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgVote>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).vote(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VoteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/Exec" => {
                    #[allow(non_camel_case_types)]
                    struct ExecSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgExec> for ExecSvc<T> {
                        type Response = super::MsgExecResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgExec>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).exec(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ExecSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.group.v1.Msg/LeaveGroup" => {
                    #[allow(non_camel_case_types)]
                    struct LeaveGroupSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgLeaveGroup> for LeaveGroupSvc<T> {
                        type Response = super::MsgLeaveGroupResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgLeaveGroup>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).leave_group(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LeaveGroupSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Msg> Clone for MsgServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Msg> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Msg> tonic::server::NamedService for MsgServer<T> {
        const NAME: &'static str = "cosmos.group.v1.Msg";
    }
}
