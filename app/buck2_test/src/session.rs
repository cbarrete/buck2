/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use core::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use allocative::Allocative;
use anyhow::Context as _;
use buck2_core::fs::paths::forward_rel_path::ForwardRelativePathBuf;
use buck2_core::provider::label::ConfiguredProvidersLabel;
use buck2_test_api::data::ConfiguredTargetHandle;
use chrono::Local;
use dashmap::DashMap;
use dupe::Dupe;

#[derive(Debug, Clone, Copy, Dupe, Default, Allocative, PartialEq, Hash, Eq)]
pub struct TestSessionOptions {
    /// Whether this session should allow things to run on RE.
    pub allow_re: bool,
    pub force_use_project_relative_paths: bool,
    pub force_run_from_project_root: bool,
}

impl fmt::Display for TestSessionOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "allow_re = {}, force_use_project_relative_paths = {}, force_run_from_project_root = {}",
            self.allow_re, self.force_use_project_relative_paths, self.force_run_from_project_root
        )
    }
}

/// The state of a buck2 test command.
pub struct TestSession {
    /// The next ConfiguredTargetHandle that will be assigned.
    next_id: AtomicU64,
    /// A mapping of ConfiguredTargetHandle (which Tpx can use with) to the underlying provider in
    /// Buck2.
    labels: DashMap<ConfiguredTargetHandle, ConfiguredProvidersLabel>,
    /// The prefix to assign to all paths for this test session. This isn't used to provide any
    /// uniqueness (at least not at this time), but it's helpful to group outputs in a way that
    /// more-or-less matches a given test session.
    prefix: Arc<ForwardRelativePathBuf>,
    /// Options overriding the behavior of tests executed in this session. This is primarily
    /// intended for unstable or debugging features.
    options: TestSessionOptions,
}

impl TestSession {
    pub fn new(options: TestSessionOptions) -> Self {
        // NOTE: This is the format that Tpx has historically used. We don't really *have* to use
        // this considering we don't even put it in the same place (we do it in ./buck-out/v2/tmp,
        // but Tpx put it in /tmp), but it's a reasonable one.
        let now = Local::now();
        let now = now.format("%Y%m%d-%H%M%S").to_string();
        let prefix = ForwardRelativePathBuf::unchecked_new(now);

        Self {
            next_id: AtomicU64::new(0),
            labels: DashMap::new(),
            prefix: Arc::new(prefix),
            options,
        }
    }

    pub fn options(&self) -> TestSessionOptions {
        self.options
    }

    pub fn prefix(&self) -> Arc<ForwardRelativePathBuf> {
        self.prefix.dupe()
    }

    /// Insert a new provider and retrieve the matching handle.
    pub fn register(&self, label: ConfiguredProvidersLabel) -> ConfiguredTargetHandle {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed).into();
        let inserted = self.labels.insert(id, label).is_none();
        assert!(inserted);
        id
    }

    /// Retrieve the provider for a given handle.
    pub fn get(&self, id: ConfiguredTargetHandle) -> anyhow::Result<ConfiguredProvidersLabel> {
        let res = self
            .labels
            .get(&id)
            .with_context(|| format!("Invalid id provided to TestSession: {id:?}"))?;

        Ok(res.clone())
    }
}
