/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::cmp::max;
use std::fmt::Display;
use std::fmt::Formatter;
use std::time::SystemTime;

use buck2_client_ctx::client_ctx::ClientCommandContext;
use buck2_client_ctx::exit_result::ExitResult;
use buck2_client_ctx::subscribers::recorder::process_memory;
use buck2_data::ActionExecutionKind;
use buck2_event_log::stream_value::StreamValue;
use buck2_event_observer::fmt_duration;
use buck2_event_observer::humanized::HumanizedBytes;
use buck2_event_observer::humanized::HumanizedBytesPerSecond;
use buck2_util::network_speed_average::NetworkSpeedAverage;
use tokio_stream::StreamExt;

use crate::commands::log::options::EventLogOptions;

#[derive(Default)]
struct Stats {
    // TODO(yurysamkevich): add number of file changes since last build once availbale in log
    total_bytes_uploaded: u64,
    total_files_materialized: u64,
    total_bytes_materialized: u64,
    total_local_actions: u64,
    // TODO(yurysamkevich): split by RE platform - mac/windows/linux once available in log
    total_remote_actions: u64,
    total_other_actions: u64,
    total_targets_analysed: u64,
    peak_process_memory_bytes: Option<u64>,
    re_avg_download_speed: NetworkSpeedAverage,
    re_avg_upload_speed: NetworkSpeedAverage,
    duration: Option<prost_types::Duration>,
    peak_used_disk_space_bytes: Option<u64>,
    total_disk_space_bytes: Option<u64>,
    system_total_memory_bytes: Option<u64>,
}

impl Stats {
    fn update_with_event(&mut self, event: &buck2_data::BuckEvent) {
        match &event.data {
            Some(buck2_data::buck_event::Data::SpanEnd(end)) => match end.data.as_ref() {
                Some(buck2_data::span_end_event::Data::ReUpload(ref data)) => {
                    self.total_bytes_uploaded += data.bytes_uploaded.unwrap_or_default();
                }
                Some(buck2_data::span_end_event::Data::Materialization(ref data)) => {
                    self.total_files_materialized += data.file_count;
                    self.total_bytes_materialized += data.total_bytes;
                }
                Some(buck2_data::span_end_event::Data::ActionExecution(ref data)) => {
                    match ActionExecutionKind::from_i32(data.execution_kind) {
                        Some(ActionExecutionKind::Local) => self.total_local_actions += 1,
                        Some(ActionExecutionKind::Remote) => self.total_remote_actions += 1,
                        Some(ActionExecutionKind::ActionCache) => self.total_remote_actions += 1,
                        _ => self.total_other_actions += 1,
                    }
                }
                Some(buck2_data::span_end_event::Data::Analysis(_)) => {
                    self.total_targets_analysed += 1;
                }
                Some(buck2_data::span_end_event::Data::Command(_command)) => {
                    self.duration = end.duration.clone();
                }
                _ => {}
            },
            Some(buck2_data::buck_event::Data::Instant(instant_event)) => {
                match instant_event.data.as_ref() {
                    Some(buck2_data::instant_event::Data::Snapshot(snapshot)) => {
                        self.peak_process_memory_bytes =
                            max(self.peak_process_memory_bytes, process_memory(snapshot));
                        self.peak_used_disk_space_bytes = max(
                            self.peak_used_disk_space_bytes,
                            snapshot.used_disk_space_bytes,
                        );

                        if let Some(ts) = get_event_timestamp(event) {
                            self.re_avg_download_speed
                                .update(ts, snapshot.re_download_bytes);
                            self.re_avg_upload_speed
                                .update(ts, snapshot.re_upload_bytes);
                        }
                    }
                    Some(buck2_data::instant_event::Data::SystemInfo(system_info)) => {
                        self.total_disk_space_bytes = system_info.total_disk_space_bytes;
                        self.system_total_memory_bytes = system_info.system_total_memory_bytes;
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }
}

fn get_event_timestamp(event: &buck2_data::BuckEvent) -> Option<SystemTime> {
    SystemTime::try_from(event.timestamp.clone()?).ok()
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "total files materialized: {}",
            self.total_files_materialized
        )?;
        writeln!(
            f,
            "total bytes materialized: {}",
            self.total_bytes_materialized
        )?;
        writeln!(f, "total bytes uploaded: {}", self.total_bytes_uploaded)?;
        writeln!(f, "local actions: {}", self.total_local_actions)?;
        writeln!(f, "remote actions: {}", self.total_remote_actions)?;
        writeln!(f, "other actions: {}", self.total_other_actions)?;
        writeln!(f, "targets analysed: {}", self.total_targets_analysed)?;
        if let (Some(peak_process_memory_bytes), Some(system_total_memory_bytes)) = (
            self.peak_process_memory_bytes,
            self.system_total_memory_bytes,
        ) {
            writeln!(
                f,
                "peak process memory: {} out of {}",
                HumanizedBytes::fixed_width(peak_process_memory_bytes),
                HumanizedBytes::fixed_width(system_total_memory_bytes)
            )?;
        }
        if let (Some(peak_used_disk_space_bytes), Some(total_disk_space_bytes)) =
            (self.peak_used_disk_space_bytes, self.total_disk_space_bytes)
        {
            writeln!(
                f,
                "peak used disk space: {} out of {}",
                HumanizedBytes::fixed_width(peak_used_disk_space_bytes),
                HumanizedBytes::fixed_width(total_disk_space_bytes)
            )?;
        }
        if let Some(re_avg_download_speed) = self.re_avg_download_speed.avg_per_second() {
            writeln!(
                f,
                "average download speed: {}",
                HumanizedBytesPerSecond::fixed_width(re_avg_download_speed)
            )?;
        }
        if let Some(re_avg_upload_speed) = self.re_avg_upload_speed.avg_per_second() {
            writeln!(
                f,
                "average upload speed: {}",
                HumanizedBytesPerSecond::fixed_width(re_avg_upload_speed)
            )?;
        }
        if let Some(duration) = &self.duration {
            let duration = std::time::Duration::new(duration.seconds as u64, duration.nanos as u32);
            writeln!(f, "duration: {}", fmt_duration::fmt_duration(duration, 1.0))
        } else {
            // TODO(ezgi): when there is no CommandEnd, take the timestamp from the last event and calculate the duration
            Ok(())
        }
    }
}

/// Outputs high level statistics about the build
#[derive(Debug, clap::Parser)]
pub struct SummaryCommand {
    #[clap(flatten)]
    event_log: EventLogOptions,
}

impl SummaryCommand {
    pub fn exec(self, _matches: &clap::ArgMatches, ctx: ClientCommandContext<'_>) -> ExitResult {
        ctx.with_runtime(|ctx| async move {
            let log_path = self.event_log.get(&ctx).await?;

            let (invocation, mut events) = log_path.unpack_stream().await?;

            buck2_client_ctx::eprintln!(
                "Showing summary from: {}",
                invocation.display_command_line()
            )?;
            buck2_client_ctx::eprintln!("build ID: {}", invocation.trace_id)?;

            let mut stats = Stats::default();

            while let Some(event) = events.try_next().await? {
                match event {
                    StreamValue::Event(event) => stats.update_with_event(&event),
                    StreamValue::Result(..) | StreamValue::PartialResult(..) => {}
                }
            }
            buck2_client_ctx::eprintln!("{}", stats)?;
            anyhow::Ok(())
        })?;

        ExitResult::success()
    }
}
