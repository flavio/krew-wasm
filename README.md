# krew-wasm

krew-wasm is an experimental project that demonstrates how kubectl plugins could
be written using WebAssembly and WASI.

krew-wasm takes inspiration from the [Krew](https://krew.sigs.k8s.io/) project,
the official plugin manager for kubectl. However, on top of being a completely
different codebase, krew-wasm **does not** aim to replace Krew. It's a complementary
tool that can be used alongside with Krew.

The sole purpose of krew-wasm is to manage kubectl plugins written using WebAssembly
and WASI.

## How it works

kubectl plugins are simple binaries that follow this naming convention:
`kubectl-<name of the plugin>`.
Once placed in a known `$PATH`, the plugin becames available by just invoking
`kubectl <name of the plugin>`.

krew-wasm can download kubectl WebAssembly plugins and make them discoverable
to the kubectl tool.
This is achieved by creating a symbolic link for each managed plugin. This symbolic
link is named `kubectl-<name of the plugin>` but, instead of pointing to the
WebAssembly module, it points to the `krew-wasm` executable.

Once invoked, `krew-wasm` determines its usage mode which could either be a
"direct invocation" (when the user invokes the `krew-wasm` binary to manage plugins)
or it could be a "wrapper invocation" done via `kubectl`.

When invoked in "wrapper mode, krew-wasm takes care of loading the WebAssembly
plugin and invoking it. krew-wasm works as a WebAssembly host, and takes care of
setting up the WASI environment used by the plugin.

### The WebAssembly runtime

Thanks to WASI, the plugins are granted access to the following portions of the host:

  * Standard input, output and error
  * Environment variables
  * The whole home directory of the user, with read and write privileges

Because of that, the plugins can read the default kubeconfig file, or the one
referenced by the `KUBECONFIG` environment variable (as long as it's located
inside of the user home directory).

The plugins can also interact with the user like any regular cli application.

#### Network access

Currently, the WebAssembly WASI specification doesn't cover network access for
guest modules. However, kubectl plugins need to interact with the Kubernetes API
server.

Network access is granted via an experimental interface provided by the krew-wasm.

The interface is defined via the [`WIT`](https://github.com/bytecodealliance/wit-bindgen/blob/c9b113be144ba8418fb4a86a5993e0c44a7d64b3/WIT.md)
format and can be found
[here](https://github.com/flavio/wasi-experimental-toolkit/tree/wasi-outbount-http-add-request-config/crates/wasi-outbound-http-defs/wit).

Currently, the plugins are allowed to make http requests **only** against the
Kubernetes API server that is defined inside of the default kubeconfig file.

## Installation

Download the right pre-built binary from the GitHub Releases page and it to
your `$PATH`.

## Usage

### List plugins

The list of installed plugins can be listed with this command:

```console
krew-wasm list
```

### Download and install a plugin

Plugins are distributed via OCI registries, the same infrastructure used to distribute
container images.

Plugins can be downloaded and made available with this command:

```console
krew-wasm pull <OCI reference>
```

For example:

```console
krew-wasm pull ghcr.io/flavio/krew-wasm-plugins/kubewarden:latest
```

This command downloads and installs the `latest` version of the `kubectl-kubewarden`
that is published inside of the `ghcr.io/flavio/krew-wasm-plugins/kubewarden`
registry.

### Uninstall plugins

Plugins can be removed from the system by using the following command:

```console
krew-wasm rm <name of the plugin>
```

The name of the plugin can be obtained by using the `list` command.

