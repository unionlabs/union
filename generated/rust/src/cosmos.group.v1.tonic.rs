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
