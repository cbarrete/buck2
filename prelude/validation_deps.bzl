# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is dual-licensed under either the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree or the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree. You may select, at your option, one of the
# above-listed licenses.

VALIDATION_DEPS_ATTR_NAME = "validation_deps"
VALIDATION_DEPS_ATTR_TYPE = attrs.set(attrs.dep(), sorted = True, default = [])

def get_validation_deps_outputs(ctx: AnalysisContext) -> list[Artifact]:
    artifacts = []
    if hasattr(ctx.attrs, VALIDATION_DEPS_ATTR_NAME):
        validation_deps = getattr(ctx.attrs, VALIDATION_DEPS_ATTR_NAME)
        for dep in validation_deps:
            default_info = dep[DefaultInfo]
            artifacts += default_info.default_outputs
    return artifacts
