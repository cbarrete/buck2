/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use buck2_cli_proto::profile_request::ProfileOpts;
use buck2_cli_proto::ProfileRequest;
use buck2_cli_proto::ProfileResponse;
use buck2_common::dice::cells::HasCellResolver;
use buck2_core::fs::paths::abs_path::AbsPath;
use buck2_error::buck2_error;
use buck2_error::internal_error;
use buck2_error::BuckErrorContext;
use buck2_interpreter::starlark_profiler::config::StarlarkProfilerConfiguration;
use buck2_interpreter::starlark_profiler::mode::StarlarkProfileMode;
use buck2_profile::get_profile_response;
use buck2_profile::starlark_profiler_configuration_from_request;
use buck2_server_ctx::ctx::ServerCommandContextTrait;
use buck2_server_ctx::global_cfg_options::global_cfg_options_from_client_context;
use buck2_server_ctx::partial_result_dispatcher::NoPartialResult;
use buck2_server_ctx::partial_result_dispatcher::PartialResultDispatcher;
use buck2_server_ctx::template::run_server_command;
use buck2_server_ctx::template::ServerCommandTemplate;
use dice::DiceTransaction;
use futures::FutureExt;

use crate::bxl::eval::eval;
use crate::bxl::eval::BxlResolvedCliArgs;
use crate::bxl::key::BxlKey;
use crate::command::get_bxl_cli_args;
use crate::command::parse_bxl_label_from_cli;

pub(crate) async fn bxl_profile_command(
    ctx: &dyn ServerCommandContextTrait,
    partial_result_dispatcher: PartialResultDispatcher<NoPartialResult>,
    req: ProfileRequest,
) -> buck2_error::Result<ProfileResponse> {
    run_server_command(
        BxlProfileServerCommand { req },
        ctx,
        partial_result_dispatcher,
    )
    .await
}

struct BxlProfileServerCommand {
    req: ProfileRequest,
}

#[async_trait]
impl ServerCommandTemplate for BxlProfileServerCommand {
    type StartEvent = buck2_data::ProfileCommandStart;
    type EndEvent = buck2_data::ProfileCommandEnd;
    type Response = buck2_cli_proto::ProfileResponse;
    type PartialResult = NoPartialResult;

    async fn command(
        &self,
        server_ctx: &dyn ServerCommandContextTrait,
        _partial_result_dispatcher: PartialResultDispatcher<Self::PartialResult>,
        mut ctx: DiceTransaction,
    ) -> buck2_error::Result<Self::Response> {
        match self
            .req
            .profile_opts
            .as_ref()
            .expect("BXL profile opts not populated")
        {
            ProfileOpts::BxlProfile(opts) => {
                let output = AbsPath::new(Path::new(&self.req.destination_path))?;

                let profile_mode = starlark_profiler_configuration_from_request(
                    &self.req,
                    server_ctx.project_root(),
                )?;

                let profile_data = match profile_mode {
                    StarlarkProfilerConfiguration::ProfileBxl(profile_mode) => {
                        let cwd = server_ctx.working_dir();

                        let cell_resolver = ctx.get_cell_resolver().await?;
                        let cell_alias_resolver = ctx.get_cell_alias_resolver_for_dir(cwd).await?;
                        let bxl_label = parse_bxl_label_from_cli(
                            cwd,
                            &opts.bxl_label,
                            &cell_resolver,
                            &cell_alias_resolver,
                        )?;

                        let bxl_args = match get_bxl_cli_args(
                            cwd,
                            &mut ctx,
                            &bxl_label,
                            &opts.bxl_args,
                            &cell_resolver,
                        )
                        .await?
                        {
                            BxlResolvedCliArgs::Resolved(bxl_args) => Arc::new(bxl_args),
                            _ => {
                                return Err(buck2_error!(
                                    buck2_error::ErrorTag::Input,
                                    "Help docs were displayed. No profiler data available"
                                ));
                            }
                        };

                        let global_cfg_options = global_cfg_options_from_client_context(
                            opts.target_cfg
                                .as_ref()
                                .internal_error("target_cfg must be set")?,
                            server_ctx,
                            &mut ctx,
                        )
                        .await?;

                        let bxl_key = BxlKey::new(
                            bxl_label.clone(),
                            bxl_args,
                            /* force print stacktrace */ false,
                            global_cfg_options,
                        );

                        server_ctx
                            .cancellation_context()
                            .with_structured_cancellation(|observer| {
                                async move {
                                    buck2_error::Ok(
                                        eval(
                                            &mut ctx,
                                            bxl_key,
                                            StarlarkProfileMode::Profile(profile_mode),
                                            observer,
                                        )
                                        .await?
                                        .1
                                        .map(Arc::new)
                                        .expect("No bxl profile data found"),
                                    )
                                }
                                .boxed()
                            })
                            .await?
                    }
                    _ => {
                        return Err(internal_error!("Incorrect profile mode"));
                    }
                };

                Ok(get_profile_response(profile_data, output)?)
            }
            _ => {
                return Err(buck2_error!(
                    buck2_error::ErrorTag::Input,
                    "Expected BXL profile opts, not target profile opts"
                ));
            }
        }
    }

    fn is_success(&self, _response: &Self::Response) -> bool {
        // No response if we failed.
        true
    }
}
