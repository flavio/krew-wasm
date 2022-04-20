This crate implements the WIT definitions provided by [this repository](https://github.com/flavio/wasi-experimental-toolkit/tree/wasi-outbount-http-add-request-config/crates/wasi-outbound-http-defs).

The exporter implementation is done differently compared to the [upstream one](https://github.com/flavio/wasi-experimental-toolkit/tree/wasi-outbount-http-add-request-config/crates/http-wasmtime), because it takes into account some "quirks" required when interacting with a Kubernetes API server:

* Connect to API server by IP address. Some kubernetes distributions like minikube and k3d generate a kubeconfig file that expresses the API server as an IP address. When rustls is being used, the certificate used by the API address cannot be verified because of a long standing issue with the WebPKI crate. This crate implements a workaround for this bug
