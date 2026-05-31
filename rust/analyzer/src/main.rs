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

struct Counter {
    c: Vec<u64>,
}

impl Counter {
    fn new() -> Self {
        return Self { c: Vec::new() };
    }

    fn count(&mut self, i: usize) {
        while self.c.len() <= i {
            self.c.push(0);
        }
        self.c[i] += 1;
    }

    fn to_string(&self) -> String {
        let mut out = String::from("0:0");
        if !self.c.is_empty() {
            out = self.c[0].to_string();
            if self.c.len() > 1 {
                for x in self.c[1..].iter() {
                    out += format!(":{}", x).as_str();
                }
            } else {
                out += ":0"
            }
        }
        out
    }
}

struct AstWalker {
    parse_errors: u64,
    generic_ty: u64,
    non_generic_ty: u64,
    generic_structs: u64,
    generic_traits: u64,
    generic_ty_aliases: u64,
    generic_other: u64,
    generic_fun: u64,
    generic_functions: u64,
    generic_methods: u64,
    non_generic_fun: u64,
    ty_param_counter: Counter,
    ty_param_counter_structs: Counter,
    ty_param_counter_traits: Counter,
    ty_param_counter_functions: Counter,
    ty_param_counter_methods: Counter,
    ty_param_counter_ty_aliases: Counter,
    ty_param_counter_other: Counter,
    non_trivial_type_bounds: u64,
    trivial_type_bounds: u64,
    non_trivial_type_bounds_structs: u64,
    trivial_type_bounds_structs: u64,
    non_trivial_type_bounds_traits: u64,
    trivial_type_bounds_traits: u64,
    non_trivial_type_bounds_functions: u64,
    trivial_type_bounds_functions: u64,
    non_trivial_type_bounds_methods: u64,
    trivial_type_bounds_methods: u64,
    non_trivial_type_bounds_ty_aliases: u64,
    trivial_type_bounds_ty_aliases: u64,
    non_trivial_type_bounds_other: u64,
    trivial_type_bounds_other: u64,
}

impl AstWalker {
    fn new() -> Self {
        Self {
            parse_errors: 0,
            generic_ty: 0,
            non_generic_ty: 0,
            generic_structs: 0,
            generic_traits: 0,
            generic_ty_aliases: 0,
            generic_other: 0,
            generic_fun: 0,
            non_generic_fun: 0,
            generic_functions: 0,
            generic_methods: 0,
            ty_param_counter: Counter::new(),
            ty_param_counter_structs: Counter::new(),
            ty_param_counter_traits: Counter::new(),
            ty_param_counter_functions: Counter::new(),
            ty_param_counter_methods: Counter::new(),
            ty_param_counter_ty_aliases: Counter::new(),
            ty_param_counter_other: Counter::new(),
            non_trivial_type_bounds: 0,
            trivial_type_bounds: 0,
            non_trivial_type_bounds_structs: 0,
            trivial_type_bounds_structs: 0,
            non_trivial_type_bounds_traits: 0,
            trivial_type_bounds_traits: 0,
            non_trivial_type_bounds_functions: 0,
            trivial_type_bounds_functions: 0,
            non_trivial_type_bounds_methods: 0,
            trivial_type_bounds_methods: 0,
            non_trivial_type_bounds_ty_aliases: 0,
            trivial_type_bounds_ty_aliases: 0,
            non_trivial_type_bounds_other: 0,
            trivial_type_bounds_other: 0,
        }
    }

    fn print(&self) {
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.parse_errors,
            self.generic_ty,
            self.non_generic_ty,
            self.generic_structs,
            self.generic_traits,
            self.generic_ty_aliases,
            self.generic_other,
            self.generic_fun,
            self.non_generic_fun,
            self.generic_functions,
            self.generic_methods,
            self.ty_param_counter.to_string(),
            self.ty_param_counter_structs.to_string(),
            self.ty_param_counter_traits.to_string(),
            self.ty_param_counter_functions.to_string(),
            self.ty_param_counter_methods.to_string(),
            self.ty_param_counter_ty_aliases.to_string(),
            self.ty_param_counter_other.to_string(),
            self.non_trivial_type_bounds,
            self.trivial_type_bounds,
            self.non_trivial_type_bounds_structs,
            self.trivial_type_bounds_structs,
            self.non_trivial_type_bounds_traits,
            self.trivial_type_bounds_traits,
            self.non_trivial_type_bounds_functions,
            self.trivial_type_bounds_functions,
            self.non_trivial_type_bounds_methods,
            self.trivial_type_bounds_methods,
            self.non_trivial_type_bounds_ty_aliases,
            self.trivial_type_bounds_ty_aliases,
            self.non_trivial_type_bounds_other,
            self.trivial_type_bounds_other,
        );
    }
}

fn count_trivial_type_params(g: &Generics, nt: &mut u64, t: &mut u64, snt: &mut u64, st: &mut u64) {
    for param in &g.params {
        match &param.kind {
            GenericParamKind::Type { default: d } => {
                if d.is_none() && param.bounds.is_empty() {
                    *t += 1;
                    *st += 1;
                } else {
                    *nt += 1;
                    *snt += 1;
                }
            }
            _ => {}
        }
    }
}

