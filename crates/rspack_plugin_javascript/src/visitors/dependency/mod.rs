mod code_generation;
mod common_js_scanner;
mod hmr_scanner;
mod import_meta_scanner;
mod node_stuff_scanner;
mod scanner;
mod util;
pub use code_generation::*;
use rspack_core::{
  ast::javascript::Program, CompilerOptions, Dependency, ModuleDependency, ResourceData,
};
use swc_core::common::{Mark, SyntaxContext};
pub use util::*;

use self::{
  common_js_scanner::CommonJsScanner, hmr_scanner::HmrDependencyScanner,
  node_stuff_scanner::NodeStuffScanner, scanner::DependencyScanner,
};

pub type ScanDependenciesResult = (Vec<Box<dyn ModuleDependency>>, Vec<Box<dyn Dependency>>);

pub fn scan_dependencies(
  program: &Program,
  unresolved_mark: Mark,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
) -> ScanDependenciesResult {
  let mut dependencies: Vec<Box<dyn ModuleDependency>> = vec![];
  let mut presentational_dependencies: Vec<Box<dyn Dependency>> = vec![];
  let unresolved_ctxt = SyntaxContext::empty().apply_mark(unresolved_mark);
  program.visit_with_path(
    &mut DependencyScanner::new(
      &unresolved_ctxt,
      resource_data,
      compiler_options,
      &mut dependencies,
      &mut presentational_dependencies,
    ),
    &mut Default::default(),
  );
  program.visit_with_path(
    &mut HmrDependencyScanner::new(&mut dependencies),
    &mut Default::default(),
  );
  program.visit_with(&mut CommonJsScanner::new(&mut presentational_dependencies));
  if let Some(node_option) = &compiler_options.node {
    program.visit_with_path(
      &mut NodeStuffScanner::new(
        &mut presentational_dependencies,
        &unresolved_ctxt,
        compiler_options,
        node_option,
        resource_data,
      ),
      &mut Default::default(),
    );
  }
  (dependencies, presentational_dependencies)
}
