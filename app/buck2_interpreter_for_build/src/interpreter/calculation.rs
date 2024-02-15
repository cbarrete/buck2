/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Interpreter related Dice calculations

use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use allocative::Allocative;
use async_trait::async_trait;
use buck2_common::package_listing::dice::HasPackageListingResolver;
use buck2_core::build_file_path::BuildFilePath;
use buck2_core::bzl::ImportPath;
use buck2_core::cells::build_file_cell::BuildFileCell;
use buck2_core::package::PackageLabel;
use buck2_events::dispatch::async_record_root_spans;
use buck2_events::span::SpanId;
use buck2_futures::cancellation::CancellationContext;
use buck2_interpreter::file_loader::LoadedModule;
use buck2_interpreter::file_loader::ModuleDeps;
use buck2_interpreter::file_type::StarlarkFileType;
use buck2_interpreter::load_module::InterpreterCalculationImpl;
use buck2_interpreter::load_module::INTERPRETER_CALCULATION_IMPL;
use buck2_interpreter::paths::module::StarlarkModulePath;
use buck2_interpreter::paths::package::PackageFilePath;
use buck2_interpreter::paths::path::StarlarkPath;
use buck2_interpreter::prelude_path::PreludePath;
use buck2_interpreter::starlark_profiler::StarlarkProfilerOrInstrumentation;
use buck2_node::metadata::key::MetadataKey;
use buck2_node::nodes::eval_result::EvaluationResult;
use buck2_node::nodes::frontend::TargetGraphCalculation;
use buck2_node::nodes::frontend::TargetGraphCalculationImpl;
use buck2_node::nodes::frontend::TARGET_GRAPH_CALCULATION_IMPL;
use buck2_node::package_values_calculation::PackageValuesCalculation;
use buck2_node::package_values_calculation::PACKAGE_VALUES_CALCULATION;
use derive_more::Display;
use dice::DiceComputations;
use dice::Key;
use dupe::Dupe;
use futures::future::BoxFuture;
use futures::FutureExt;
use smallvec::SmallVec;
use starlark::environment::Globals;
use starlark_map::small_map::SmallMap;

use crate::interpreter::dice_calculation_delegate::HasCalculationDelegate;
use crate::interpreter::global_interpreter_state::HasGlobalInterpreterState;

// Key for 'InterpreterCalculation::get_interpreter_results'
#[derive(Clone, Dupe, Display, Debug, Eq, Hash, PartialEq, Allocative)]
pub struct InterpreterResultsKey(pub PackageLabel);

struct TargetGraphCalculationInstance;

pub(crate) fn init_target_graph_calculation_impl() {
    TARGET_GRAPH_CALCULATION_IMPL.init(&TargetGraphCalculationInstance);
}

#[async_trait]
impl TargetGraphCalculationImpl for TargetGraphCalculationInstance {
    async fn get_interpreter_results_uncached(
        &self,
        ctx: &DiceComputations<'_>,
        package: PackageLabel,
    ) -> buck2_error::Result<Arc<EvaluationResult>> {
        let interpreter = ctx
            .get_interpreter_calculator(
                package.cell_name(),
                BuildFileCell::new(package.cell_name()),
            )
            .await?;
        interpreter
            .eval_build_file(
                package.dupe(),
                &mut StarlarkProfilerOrInstrumentation::disabled(),
            )
            .await
    }

    fn get_interpreter_results<'a>(
        &self,
        ctx: &'a DiceComputations,
        package: PackageLabel,
    ) -> BoxFuture<'a, anyhow::Result<Arc<EvaluationResult>>> {
        #[async_trait]
        impl Key for InterpreterResultsKey {
            type Value = buck2_error::Result<Arc<EvaluationResult>>;
            async fn compute(
                &self,
                ctx: &mut DiceComputations,
                _cancellation: &CancellationContext,
            ) -> Self::Value {
                let now = Instant::now();

                let (result, spans) =
                    async_record_root_spans(ctx.get_interpreter_results_uncached(self.0.dupe()))
                        .await;

                ctx.store_evaluation_data(IntepreterResultsKeyActivationData {
                    duration: now.elapsed(),
                    result: result.dupe(),
                    spans,
                })?;

                result
            }

            fn equality(_: &Self::Value, _: &Self::Value) -> bool {
                // TODO consider if we want to impl eq for this
                false
            }

            fn validity(x: &Self::Value) -> bool {
                x.is_ok()
            }
        }
        async move {
            ctx.bad_dice()
                .compute(&InterpreterResultsKey(package.dupe()))
                .await?
                .map_err(anyhow::Error::from)
        }
        .boxed()
    }
}