impl<'ast> Visitor<'ast> for AstWalker {
    fn visit_item(&mut self, item: &'ast Item) {
        match &item.kind {
            ItemKind::Fn(f) => {
                if is_generic(&f.generics) {
                    self.generic_fun += 1;
                    self.generic_functions += 1;
                    self.ty_param_counter.count(f.generics.params.len());
                    self.ty_param_counter_functions
                        .count(f.generics.params.len());
                    count_trivial_type_params(
                        &f.generics,
                        &mut self.non_trivial_type_bounds,
                        &mut self.trivial_type_bounds,
                        &mut self.non_trivial_type_bounds_functions,
                        &mut self.trivial_type_bounds_functions,
                    );
                } else {
                    self.non_generic_fun += 1
                }
            }
            ItemKind::Struct(_, g, _) => {
                if is_generic(g) {
                    self.generic_ty += 1;
                    self.generic_structs += 1;
                    self.ty_param_counter.count(g.params.len());
                    self.ty_param_counter_structs.count(g.params.len());
                    count_trivial_type_params(
                        &g,
                        &mut self.non_trivial_type_bounds,
                        &mut self.trivial_type_bounds,
                        &mut self.non_trivial_type_bounds_structs,
                        &mut self.trivial_type_bounds_structs,
                    );
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::Enum(_, g, _) => {
                if is_generic(g) {
                    self.generic_ty += 1;
                    self.generic_other += 1;
                    self.ty_param_counter.count(g.params.len());
                    self.ty_param_counter_other.count(g.params.len());
                    count_trivial_type_params(
                        &g,
                        &mut self.non_trivial_type_bounds,
                        &mut self.trivial_type_bounds,
                        &mut self.non_trivial_type_bounds_other,
                        &mut self.trivial_type_bounds_other,
                    );
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::TyAlias(tyalias) => {
                if is_generic(&tyalias.generics) {
                    self.generic_ty += 1;
                    self.generic_ty_aliases += 1;
                    self.ty_param_counter.count(tyalias.generics.params.len());
                    self.ty_param_counter_ty_aliases
                        .count(tyalias.generics.params.len());
                    count_trivial_type_params(
                        &tyalias.generics,
                        &mut self.non_trivial_type_bounds,
                        &mut self.trivial_type_bounds,
                        &mut self.non_trivial_type_bounds_ty_aliases,
                        &mut self.trivial_type_bounds_ty_aliases,
                    );
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::Union(_, g, _) => {
                if is_generic(g) {
                    self.generic_ty += 1;
                    self.generic_other += 1;
                    self.ty_param_counter.count(g.params.len());
                    self.ty_param_counter_other.count(g.params.len());
                    count_trivial_type_params(
                        &g,
                        &mut self.non_trivial_type_bounds,
                        &mut self.trivial_type_bounds,
                        &mut self.non_trivial_type_bounds_other,
                        &mut self.trivial_type_bounds_other,
                    );
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::Trait(t) => {
                if is_generic(&t.generics) {
                    self.generic_ty += 1;
                    self.generic_traits += 1;
                    self.ty_param_counter.count(t.generics.params.len());
                    self.ty_param_counter_traits.count(t.generics.params.len());
                    count_trivial_type_params(
                        &t.generics,
                        &mut self.non_trivial_type_bounds,
                        &mut self.trivial_type_bounds,
                        &mut self.non_trivial_type_bounds_traits,
                        &mut self.trivial_type_bounds_traits,
                    );
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::TraitAlias(trait_alias) => {
                if is_generic(&trait_alias.generics) {
                    self.generic_ty += 1;
                    self.generic_other += 1;
                    self.ty_param_counter
                        .count(trait_alias.generics.params.len());
                    self.ty_param_counter_other
                        .count(trait_alias.generics.params.len());
                    count_trivial_type_params(
                        &trait_alias.generics,
                        &mut self.non_trivial_type_bounds,
                        &mut self.trivial_type_bounds,
                        &mut self.non_trivial_type_bounds_other,
                        &mut self.trivial_type_bounds_other,
                    );
                } else {
                    self.non_generic_ty += 1
                }
            }
            ItemKind::Impl(impl_block) => {
                for impl_item in impl_block.items.iter() {
                    match &impl_item.kind {
                        AssocItemKind::Fn(f) => {
                            if is_generic(&f.generics) {
                                self.generic_fun += 1;
                                self.generic_methods += 1;
                                self.ty_param_counter.count(f.generics.params.len());
                                self.ty_param_counter_methods.count(f.generics.params.len());
                                count_trivial_type_params(
                                    &f.generics,
                                    &mut self.non_trivial_type_bounds,
                                    &mut self.trivial_type_bounds,
                                    &mut self.non_trivial_type_bounds_methods,
                                    &mut self.trivial_type_bounds_methods,
                                );
                            } else {
                                self.non_generic_fun += 1
                            }
                        }
                        AssocItemKind::Type(tyalias) => {
                            if is_generic(&tyalias.generics) {
                                self.generic_ty += 1;
                                self.generic_ty_aliases += 1;
                                self.ty_param_counter_ty_aliases
                                    .count(tyalias.generics.params.len());
                                count_trivial_type_params(
                                    &tyalias.generics,
                                    &mut self.non_trivial_type_bounds,
                                    &mut self.trivial_type_bounds,
                                    &mut self.non_trivial_type_bounds_ty_aliases,
                                    &mut self.trivial_type_bounds_ty_aliases,
                                );
                            } else {
                                self.non_generic_ty += 1
                            }
                        }
                        _ => {}
                    }
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

        let r = rustc_driver::catch_fatal_errors(|| run_compiler(&rustc_args, &mut cb));
        if r.is_err() {
            cb.walker.lock().unwrap().parse_errors += 1;
        }
    }

    cb.walker.lock().unwrap().print();
}
