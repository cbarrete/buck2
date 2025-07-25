/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use buck2_core::execution_types::executor_config::RePlatformFields;
use remote_execution as RE;

pub trait RePlatformFieldsToRePlatform {
    fn to_re_platform(&self) -> RE::Platform;
}

impl RePlatformFieldsToRePlatform for RePlatformFields {
    fn to_re_platform(&self) -> RE::Platform {
        RE::Platform {
            properties: self
                .properties
                .iter()
                .map(|(k, v)| RE::Property {
                    name: k.clone(),
                    value: v.clone(),
                })
                .collect(),
        }
    }
}
