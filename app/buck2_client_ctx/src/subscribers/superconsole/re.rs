/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use buck2_event_observer::re_state::ReState;
use buck2_event_observer::two_snapshots::TwoSnapshots;
use superconsole::Component;

use crate::subscribers::superconsole::SuperConsoleConfig;

/// Draw the test summary line above the `timed_list`
pub(crate) struct ReHeader<'a> {
    pub(crate) super_console_config: &'a SuperConsoleConfig,
    pub(crate) re_state: &'a ReState,
    pub(crate) two_snapshots: &'a TwoSnapshots,
}

impl Component for ReHeader<'_> {
    fn draw_unchecked(
        &self,
        _dimensions: superconsole::Dimensions,
        mode: superconsole::DrawMode,
    ) -> anyhow::Result<superconsole::Lines> {
        Ok(self.re_state.render(
            self.two_snapshots,
            self.super_console_config.enable_detailed_re,
            mode,
        )?)
    }
}