struct InterpreterCalculationInstance;
struct PackageValuesCalculationInstance;

pub(crate) fn init_interpreter_calculation_impl() {
    INTERPRETER_CALCULATION_IMPL.init(&InterpreterCalculationInstance);
    PACKAGE_VALUES_CALCULATION.init(&PackageValuesCalculationInstance);
}

#[async_trait]
impl InterpreterCalculationImpl for InterpreterCalculationInstance {
    async fn get_loaded_module(
        &self,
        ctx: &DiceComputations<'_>,
        path: StarlarkModulePath<'_>,
    ) -> anyhow::Result<LoadedModule> {
        ctx.get_interpreter_calculator(path.cell(), path.build_file_cell())
            .await?
            .eval_module(path)
            .await
    }

    async fn get_module_deps(
        &self,
        ctx: &DiceComputations<'_>,
        package: PackageLabel,
        build_file_cell: BuildFileCell,
    ) -> anyhow::Result<ModuleDeps> {
        let calc = ctx
            .get_interpreter_calculator(package.cell_name(), build_file_cell)
            .await?;

        let build_file_name = ctx
            .resolve_package_listing(package.dupe())
            .await?
            .buildfile()
            .to_owned();

        let (_module, module_deps) = calc
            .prepare_eval(StarlarkPath::BuildFile(&BuildFilePath::new(
                package.dupe(),
                build_file_name,
            )))
            .await?;

        Ok(module_deps)
    }

    async fn get_package_file_deps(
        &self,
        ctx: &DiceComputations<'_>,
        package: &PackageFilePath,
    ) -> anyhow::Result<Option<Vec<ImportPath>>> {
        // These aren't cached on the DICE graph, since in normal evaluation there aren't that many, and we can cache at a higher level.
        // Therefore we re-parse the file, if it exists.
        // Fortunately, there are only a small number (currently a few hundred)
        let interpreter = ctx
            .get_interpreter_calculator(package.cell(), package.build_file_cell())
            .await?;
        Ok(interpreter
            .prepare_package_file_eval(package)
            .await?
            .map(|x| x.1.get_loaded_modules().imports().cloned().collect()))
    }

    async fn global_env_for_file_type(
        &self,
        ctx: &DiceComputations<'_>,
        file_type: StarlarkFileType,
    ) -> anyhow::Result<Globals> {
        Ok(ctx
            .get_global_interpreter_state()
            .await?
            .globals_for_file_type(file_type)
            .dupe())
    }

    async fn prelude_import(
        &self,
        ctx: &DiceComputations<'_>,
    ) -> anyhow::Result<Option<PreludePath>> {
        Ok(ctx
            .get_global_interpreter_state()
            .await?
            .configuror
            .prelude_import()
            .cloned())
    }
}

#[async_trait]
impl PackageValuesCalculation for PackageValuesCalculationInstance {
    async fn package_values(
        &self,
        ctx: &DiceComputations<'_>,
        package: PackageLabel,
    ) -> anyhow::Result<SmallMap<MetadataKey, serde_json::Value>> {
        let listing = ctx.resolve_package_listing(package.dupe()).await?;
        let super_package = ctx
            .get_interpreter_calculator(
                package.cell_name(),
                BuildFileCell::new(package.cell_name()),
            )
            .await?
            .eval_package_file_for_build_file(package, &listing)
            .await?;
        super_package.package_values().package_values_json()
    }
}

pub struct IntepreterResultsKeyActivationData {
    pub duration: Duration,
    pub result: buck2_error::Result<Arc<EvaluationResult>>,
    pub spans: SmallVec<[SpanId; 1]>,
}
