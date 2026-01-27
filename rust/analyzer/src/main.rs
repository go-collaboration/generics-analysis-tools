#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_span;

use rustc_ast::visit::{self, Visitor};
use rustc_ast::*;
use rustc_driver::{Callbacks, Compilation, run_compiler};
use rustc_interface::interface;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

struct AstWalker {
    generic_ty: u64,
    non_generic_ty: u64,
    generic_fun: u64,
    non_generic_fun: u64,
}

impl AstWalker {
    fn new() -> Self {
        Self {
            generic_ty: 0,
            non_generic_ty: 0,
            generic_fun: 0,
            non_generic_fun: 0,
        }
    }

    fn print(&self) {
        println!(
            "{},{},{},{}",
            self.generic_ty, self.non_generic_ty, self.generic_fun, self.non_generic_fun
        );
    }
}

impl<'ast> Visitor<'ast> for AstWalker {
    fn visit_item(&mut self, item: &'ast Item) {
        match &item.kind {
            ItemKind::Fn(f) => {
                if is_generic(&f.generics) {
                    self.generic_fun += 1;
                } else {
                    self.non_generic_fun += 1
                }
            }
            ItemKind::Struct(_, g, _) => {
                if is_generic(g) {
                    self.generic_ty += 1;
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::Enum(_, g, _) => {
                if is_generic(g) {
                    self.generic_ty += 1;
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::TyAlias(tyalias) => {
                if is_generic(&tyalias.generics) {
                    self.generic_ty += 1;
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::Union(_, g, _) => {
                if is_generic(g) {
                    self.generic_ty += 1;
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::Trait(t) => {
                if is_generic(&t.generics) {
                    self.generic_ty += 1;
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::TraitAlias(trait_alias) => {
                if is_generic(&trait_alias.generics) {
                    self.generic_ty += 1;
                } else {
                    self.non_generic_ty += 1
                }
            }
            _ => {}
        }

        visit::walk_item(self, item);
    }
}

fn is_generic(g: &Generics) -> bool {
    !g.params.is_empty()
}

struct MyCallbacks {
    walker: Arc<Mutex<AstWalker>>,
}

impl Callbacks for MyCallbacks {
    fn after_crate_root_parsing<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        _krate: &mut Crate,
    ) -> Compilation {
        for item in &_krate.items {
            self.walker.lock().unwrap().visit_item(item);
        }
        Compilation::Stop
    }
}

#[derive(Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Deserialize)]
struct WCargoToml {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    edition: String,
}

fn parse_edition(path: &str) -> std::io::Result<Option<String>> {
    // println!("memoize {}", path);
    let content = std::fs::read_to_string(path)?;
    if let Ok(ct) = toml::from_str::<CargoToml>(content.as_str()) {
        return Ok(Some(ct.package.edition));
    } else if let Ok(wct) = toml::from_str::<WCargoToml>(content.as_str()) {
        // Workspace
        return Ok(Some(wct.workspace.package.edition));
    }
    Ok(None)
}

static EDITION_CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

fn parse_edition_memoized(path: &str) -> Option<String> {
    let cache = EDITION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(cached) = cache.lock().unwrap().get(path).cloned() {
        return cached;
    }
    let result = parse_edition(path).unwrap_or(None);
    cache
        .lock()
        .unwrap()
        .insert(path.to_string(), result.clone());
    result
}

fn get_edition(path: &str) -> String {
    // This solution is horrible.
    // But good enough for... well, let's see how long.
    let mut remaining = path;
    while let Some((dirname, _)) = remaining.rsplit_once('/') {
        if let Some(edition) = parse_edition_memoized(format!("{}/Cargo.toml", dirname).as_str()) {
            return edition.clone();
        }
        remaining = dirname;
    }

    if let Some(edition) = parse_edition_memoized("Cargo.toml") {
        return edition.clone();
    }

    "2024".to_string()
}

fn main() {
    let walker = AstWalker::new();
    let mut cb = MyCallbacks {
        walker: Arc::new(Mutex::new(walker)),
    };

    for (i, file) in std::env::args().enumerate() {
        if i == 0 {
            continue;
        } else if i == 1 {
            std::env::set_current_dir(file).unwrap();
            continue;
        }

        let edition = get_edition(file.as_str());
        let rustc_args = vec![
            "rustc".to_string(),
            file.clone(),
            "--crate-type=bin".to_string(),
            format!("--edition={}", edition).to_string(),
        ];

        let _ = rustc_driver::catch_fatal_errors(|| run_compiler(&rustc_args, &mut cb));
    }

    cb.walker.lock().unwrap().print();
}
