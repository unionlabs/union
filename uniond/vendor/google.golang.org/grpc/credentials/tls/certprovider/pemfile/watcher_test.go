/*
 *
 * Copyright 2020 gRPC authors.
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

package pemfile

import (
	"context"
	"fmt"
	"os"
	"path"
	"testing"
	"time"

	"google.golang.org/grpc/credentials/tls/certprovider"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/testdata"
)

const (
	// These are the names of files inside temporary directories, which the
	// plugin is asked to watch.
	certFile = "cert.pem"
	keyFile  = "key.pem"
	rootFile = "ca.pem"

	defaultTestRefreshDuration = 100 * time.Millisecond
	defaultTestTimeout         = 5 * time.Second
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func compareKeyMaterial(got, want *certprovider.KeyMaterial) error {
	if len(got.Certs) != len(want.Certs) {
		return fmt.Errorf("keyMaterial certs = %+v, want %+v", got, want)
	}
	for i := 0; i < len(got.Certs); i++ {
		if !got.Certs[i].Leaf.Equal(want.Certs[i].Leaf) {
			return fmt.Errorf("keyMaterial certs = %+v, want %+v", got, want)
		}
	}

	if gotR, wantR := got.Roots, want.Roots; !gotR.Equal(wantR) {
		return fmt.Errorf("keyMaterial roots = %v, want %v", gotR, wantR)
	}

	return nil
}

// TestNewProvider tests the NewProvider() function with different inputs.
func (s) TestNewProvider(t *testing.T) {
	tests := []struct {
		desc      string
		options   Options
		wantError bool
	}{
		{
			desc:      "No credential files specified",
			options:   Options{},
			wantError: true,
		},
		{
			desc: "Only identity cert is specified",
			options: Options{
				CertFile: testdata.Path("x509/client1_cert.pem"),
			},
			wantError: true,
		},
		{
			desc: "Only identity key is specified",
			options: Options{
				KeyFile: testdata.Path("x509/client1_key.pem"),
			},
			wantError: true,
		},
		{
			desc: "Identity cert/key pair is specified",
			options: Options{
				KeyFile:  testdata.Path("x509/client1_key.pem"),
				CertFile: testdata.Path("x509/client1_cert.pem"),
			},
		},
		{
			desc: "Only root certs are specified",
			options: Options{
				RootFile: testdata.Path("x509/client_ca_cert.pem"),
			},
		},
		{
			desc: "Everything is specified",
			options: Options{
				KeyFile:  testdata.Path("x509/client1_key.pem"),
				CertFile: testdata.Path("x509/client1_cert.pem"),
				RootFile: testdata.Path("x509/client_ca_cert.pem"),
			},
			wantError: false,
		},
	}
	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			provider, err := NewProvider(test.options)
			if (err != nil) != test.wantError {
				t.Fatalf("NewProvider(%v) = %v, want %v", test.options, err, test.wantError)
			}
			if err != nil {
				return
			}
			provider.Close()
		})
	}
}

// wrappedDistributor wraps a distributor and pushes on a channel whenever new
// key material is pushed to the distributor.
type wrappedDistributor struct {
	*certprovider.Distributor
	distCh *testutils.Channel
}

func newWrappedDistributor(distCh *testutils.Channel) *wrappedDistributor {
	return &wrappedDistributor{
		distCh:      distCh,
		Distributor: certprovider.NewDistributor(),
	}
}

func (wd *wrappedDistributor) Set(km *certprovider.KeyMaterial, err error) {
	wd.Distributor.Set(km, err)
	wd.distCh.Send(nil)
}

func createTmpFile(t *testing.T, src, dst string) {
	t.Helper()

	data, err := os.ReadFile(src)
	if err != nil {
		t.Fatalf("os.ReadFile(%q) failed: %v", src, err)
	}
	if err := os.WriteFile(dst, data, os.ModePerm); err != nil {
		t.Fatalf("os.WriteFile(%q) failed: %v", dst, err)
	}
	t.Logf("Wrote file at: %s", dst)
	t.Logf("%s", string(data))
}

// createTempDirWithFiles creates a temporary directory under the system default
// tempDir with the given dirSuffix. It also reads from certSrc, keySrc and
// rootSrc files are creates appropriate files under the newly create tempDir.
// Returns the name of the created tempDir.
func createTmpDirWithFiles(t *testing.T, dirSuffix, certSrc, keySrc, rootSrc string) string {
	t.Helper()

	// Create a temp directory. Passing an empty string for the first argument
	// uses the system temp directory.
	dir, err := os.MkdirTemp("", dirSuffix)
	if err != nil {
		t.Fatalf("os.MkdirTemp() failed: %v", err)
	}
	t.Logf("Using tmpdir: %s", dir)

	createTmpFile(t, testdata.Path(certSrc), path.Join(dir, certFile))
	createTmpFile(t, testdata.Path(keySrc), path.Join(dir, keyFile))
	createTmpFile(t, testdata.Path(rootSrc), path.Join(dir, rootFile))
	return dir
}

// initializeProvider performs setup steps common to all tests (except the one
// which uses symlinks).
func initializeProvider(t *testing.T, testName string) (string, certprovider.Provider, *testutils.Channel, func()) {
	t.Helper()

	// Override the newDistributor to one which pushes on a channel that we
	// can block on.
	origDistributorFunc := newDistributor
	distCh := testutils.NewChannel()
	d := newWrappedDistributor(distCh)
	newDistributor = func() distributor { return d }

	// Create a new provider to watch the files in tmpdir.
	dir := createTmpDirWithFiles(t, testName+"*", "x509/client1_cert.pem", "x509/client1_key.pem", "x509/client_ca_cert.pem")
	opts := Options{
		CertFile:        path.Join(dir, certFile),
		KeyFile:         path.Join(dir, keyFile),
		RootFile:        path.Join(dir, rootFile),
		RefreshDuration: defaultTestRefreshDuration,
	}
	prov, err := NewProvider(opts)
	if err != nil {
		t.Fatalf("NewProvider(%+v) failed: %v", opts, err)
	}

	// Make sure the provider picks up the files and pushes the key material on
	// to the distributors.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for i := 0; i < 2; i++ {
		// Since we have root and identity certs, we need to make sure the
		// update is pushed on both of them.
		if _, err := distCh.Receive(ctx); err != nil {
			t.Fatalf("timeout waiting for provider to read files and push key material to distributor: %v", err)
		}
	}

	return dir, prov, distCh, func() {
		newDistributor = origDistributorFunc
		prov.Close()
	}
}

// TestProvider_NoUpdate tests the case where a file watcher plugin is created
// successfully, and the underlying files do not change. Verifies that the
// plugin does not push new updates to the distributor in this case.
func (s) TestProvider_NoUpdate(t *testing.T) {
	_, prov, distCh, cancel := initializeProvider(t, "no_update")
	defer cancel()

	// Make sure the provider is healthy and returns key material.
	ctx, cc := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cc()
	if _, err := prov.KeyMaterial(ctx); err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}

	// Files haven't change. Make sure no updates are pushed by the provider.
	sCtx, sc := context.WithTimeout(context.Background(), 2*defaultTestRefreshDuration)
	defer sc()
	if _, err := distCh.Receive(sCtx); err == nil {
		t.Fatal("new key material pushed to distributor when underlying files did not change")
	}
}

// TestProvider_UpdateSuccess tests the case where a file watcher plugin is
// created successfully and the underlying files change. Verifies that the
// changes are picked up by the provider.
func (s) TestProvider_UpdateSuccess(t *testing.T) {
	dir, prov, distCh, cancel := initializeProvider(t, "update_success")
	defer cancel()

	// Make sure the provider is healthy and returns key material.
	ctx, cc := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cc()
	km1, err := prov.KeyMaterial(ctx)
	if err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}

	// Change only the root file.
	createTmpFile(t, testdata.Path("x509/server_ca_cert.pem"), path.Join(dir, rootFile))
	if _, err := distCh.Receive(ctx); err != nil {
		t.Fatal("timeout waiting for new key material to be pushed to the distributor")
	}

	// Make sure update is picked up.
	km2, err := prov.KeyMaterial(ctx)
	if err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}
	if err := compareKeyMaterial(km1, km2); err == nil {
		t.Fatal("expected provider to return new key material after update to underlying file")
	}

	// Change only cert/key files.
	createTmpFile(t, testdata.Path("x509/client2_cert.pem"), path.Join(dir, certFile))
	createTmpFile(t, testdata.Path("x509/client2_key.pem"), path.Join(dir, keyFile))
	if _, err := distCh.Receive(ctx); err != nil {
		t.Fatal("timeout waiting for new key material to be pushed to the distributor")
	}

	// Make sure update is picked up.
	km3, err := prov.KeyMaterial(ctx)
	if err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}
	if err := compareKeyMaterial(km2, km3); err == nil {
		t.Fatal("expected provider to return new key material after update to underlying file")
	}
}

// TestProvider_UpdateSuccessWithSymlink tests the case where a file watcher
// plugin is created successfully to watch files through a symlink and the
// symlink is updates to point to new files. Verifies that the changes are
// picked up by the provider.
func (s) TestProvider_UpdateSuccessWithSymlink(t *testing.T) {
	// Override the newDistributor to one which pushes on a channel that we
	// can block on.
	origDistributorFunc := newDistributor
	distCh := testutils.NewChannel()
	d := newWrappedDistributor(distCh)
	newDistributor = func() distributor { return d }
	defer func() { newDistributor = origDistributorFunc }()

	// Create two tempDirs with different files.
	dir1 := createTmpDirWithFiles(t, "update_with_symlink1_*", "x509/client1_cert.pem", "x509/client1_key.pem", "x509/client_ca_cert.pem")
	dir2 := createTmpDirWithFiles(t, "update_with_symlink2_*", "x509/server1_cert.pem", "x509/server1_key.pem", "x509/server_ca_cert.pem")

	// Create a symlink under a new tempdir, and make it point to dir1.
	tmpdir, err := os.MkdirTemp("", "test_symlink_*")
	if err != nil {
		t.Fatalf("os.MkdirTemp() failed: %v", err)
	}
	symLinkName := path.Join(tmpdir, "test_symlink")
	if err := os.Symlink(dir1, symLinkName); err != nil {
		t.Fatalf("failed to create symlink to %q: %v", dir1, err)
	}

	// Create a provider which watches the files pointed to by the symlink.
	opts := Options{
		CertFile:        path.Join(symLinkName, certFile),
		KeyFile:         path.Join(symLinkName, keyFile),
		RootFile:        path.Join(symLinkName, rootFile),
		RefreshDuration: defaultTestRefreshDuration,
	}
	prov, err := NewProvider(opts)
	if err != nil {
		t.Fatalf("NewProvider(%+v) failed: %v", opts, err)
	}
	defer prov.Close()

	// Make sure the provider picks up the files and pushes the key material on
	// to the distributors.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for i := 0; i < 2; i++ {
		// Since we have root and identity certs, we need to make sure the
		// update is pushed on both of them.
		if _, err := distCh.Receive(ctx); err != nil {
			t.Fatalf("timeout waiting for provider to read files and push key material to distributor: %v", err)
		}
	}
	km1, err := prov.KeyMaterial(ctx)
	if err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}

	// Update the symlink to point to dir2.
	symLinkTmpName := path.Join(tmpdir, "test_symlink.tmp")
	if err := os.Symlink(dir2, symLinkTmpName); err != nil {
		t.Fatalf("failed to create symlink to %q: %v", dir2, err)
	}
	if err := os.Rename(symLinkTmpName, symLinkName); err != nil {
		t.Fatalf("failed to update symlink: %v", err)
	}

	// Make sure the provider picks up the new files and pushes the key material
	// on to the distributors.
	for i := 0; i < 2; i++ {
		// Since we have root and identity certs, we need to make sure the
		// update is pushed on both of them.
		if _, err := distCh.Receive(ctx); err != nil {
			t.Fatalf("timeout waiting for provider to read files and push key material to distributor: %v", err)
		}
	}
	km2, err := prov.KeyMaterial(ctx)
	if err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}

	if err := compareKeyMaterial(km1, km2); err == nil {
		t.Fatal("expected provider to return new key material after symlink update")
	}
}

// TestProvider_UpdateFailure_ThenSuccess tests the case where updating cert/key
// files fail. Verifies that the failed update does not push anything on the
// distributor. Then the update succeeds, and the test verifies that the key
// material is updated.
func (s) TestProvider_UpdateFailure_ThenSuccess(t *testing.T) {
	dir, prov, distCh, cancel := initializeProvider(t, "update_failure")
	defer cancel()

	// Make sure the provider is healthy and returns key material.
	ctx, cc := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cc()
	km1, err := prov.KeyMaterial(ctx)
	if err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}

	// Update only the cert file. The key file is left unchanged. This should
	// lead to these two files being not compatible with each other. This
	// simulates the case where the watching goroutine might catch the files in
	// the midst of an update.
	createTmpFile(t, testdata.Path("x509/server1_cert.pem"), path.Join(dir, certFile))

	// Since the last update left the files in an incompatible state, the update
	// should not be picked up by our provider.
	sCtx, sc := context.WithTimeout(context.Background(), 2*defaultTestRefreshDuration)
	defer sc()
	if _, err := distCh.Receive(sCtx); err == nil {
		t.Fatal("new key material pushed to distributor when underlying files did not change")
	}

	// The provider should return key material corresponding to the old state.
	km2, err := prov.KeyMaterial(ctx)
	if err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}
	if err := compareKeyMaterial(km1, km2); err != nil {
		t.Fatalf("expected provider to not update key material: %v", err)
	}

	// Update the key file to match the cert file.
	createTmpFile(t, testdata.Path("x509/server1_key.pem"), path.Join(dir, keyFile))

	// Make sure update is picked up.
	if _, err := distCh.Receive(ctx); err != nil {
		t.Fatal("timeout waiting for new key material to be pushed to the distributor")
	}
	km3, err := prov.KeyMaterial(ctx)
	if err != nil {
		t.Fatalf("provider.KeyMaterial() failed: %v", err)
	}
	if err := compareKeyMaterial(km2, km3); err == nil {
		t.Fatal("expected provider to return new key material after update to underlying file")
	}
}
