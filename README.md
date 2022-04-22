# krew-wasm

krew-wasm is an experimental project that demonstrates how kubectl plugins could
be written using WebAssembly and WASI.

krew-wasm takes inspiration from the [Krew](https://krew.sigs.k8s.io/) project,
the official plugin manager for kubectl. However, on top of being a completely
different codebase, krew-wasm **does not** aim to replace Krew. It's a complementary
tool that can be used alongside with Krew.

The sole purpose of krew-wasm is to manage kubectl plugins written using
[WebAssembly](https://webassembly.org/)
and [WASI](https://wasi.dev/).

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

When invoked in "wrapper mode", krew-wasm takes care of loading the WebAssembly
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

Network access is granted via an experimental interface provided by
the krew-wasm runtime.

The interface is defined via the [`WIT`](https://github.com/bytecodealliance/wit-bindgen/blob/c9b113be144ba8418fb4a86a5993e0c44a7d64b3/WIT.md)
format and can be found
[here](https://github.com/flavio/wasi-experimental-toolkit/tree/wasi-outbount-http-add-request-config/crates/wasi-outbound-http-defs/wit).

Currently, plugins are allowed to make http requests **only** against the
Kubernetes API server that is defined inside of the default kubeconfig file.

## Why?

> Why would someone be interested in writing kubectl plugins in this way?

That's a legitimate question. I think the main advantages are:

  * WebAssembly is portable: you don't have to build your plugin for all the
    possible operating systems and architectures the end users might want.
    Portability remains a problem of krew-wasm maintainers, not of policy authors.
  * Security: WebAssembly modules are executed inside of a dedicated sandbox. They
    cannot see other processes running on the host nor have access to the host
    filesystem.
    Note well: currently this POC gives access to the whole user home directory
    to each plugin. We have plans to change that and give access only to the
    kubeconfig file and all the resources referenced by it.
  * Size: the majority of kubectl plugins are written using Go, which produces
    big binaries (the average size of a kubectl plugin is around ~9Mb). A Rust
    plugin compiled into WebAssembly is almost half the size of it (~4.2 Mb).
    This can be even trimmed down a bit by running a WebAssembly binary optimizer
    like `wasm-opt`.

Last but not least, this was fun! We have high expectations for WebAssembly and
WASI, this is just another way to prove that to the world 🤓

## Limitations

Currently the biggest pain point is network access. The network interface introduced
by krew-wasm kinda solves the problem, but there are some limitations.

Regular Kubernetes client libraries cannot be used. With Rust the [`k8s-openapi`](https://crates.io/crates/k8s-openapi)
can be used, but just because it uses a [sans-io](https://sans-io.readthedocs.io/)
approach.
We don't think other languages feature a library similar to `k8s-openapi`.

Finally, all the long polling requests issued via `k8s-openapi` (like all the `watch`
operations) are not going to work because of limitations with how data
is exchanged between the WebAssembly host and the guest.

All these limitations should be solved once WASI implements the [socket proposal](https://github.com/WebAssembly/wasi-sockets).
That, among other things, should allow the usage of regular Kubernetes clients.

## Installation

Download the right pre-built binary from the GitHub Releases page and
install it in your `$PATH`.

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

## Writing a plugin

> Note well: this is still a POC, the documentation is limited, but will be
> improved in the future

Plugins are written as regular WebAssembly modules leveraging the WASI interface.

[This](https://wasmbyexample.dev/examples/wasi-hello-world/wasi-hello-world.rust.en-us.html)
website has many examples about "Hello World" WASI programs.

A demo policy, that interacts with the API server, can be found [here](https://github.com/flavio/kubectl-kubewarden/).

## Acknowledgements

The idea about writing kubectl plugins using WebAssembly
was born by [Rafael Fernández López](https://github.com/ereslibre)
during a brain storming session with
[Flavio Castelli](https://github.com/flavio).
