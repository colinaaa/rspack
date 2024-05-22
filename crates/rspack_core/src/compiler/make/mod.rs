mod cutout;
mod file_counter;
pub mod repair;

use std::path::PathBuf;

use rspack_error::{Diagnostic, Result};
use rspack_identifier::IdentifierSet;
use rustc_hash::FxHashSet as HashSet;

use self::{cutout::Cutout, file_counter::FileCounter, repair::repair};
use crate::{
  BuildDependency, Compilation, DependencyId, ModuleGraph, ModuleGraphPartial, ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct MakeArtifact {
  // should be reset when make
  pub make_failed_dependencies: HashSet<BuildDependency>,
  pub make_failed_module: HashSet<ModuleIdentifier>,
  pub diagnostics: Vec<Diagnostic>,
  pub has_module_graph_change: bool,

  pub module_graph_partial: ModuleGraphPartial,
  entry_dependencies: HashSet<DependencyId>,
  pub entry_module_identifiers: IdentifierSet,
  pub file_dependencies: FileCounter,
  pub context_dependencies: FileCounter,
  pub missing_dependencies: FileCounter,
  pub build_dependencies: FileCounter,
}

impl MakeArtifact {
  fn get_module_graph(&self) -> ModuleGraph {
    ModuleGraph::new(vec![&self.module_graph_partial], None)
  }
  fn get_module_graph_mut(&mut self) -> ModuleGraph {
    ModuleGraph::new(vec![], Some(&mut self.module_graph_partial))
  }
  // TODO remove it
  pub fn get_module_graph_partial(&self) -> &ModuleGraphPartial {
    &self.module_graph_partial
  }
  // TODO remove it
  pub fn get_module_graph_partial_mut(&mut self) -> &mut ModuleGraphPartial {
    &mut self.module_graph_partial
  }

  fn revoke_modules(&mut self, ids: HashSet<ModuleIdentifier>) -> Vec<BuildDependency> {
    let mut module_graph = ModuleGraph::new(vec![], Some(&mut self.module_graph_partial));
    let mut res = vec![];
    for module_identifier in &ids {
      let module = module_graph
        .module_by_identifier(module_identifier)
        .expect("should have module");
      if let Some(build_info) = module.build_info() {
        self
          .file_dependencies
          .remove_batch_file(&build_info.file_dependencies);
        self
          .context_dependencies
          .remove_batch_file(&build_info.context_dependencies);
        self
          .missing_dependencies
          .remove_batch_file(&build_info.missing_dependencies);
        self
          .build_dependencies
          .remove_batch_file(&build_info.build_dependencies);
      }
      res.extend(module_graph.revoke_module(module_identifier));
    }
    res
  }
}

#[derive(Debug, Clone)]
pub enum MakeParam {
  Entry(HashSet<DependencyId>),
  ModifiedFiles(HashSet<PathBuf>),
  RemovedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
  ForceBuildModules(HashSet<ModuleIdentifier>),
}

pub fn make_module_graph(
  compilation: &mut Compilation,
  mut artifact: MakeArtifact,
) -> Result<MakeArtifact> {
  let mut params = Vec::with_capacity(5);

  if !compilation.entries.is_empty() {
    params.push(MakeParam::Entry(
      compilation
        .entries
        .values()
        .flat_map(|item| item.all_dependencies())
        .chain(compilation.global_entry.all_dependencies())
        .cloned()
        .collect(),
    ));
  }
  // no modified files but rebuild means force build
  // some module which cacheable is false will need to be rebuilt even if modified files is empty
  params.push(MakeParam::ModifiedFiles(compilation.modified_files.clone()));
  if !compilation.removed_files.is_empty() {
    params.push(MakeParam::RemovedFiles(compilation.removed_files.clone()));
  }
  if !artifact.make_failed_module.is_empty() {
    let make_failed_module = std::mem::take(&mut artifact.make_failed_module);
    params.push(MakeParam::ForceBuildModules(make_failed_module));
  }
  if !artifact.make_failed_dependencies.is_empty() {
    let make_failed_dependencies = std::mem::take(&mut artifact.make_failed_dependencies);
    params.push(MakeParam::ForceBuildDeps(make_failed_dependencies));
  }

  // reset diagnostics
  artifact.diagnostics = Default::default();
  artifact.has_module_graph_change = false;

  artifact = update_module_graph_with_artifact(compilation, artifact, params)?;

  compilation.push_batch_diagnostic(std::mem::take(&mut artifact.diagnostics));
  Ok(artifact)
}

pub async fn update_module_graph(
  compilation: &mut Compilation,
  params: Vec<MakeParam>,
) -> Result<()> {
  let mut artifact = MakeArtifact::default();
  compilation.swap_make_artifact(&mut artifact);

  artifact = update_module_graph_with_artifact(compilation, artifact, params)?;

  compilation.push_batch_diagnostic(std::mem::take(&mut artifact.diagnostics));
  compilation.swap_make_artifact(&mut artifact);
  Ok(())
}

pub fn update_module_graph_with_artifact(
  compilation: &Compilation,
  mut artifact: MakeArtifact,
  params: Vec<MakeParam>,
) -> Result<MakeArtifact> {
  let mut cutout = Cutout::default();
  let build_dependencies = cutout.cutout_artifact(&mut artifact, params);
  artifact = repair(compilation, artifact, build_dependencies)?;
  cutout.fix_artifact(&mut artifact);
  Ok(artifact)
}
