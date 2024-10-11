/*
 *
 * Copyright 2021 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package xds_test

import (
	"context"
	"fmt"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	xdscreds "google.golang.org/grpc/credentials/xds"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/e2e/setup"
	"google.golang.org/grpc/resolver"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3tlspb "github.com/envoyproxy/go-control-plane/envoy/extensions/transport_sockets/tls/v3"
	"github.com/google/uuid"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

func (s) TestUnmarshalListener_WithUpdateValidatorFunc(t *testing.T) {
	const (
		serviceName                     = "my-service-client-side-xds"
		missingIdentityProviderInstance = "missing-identity-provider-instance"
		missingRootProviderInstance     = "missing-root-provider-instance"
	)

	tests := []struct {
		name           string
		securityConfig *v3corepb.TransportSocket
		wantErr        bool
	}{
		{
			name: "both identity and root providers are not present in bootstrap",
			securityConfig: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
								InstanceName: missingIdentityProviderInstance,
							},
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
								ValidationContext: &v3tlspb.CertificateValidationContext{
									CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName: missingRootProviderInstance,
									},
								},
							},
						},
					}),
				},
			},
			wantErr: true,
		},
		{
			name: "only identity provider is not present in bootstrap",
			securityConfig: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
								InstanceName: missingIdentityProviderInstance,
							},
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
								ValidationContext: &v3tlspb.CertificateValidationContext{
									CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName: e2e.ServerSideCertProviderInstance,
									},
								},
							},
						},
					}),
				},
			},
			wantErr: true,
		},
		{
			name: "only root provider is not present in bootstrap",
			securityConfig: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
								InstanceName: e2e.ServerSideCertProviderInstance,
							},
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
								ValidationContext: &v3tlspb.CertificateValidationContext{
									CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName: missingRootProviderInstance,
									},
								},
							},
						},
					}),
				},
			},
			wantErr: true,
		},
		{
			name: "both identity and root providers are present in bootstrap",
			securityConfig: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
								InstanceName: e2e.ServerSideCertProviderInstance,
							},
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
								ValidationContext: &v3tlspb.CertificateValidationContext{
									CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName: e2e.ServerSideCertProviderInstance,
									},
								},
							},
						},
					}),
				},
			},
			wantErr: false,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			managementServer, nodeID, bootstrapContents, xdsResolver := setup.ManagementServerAndResolver(t)

			lis, cleanup2 := setupGRPCServer(t, bootstrapContents)
			defer cleanup2()

			// Grab the host and port of the server and create client side xDS
			// resources corresponding to it.
			host, port, err := hostPortFromListener(lis)
			if err != nil {
				t.Fatalf("failed to retrieve host and port of server: %v", err)
			}

			// Create xDS resources to be consumed on the client side. This
			// includes the listener, route configuration, cluster (with
			// security configuration) and endpoint resources.
			resources := e2e.DefaultClientResources(e2e.ResourceParams{
				DialTarget: serviceName,
				NodeID:     nodeID,
				Host:       host,
				Port:       port,
				SecLevel:   e2e.SecurityLevelMTLS,
			})

			// Create an inbound xDS listener resource for the server side.
			inboundLis := e2e.DefaultServerListener(host, port, e2e.SecurityLevelMTLS, "routeName")
			for _, fc := range inboundLis.GetFilterChains() {
				fc.TransportSocket = test.securityConfig
			}
			resources.Listeners = append(resources.Listeners, inboundLis)

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := managementServer.Update(ctx, resources); err != nil {
				t.Fatal(err)
			}

			// Create client-side xDS credentials with an insecure fallback.
			creds, err := xdscreds.NewClientCredentials(xdscreds.ClientOptions{FallbackCreds: insecure.NewCredentials()})
			if err != nil {
				t.Fatal(err)
			}

			// Create a ClientConn with the xds scheme and make an RPC.
			cc, err := grpc.DialContext(ctx, fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(creds), grpc.WithResolvers(xdsResolver))
			if err != nil {
				t.Fatalf("failed to dial local test server: %v", err)
			}
			defer cc.Close()

			// Make a context with a shorter timeout from the top level test
			// context for cases where we expect failures.
			timeout := defaultTestTimeout
			if test.wantErr {
				timeout = defaultTestShortTimeout
			}
			ctx, cancel = context.WithTimeout(ctx, timeout)
			defer cancel()
			client := testgrpc.NewTestServiceClient(cc)
			if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); (err != nil) != test.wantErr {
				t.Fatalf("EmptyCall() returned err: %v, wantErr %v", err, test.wantErr)
			}
		})
	}
}

func (s) TestUnmarshalCluster_WithUpdateValidatorFunc(t *testing.T) {
	const (
		serviceName                     = "my-service-client-side-xds"
		missingIdentityProviderInstance = "missing-identity-provider-instance"
		missingRootProviderInstance     = "missing-root-provider-instance"
	)

	tests := []struct {
		name           string
		securityConfig *v3corepb.TransportSocket
		wantErr        bool
	}{
		{
			name: "both identity and root providers are not present in bootstrap",
			securityConfig: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
								InstanceName: missingIdentityProviderInstance,
							},
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
								ValidationContext: &v3tlspb.CertificateValidationContext{
									CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName: missingRootProviderInstance,
									},
								},
							},
						},
					}),
				},
			},
			wantErr: true,
		},
		{
			name: "only identity provider is not present in bootstrap",
			securityConfig: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
								InstanceName: missingIdentityProviderInstance,
							},
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
								ValidationContext: &v3tlspb.CertificateValidationContext{
									CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName: e2e.ClientSideCertProviderInstance,
									},
								},
							},
						},
					}),
				},
			},
			wantErr: true,
		},
		{
			name: "only root provider is not present in bootstrap",
			securityConfig: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
								InstanceName: e2e.ClientSideCertProviderInstance,
							},
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
								ValidationContext: &v3tlspb.CertificateValidationContext{
									CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName: missingRootProviderInstance,
									},
								},
							},
						},
					}),
				},
			},
			wantErr: true,
		},
		{
			name: "both identity and root providers are present in bootstrap",
			securityConfig: &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							TlsCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
								InstanceName: e2e.ClientSideCertProviderInstance,
							},
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContext{
								ValidationContext: &v3tlspb.CertificateValidationContext{
									CaCertificateProviderInstance: &v3tlspb.CertificateProviderPluginInstance{
										InstanceName: e2e.ClientSideCertProviderInstance,
									},
								},
							},
						},
					}),
				},
			},
			wantErr: false,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

			// Create bootstrap configuration pointing to the above management
			// server with certificate provider configuration.
			nodeID := uuid.New().String()
			bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

			// Create an xDS resolver with the above bootstrap configuration.
			var xdsResolver resolver.Builder
			if newResolver := internal.NewXDSResolverWithConfigForTesting; newResolver != nil {
				var err error
				xdsResolver, err = newResolver.(func([]byte) (resolver.Builder, error))(bootstrapContents)
				if err != nil {
					t.Fatalf("Failed to create xDS resolver for testing: %v", err)
				}
			}

			server := stubserver.StartTestService(t, nil)
			defer server.Stop()

			// This creates a `Cluster` resource with a security config which
			// refers to `e2e.ClientSideCertProviderInstance` for both root and
			// identity certs.
			resources := e2e.DefaultClientResources(e2e.ResourceParams{
				DialTarget: serviceName,
				NodeID:     nodeID,
				Host:       "localhost",
				Port:       testutils.ParsePort(t, server.Address),
				SecLevel:   e2e.SecurityLevelMTLS,
			})
			resources.Clusters[0].TransportSocket = test.securityConfig
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := managementServer.Update(ctx, resources); err != nil {
				t.Fatal(err)
			}

			cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsResolver))
			if err != nil {
				t.Fatalf("failed to dial local test server: %v", err)
			}
			defer cc.Close()

			// Make a context with a shorter timeout from the top level test
			// context for cases where we expect failures.
			timeout := defaultTestTimeout
			if test.wantErr {
				timeout = defaultTestShortTimeout
			}
			ctx2, cancel2 := context.WithTimeout(ctx, timeout)
			defer cancel2()
			client := testgrpc.NewTestServiceClient(cc)
			if _, err := client.EmptyCall(ctx2, &testpb.Empty{}, grpc.WaitForReady(true)); (err != nil) != test.wantErr {
				t.Fatalf("EmptyCall() returned err: %v, wantErr %v", err, test.wantErr)
			}
		})
	}
}
