# Google Proxies for Enterprise Certificates (GA)

## Certificate-based-access

If you use [certificate-based access][cba] to protect your Google Cloud resources, the end user [device certificate][clientcert] is one of the credentials that is verified before access to a resource is granted. You can configure Google Cloud to use the device certificates in your operating system key store when verifying access to a resource from the gcloud CLI or Terraform by using the enterprise certificates feature.

## Google Enterprise Certificate Proxies (ECP)

Google Enterprise Certificate Proxies (ECP) are part of the [Google Cloud Zero Trust architecture][zerotrust] that enables mutual authentication with [client-side certificates][clientcert]. This repository contains a set of proxies/modules that can be used by clients or toolings to interact with certificates that are stored in [protected key storage systems][keystore].

To interact the client certificates, application code should not need to use most of these proxies within this repository directly. Instead, the application should leverage the clients and toolings provided by Google such as [Cloud SDK](https://cloud.google.com/sdk) to have a more convenient developer experience.

## Compatibility

The following platforms/keystores are supported by ECP:

- MacOS: __Keychain__
- Linux: __PKCS#11__
- Windows: __MY__

## User Guide

Before using ECP with your application/client, you should complete the policy configurations documented in [Enable CBA for Enterprise Certificate][enterprisecert]. The remainder of this README focuses on client configuration.

### Quick Start

1. Install gcloud CLI (Cloud SDK) at: https://cloud.google.com/sdk/docs/install. Install with the bundled python option enabled.

   1. **Note:** gcloud version 416.0 or newer is required. Version 430.0 or newer is recommended.

1. For macOS and Linux, run the install.sh script after downloading it to complete installation. 
    ```
    $ ./google-cloud-sdk/install.sh
    ```
1. Install the ECP helper component:
    ```
    $ gcloud components install enterprise-certificate-proxy
    ```
1. Initialize ECP certificate configuration:

   * **MacOS** `$ gcloud auth enterprise-certificate-config create macos --issuer=<CERT_ISSUER>`

   * **Linux** `$ gcloud auth enterprise-certificate-config create linux --label=<CERT_LABEL> --module=<PKCS11_MODULE_PATH> --slot=<SLOT_ID>`

   * **Windows** `$ gcloud auth enterprise-certificate-config create windows --issuer=<CERT_ISSUER> --provider=<PROVIDER> --store=<STORE>`

1. Enable usage of client certificates through gcloud CLI config command:
    ```
    $ gcloud config set context_aware/use_client_certificate true
    ```
1. You can now use gcloud to access CBA-protected GCP resources. For example:
    ```
    $ gcloud pubsub topics list
    ```

### Manual Certificate Configuration

ECP relies on a certificate configuration JSON file to read all the metadata information for locating the certificate.
By default, it is named `certificate_config.json` and stored at the following location on the user's device:

* **Linux and MacOS**: `~/.config/gcloud/certificate_config.json`
* **Windows**: `%APPDATA%\gcloud\certificate_config.json`

You can put the JSON file in the location of your choice and set the path to it using:

```
$ gcloud config set context_aware/certificate_config_file_path "<json file path>"
```

Another approach for setting the JSON file location is with the `GOOGLE_API_CERTIFICATE_CONFIG` environment variable.

```
$ export GOOGLE_API_CERTIFICATE_CONFIG="<json file path>"
```

Below are examples of the certificate configuration file:

#### MacOS (Keychain)

```json
{
  "cert_configs": {
    "macos_keychain": {
      "issuer": "YOUR_CERT_ISSUER"
    }
  },
  "libs": {
      "ecp": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/bin/ecp",
      "ecp_client": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/platform/enterprise_cert/libecp.dylib",
      "tls_offload": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/platform/enterprise_cert/libtls_offload.dylib"
  },
  "version": 1
}
```

#### Windows (MyStore)

```json
{
  "cert_configs": {
    "windows_store": {
      "store": "MY",
      "provider": "current_user",
      "issuer": "YOUR_CERT_ISSUER"
    }
  },
  "libs": {
      "ecp": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/bin/ecp.exe",
      "ecp_client": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/platform/enterprise_cert/libecp.dll",
      "tls_offload": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/platform/enterprise_cert/libtls_offload.dll"
  },
  "version": 1
}
```

#### Linux (PKCS#11)

```json
{
  "cert_configs": {
    "pkcs11": {
      "label": "YOUR_TOKEN_LABEL",
      "user_pin": "YOUR_PIN",
      "slot": "YOUR_SLOT",
      "module": "The PKCS #11 module library file path"
    }
  },
  "libs": {
      "ecp": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/bin/ecp",
      "ecp_client": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/platform/enterprise_cert/libecp.so",
      "tls_offload": "[GCLOUD-INSTALL-LOCATION]/google-cloud-sdk/platform/enterprise_cert/libtls_offload.so"
  },
  "version": 1
}
```

### Logging

To enable logging set the `ENABLE_ENTERPRISE_CERTIFICATE_LOGS` environment variable.

#### Example

```
$ export ENABLE_ENTERPRISE_CERTIFICATE_LOGS=1 # Now the enterprise-certificate-proxy will output logs to stdout.
```

## Building ECP binaries from source

For amd64 MacOS, run `./build/scripts/darwin_amd64.sh`. The binaries will be placed in `build/bin/darwin_amd64` folder.

For amd64 Linux, run `./build/scripts/linux_amd64.sh`. The binaries will be placed in `build/bin/linux_amd64` folder.

For amd64 Windows, in powershell terminal, run `.\build\scripts\windows_amd64.ps1`. The binaries will be placed in `build\bin\windows_amd64` folder.
Note that gcc is required for compiling the Windows shared library. The easiest way to get gcc on Windows is to download Mingw64, and add "gcc.exe" to the powershell path.

## Contributing

Contributions to this library are always welcome and highly encouraged. See the [CONTRIBUTING](./CONTRIBUTING.md) documentation for more information on how to get started.

## License

Apache - See [LICENSE](./LICENSE) for more information.

[cba]: https://cloud.google.com/beyondcorp-enterprise/docs/securing-resources-with-certificate-based-access
[clientcert]: https://en.wikipedia.org/wiki/Client_certificate
[openssl]: https://wiki.openssl.org/index.php/Binaries
[keystore]: https://en.wikipedia.org/wiki/Key_management
[cloudsdk]: https://cloud.google.com/sdk
[enterprisecert]: https://cloud.google.com/beyondcorp-enterprise/docs/enable-cba-enterprise-certificates
[zerotrust]: https://cloud.google.com/blog/topics/developers-practitioners/zero-trust-and-beyondcorp-google-cloud

