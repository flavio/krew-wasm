use std::path::PathBuf;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_common::WasiCtx;
use wasi_outbound_http_wasmtime::OutboundHttp;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::*;

use crate::errors::{KrewWapcError, Result};

struct Context {
    pub wasi: WasiCtx,
    pub runtime_data: Option<OutboundHttp>,
}

fn build_ctx(runtime_data: Option<OutboundHttp>) -> Context {
    let wasi = default_wasi();
    Context { wasi, runtime_data }
}

fn default_wasi() -> WasiCtx {
    let mut ctx = WasiCtxBuilder::new().inherit_stdio().inherit_stdout();
    ctx = ctx.inherit_args().unwrap();
    ctx = ctx.inherit_env().unwrap();
    ctx = ctx
        .preopened_dir(
            Dir::open_ambient_dir("/home/flavio", ambient_authority()).unwrap(),
            "/home/flavio",
        )
        .unwrap();

    ctx.build()
}

fn kube_api_server_url() -> anyhow::Result<String> {
    let config = kube_conf::Config::load_default()
        .map_err(|e| anyhow::anyhow!("kubeconf: cannot read config: {:?}", e))?;

    let kube_ctx = config
        .get_current_context()
        .ok_or(anyhow::anyhow!("kubeconf: no default kubernetes context"))?;

    let cluster = kube_ctx
        .get_cluster(&config)
        .ok_or(anyhow::anyhow!("kubeconf: cannot find cluster definition"))?;

    Ok(cluster.server)
}

pub(crate) fn run_plugin(wasm_module_path: PathBuf) -> Result<()> {
    if !wasm_module_path.exists() {
        return Err(KrewWapcError::GenericError(format!(
            "Cannot find {}",
            wasm_module_path.to_str().unwrap()
        )));
    }

    let allowed_hosts = vec![kube_api_server_url()?];
    let outbound_http = OutboundHttp::new(Some(allowed_hosts));
    let ctx = build_ctx(Some(outbound_http));

    // Modules can be compiled through either the text or binary format
    let mut config = Config::new();
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.wasm_multi_memory(true);
    config.wasm_module_linking(true);

    let engine = Engine::new(&config).unwrap();
    let module = Module::from_file(&engine, wasm_module_path)?;
    let mut linker = Linker::<Context>::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context| &mut cx.wasi)?;
    let mut store = Store::new(&engine, ctx);

    wasi_outbound_http_wasmtime::add_to_linker(&mut linker, |ctx| -> &mut OutboundHttp {
        ctx.runtime_data.as_mut().unwrap()
    })?;

    let instance = linker.instantiate(&mut store, &module)?;
    // Instantiation of a module requires specifying its imports and then
    // afterwards we can fetch exports by name, as well as asserting the
    // type signature of the function with `get_typed_func`.
    let start = instance.get_typed_func::<(), (), _>(&mut store, "_start")?;

    // And finally we can call the wasm!
    start.call(&mut store, ()).map_err(|e| {
        if let Some(exit_code) = e.i32_exit_status() {
            KrewWapcError::PlugingExitError { code: exit_code }
        } else {
            KrewWapcError::GenericWasmEvalError(e.display_reason().to_string())
        }
    })?;

    Ok(())
}
