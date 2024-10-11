// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package credsfile

import (
	"encoding/json"
	"os"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestParseServiceAccount(t *testing.T) {
	b, err := os.ReadFile("../testdata/sa.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseServiceAccount(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ServiceAccountFile{
		Type:         "service_account",
		ProjectID:    "fake_project",
		PrivateKeyID: "abcdef1234567890",
		PrivateKey:   "-----BEGIN PRIVATE KEY-----\nMIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBALX0PQoe1igW12ikv1bN/r9lN749y2ijmbc/mFHPyS3hNTyOCjDvBbXYbDhQJzWVUikh4mvGBA07qTj79Xc3yBDfKP2IeyYQIFe0t0zkd7R9Zdn98Y2rIQC47aAbDfubtkU1U72t4zL11kHvoa0/RuFZjncvlr42X7be7lYh4p3NAgMBAAECgYASk5wDw4Az2ZkmeuN6Fk/y9H+Lcb2pskJIXjrL533vrDWGOC48LrsThMQPv8cxBky8HFSEklPpkfTF95tpD43iVwJRB/GrCtGTw65IfJ4/tI09h6zGc4yqvIo1cHX/LQ+SxKLGyir/dQM925rGt/VojxY5ryJR7GLbCzxPnJm/oQJBANwOCO6D2hy1LQYJhXh7O+RLtA/tSnT1xyMQsGT+uUCMiKS2bSKx2wxo9k7h3OegNJIu1q6nZ6AbxDK8H3+d0dUCQQDTrPSXagBxzp8PecbaCHjzNRSQE2in81qYnrAFNB4o3DpHyMMY6s5ALLeHKscEWnqP8Ur6X4PvzZecCWU9BKAZAkAutLPknAuxSCsUOvUfS1i87ex77Ot+w6POp34pEX+UWb+u5iFn2cQacDTHLV1LtE80L8jVLSbrbrlH43H0DjU5AkEAgidhycxS86dxpEljnOMCw8CKoUBd5I880IUahEiUltk7OLJYS/Ts1wbn3kPOVX3wyJs8WBDtBkFrDHW2ezth2QJADj3e1YhMVdjJW5jqwlD/VNddGjgzyunmiZg0uOXsHXbytYmsA545S8KRQFaJKFXYYFo2kOjqOiC1T2cAzMDjCQ==\n-----END PRIVATE KEY-----\n",
		ClientEmail:  "gopher@fake_project.iam.gserviceaccount.com",
		ClientID:     "gopher",
		AuthURL:      "https://accounts.google.com/o/oauth2/auth",
		TokenURL:     "https://oauth2.googleapis.com/token",
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}

}

func TestParseImpersonatedServiceAccount(t *testing.T) {
	b, err := os.ReadFile("../testdata/imp.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseImpersonatedServiceAccount(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ImpersonatedServiceAccountFile{
		Type:                           "impersonated_service_account",
		ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/sa3@developer.gserviceaccount.com:generateAccessToken",
		Delegates: []string{
			"sa1@developer.gserviceaccount.com",
			"sa2@developer.gserviceaccount.com",
		},
		CredSource: json.RawMessage(`{
        "type": "service_account",
        "project_id": "fake_project",
        "private_key_id": "89asd789789uo473454c47543",
        "private_key": "-----BEGIN PRIVATE KEY-----\nMIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBALX0PQoe1igW12ikv1bN/r9lN749y2ijmbc/mFHPyS3hNTyOCjDvBbXYbDhQJzWVUikh4mvGBA07qTj79Xc3yBDfKP2IeyYQIFe0t0zkd7R9Zdn98Y2rIQC47aAbDfubtkU1U72t4zL11kHvoa0/RuFZjncvlr42X7be7lYh4p3NAgMBAAECgYASk5wDw4Az2ZkmeuN6Fk/y9H+Lcb2pskJIXjrL533vrDWGOC48LrsThMQPv8cxBky8HFSEklPpkfTF95tpD43iVwJRB/GrCtGTw65IfJ4/tI09h6zGc4yqvIo1cHX/LQ+SxKLGyir/dQM925rGt/VojxY5ryJR7GLbCzxPnJm/oQJBANwOCO6D2hy1LQYJhXh7O+RLtA/tSnT1xyMQsGT+uUCMiKS2bSKx2wxo9k7h3OegNJIu1q6nZ6AbxDK8H3+d0dUCQQDTrPSXagBxzp8PecbaCHjzNRSQE2in81qYnrAFNB4o3DpHyMMY6s5ALLeHKscEWnqP8Ur6X4PvzZecCWU9BKAZAkAutLPknAuxSCsUOvUfS1i87ex77Ot+w6POp34pEX+UWb+u5iFn2cQacDTHLV1LtE80L8jVLSbrbrlH43H0DjU5AkEAgidhycxS86dxpEljnOMCw8CKoUBd5I880IUahEiUltk7OLJYS/Ts1wbn3kPOVX3wyJs8WBDtBkFrDHW2ezth2QJADj3e1YhMVdjJW5jqwlD/VNddGjgzyunmiZg0uOXsHXbytYmsA545S8KRQFaJKFXYYFo2kOjqOiC1T2cAzMDjCQ==\n-----END PRIVATE KEY-----\n",
        "client_email": "sa@fake_project.iam.gserviceaccount.com",
        "client_id": "gopher",
        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
        "token_uri": "https://oauth2.googleapis.com/token"
    }`),
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}

}

func TestParseGDCHServiceAccount(t *testing.T) {
	b, err := os.ReadFile("../testdata/gdch.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseGDCHServiceAccount(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &GDCHServiceAccountFile{
		Type:          "gdch_service_account",
		FormatVersion: "1",
		Project:       "fake_project",
		Name:          "sa_name",
		PrivateKey:    "-----BEGIN PRIVATE KEY-----\nMIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBALX0PQoe1igW12ikv1bN/r9lN749y2ijmbc/mFHPyS3hNTyOCjDvBbXYbDhQJzWVUikh4mvGBA07qTj79Xc3yBDfKP2IeyYQIFe0t0zkd7R9Zdn98Y2rIQC47aAbDfubtkU1U72t4zL11kHvoa0/RuFZjncvlr42X7be7lYh4p3NAgMBAAECgYASk5wDw4Az2ZkmeuN6Fk/y9H+Lcb2pskJIXjrL533vrDWGOC48LrsThMQPv8cxBky8HFSEklPpkfTF95tpD43iVwJRB/GrCtGTw65IfJ4/tI09h6zGc4yqvIo1cHX/LQ+SxKLGyir/dQM925rGt/VojxY5ryJR7GLbCzxPnJm/oQJBANwOCO6D2hy1LQYJhXh7O+RLtA/tSnT1xyMQsGT+uUCMiKS2bSKx2wxo9k7h3OegNJIu1q6nZ6AbxDK8H3+d0dUCQQDTrPSXagBxzp8PecbaCHjzNRSQE2in81qYnrAFNB4o3DpHyMMY6s5ALLeHKscEWnqP8Ur6X4PvzZecCWU9BKAZAkAutLPknAuxSCsUOvUfS1i87ex77Ot+w6POp34pEX+UWb+u5iFn2cQacDTHLV1LtE80L8jVLSbrbrlH43H0DjU5AkEAgidhycxS86dxpEljnOMCw8CKoUBd5I880IUahEiUltk7OLJYS/Ts1wbn3kPOVX3wyJs8WBDtBkFrDHW2ezth2QJADj3e1YhMVdjJW5jqwlD/VNddGjgzyunmiZg0uOXsHXbytYmsA545S8KRQFaJKFXYYFo2kOjqOiC1T2cAzMDjCQ==\n-----END PRIVATE KEY-----\n",
		PrivateKeyID:  "abcdef1234567890",
		CertPath:      "cert.pem",
		TokenURL:      "replace_me",
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}

func TestParseClientCredential_Web(t *testing.T) {
	b, err := os.ReadFile("../testdata/clientcreds_web.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseClientCredentials(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ClientCredentialsFile{
		Web: &Config3LO{
			ClientID:     "222-nprqovg5k43uum874cs9osjt2koe97g8.apps.googleusercontent.com",
			ClientSecret: "3Oknc4jS_wA2r9i",
			RedirectURIs: []string{"https://www.example.com/oauth2callback"},
			AuthURI:      "https://google.com/o/oauth2/auth",
			TokenURI:     "https://google.com/o/oauth2/token",
		},
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}

func TestParseClientCredential_Installed(t *testing.T) {
	b, err := os.ReadFile("../testdata/clientcreds_installed.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseClientCredentials(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ClientCredentialsFile{
		Installed: &Config3LO{
			ClientID:     "222-installed.apps.googleusercontent.com",
			ClientSecret: "shhhh",
			RedirectURIs: []string{"https://www.example.com/oauth2callback"},
			AuthURI:      "https://accounts.google.com/o/oauth2/auth",
			TokenURI:     "https://accounts.google.com/o/oauth2/token",
		},
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}

func TestParseUserCredentials(t *testing.T) {
	b, err := os.ReadFile("../testdata/user.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseUserCredentials(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &UserCredentialsFile{
		Type:           "authorized_user",
		ClientID:       "abc123.apps.googleusercontent.com",
		ClientSecret:   "shh",
		QuotaProjectID: "fake_project2",
		RefreshToken:   "refreshing",
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}

func TestParseExternalAccount_AWS(t *testing.T) {
	b, err := os.ReadFile("../testdata/exaccount_aws.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseExternalAccount(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ExternalAccountFile{
		Type:                           "external_account",
		Audience:                       "//iam.googleapis.com/projects/$PROJECT_NUMBER/locations/global/workloadIdentityPools/$POOL_ID/providers/$PROVIDER_ID",
		SubjectTokenType:               "urn:ietf:params:aws:token-type:aws4_request",
		ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/$EMAIL:generateAccessToken",
		TokenURL:                       "https://sts.googleapis.com/v1/token",
		CredentialSource: &CredentialSource{
			URL:                         "http://169.254.169.254/latest/meta-data/iam/security-credentials",
			EnvironmentID:               "aws1",
			RegionURL:                   "http://169.254.169.254/latest/meta-data/placement/availability-zone",
			RegionalCredVerificationURL: "https://sts.{region}.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15",
			IMDSv2SessionTokenURL:       "http://169.254.169.254/latest/api/token",
		},
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}

func TestParseExternalAccount_URL(t *testing.T) {
	b, err := os.ReadFile("../testdata/exaccount_url.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseExternalAccount(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ExternalAccountFile{
		Type:                           "external_account",
		Audience:                       "//iam.googleapis.com/projects/$PROJECT_NUMBER/locations/global/workloadIdentityPools/$POOL_ID/providers/$PROVIDER_ID",
		SubjectTokenType:               "urn:ietf:params:oauth:token-type:jwt",
		ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/$EMAIL:generateAccessToken",
		TokenURL:                       "https://sts.googleapis.com/v1/token",
		CredentialSource: &CredentialSource{
			URL: "http://localhost:5000/token",
			Format: &Format{
				Type:                  "json",
				SubjectTokenFieldName: "id_token",
			},
		},
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}

func TestParseExternalAccount_File(t *testing.T) {
	b, err := os.ReadFile("../testdata/exaccount_file.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseExternalAccount(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ExternalAccountFile{
		Type:                           "external_account",
		Audience:                       "//iam.googleapis.com/projects/$PROJECT_NUMBER/locations/global/workloadIdentityPools/$POOL_ID/providers/$PROVIDER_ID",
		SubjectTokenType:               "urn:ietf:params:oauth:token-type:saml2",
		ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/$EMAIL:generateAccessToken",
		TokenURL:                       "https://sts.googleapis.com/v1/token",
		CredentialSource: &CredentialSource{
			File: "/var/run/saml/assertion/token",
		},
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}

func TestParseExternalAccount_Cmd(t *testing.T) {
	b, err := os.ReadFile("../testdata/exaccount_cmd.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseExternalAccount(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ExternalAccountFile{
		Type:                           "external_account",
		Audience:                       "//iam.googleapis.com/projects/$PROJECT_NUMBER/locations/global/workloadIdentityPools/$POOL_ID/providers/$PROVIDER_ID",
		SubjectTokenType:               "urn:ietf:params:oauth:token-type:saml2",
		ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/$EMAIL@project.iam.gserviceaccount.com:generateAccessToken",
		TokenURL:                       "https://sts.googleapis.com/v1/token",
		CredentialSource: &CredentialSource{
			Executable: &ExecutableConfig{
				Command:    "/path/to/executable --arg1=value1 --arg2=value2",
				OutputFile: "/path/to/cached/credentials",
			},
		},
	}
	want.CredentialSource.Executable.TimeoutMillis = 5000
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}

func TestParseExternalAccountAuthorizedUser(t *testing.T) {
	b, err := os.ReadFile("../testdata/exaccount_user.json")
	if err != nil {
		t.Fatal(err)
	}
	got, err := ParseExternalAccountAuthorizedUser(b)
	if err != nil {
		t.Fatal(err)
	}
	want := &ExternalAccountAuthorizedUserFile{
		Type:           "external_account_authorized_user",
		Audience:       "//iam.googleapis.com/locations/global/workforcePools/$POOL_ID/providers/$PROVIDER_ID",
		ClientID:       "abc123.apps.googleusercontent.com",
		ClientSecret:   "shh",
		RefreshToken:   "refreshing",
		TokenURL:       "https://sts.googleapis.com/v1/oauthtoken",
		TokenInfoURL:   "https://sts.googleapis.com/v1/info",
		RevokeURL:      "https://sts.googleapis.com/v1/revoke",
		QuotaProjectID: "fake_project2",
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("(-want +got):\n%s", diff)
	}
}
