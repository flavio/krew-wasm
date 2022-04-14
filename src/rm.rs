use crate::store::STORE_ROOT;

use std::path::PathBuf;

// This removes the module from the store, and then removes both
// links, the `all` toplevel link of the module itself, and the
// kubectl-plugin link to `krew-wasm`. When the module is removed from
// the store, it also cleans up the structure up to the root of the
// store, so no empty folders are kept around in the store
pub(crate) fn rm(module: &str) {
    let (module_paths, module_store_path) =
        crate::store::all_module_paths(module).expect("failed to get module paths for module");

    // Unlink files that can be directly removed without any extra
    // cleanup: the toplevel "all" module and the symlink for the
    // kubectl-plugin
    for path in module_paths {
        #[allow(unused_must_use)]
        {
            std::fs::remove_file(path);
        }
    }

    if !module_store_path.starts_with(STORE_ROOT.as_path()) {
        // Nothing to clean in the store itself, given this module
        // comes from another part of the filesystem. Just return.
        return;
    }

    #[allow(unused_must_use)]
    {
        std::fs::remove_file(&module_store_path);
    }

    // Clean up parent directories in the store up to its root
    {
        let mut prefix = STORE_ROOT.clone();
        let module_leading_store_components = module_store_path
            .iter()
            .map(|component| {
                prefix = prefix.join(component);
                prefix.clone()
            })
            .collect::<Vec<PathBuf>>();

        module_leading_store_components
            .iter()
            .rev()
            .skip(1) // module file -- already unlinked
            .take(module_store_path.components().count() - STORE_ROOT.components().count() - 1 /* krew-wasm-store */)
            .for_each(|component| {
                #[allow(unused_must_use)]
                {
                    // try to clean up empty dirs. Ignore errors.
                    std::fs::remove_dir(component);
                }
            })
    }
}
