use anyhow::Result;
use pathdiff::diff_paths;
use std::{fs, path::Path};
use term_table::{row::Row, Table, TableStyle};

use crate::store::{ALL_MODULES_STORE_ROOT, STORE_ROOT};

pub(crate) fn ls() {
    let mut table = Table::new();
    table.style = TableStyle::simple();
    table.add_row(Row::new(vec!["Name", "Location"]));
    for module in fs::read_dir(ALL_MODULES_STORE_ROOT.as_path())
        .expect("could not read store root")
        .flatten()
    {
        if let Some(module_name) = module.file_name().to_str() {
            table.add_row(Row::new(vec![
                module_name,
                &module_store_location(&module.path()).expect("invalid filename"),
            ]));
        }
    }
    println!("{}", table.render());
}

// Given a module location in the directory where symlinks to all
// modules are located, give back the URI resembling where this module
// was pulled from, or the path to the local filesystem where this
// module is located if it wasn't pulled from a remote location
fn module_store_location(module_path: &Path) -> Result<String> {
    let module_path = std::fs::read_link(module_path)?;
    // If this module was added from somehwere in the filesystem
    // (outside of the store), just return it as it is
    if !module_path.starts_with(STORE_ROOT.as_path()) {
        return Ok(format!(
            "{} (not in the store)",
            module_path.to_str().expect("invalid path")
        ));
    }
    let path = diff_paths(module_path, STORE_ROOT.as_path()).expect("failed to diff paths");
    let mut component_iterator = path.components();
    let scheme = component_iterator.next().expect("invalid path");
    Ok(component_iterator.fold(
        format!("{}:/", scheme.as_os_str().to_str().expect("invalid path")),
        |acc, element| {
            format!(
                "{}/{}",
                acc,
                element.as_os_str().to_str().expect("invalid path")
            )
        },
    ))
}
