# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

RuleRegistrationSpec = record(
    name = field(str),
    impl = field(typing.Callable),
    attrs = field(dict[str, Attr]),
    # TODO(nga): should be `transition | None`, but `transition` does not work as type.
    cfg = field(typing.Any | None, None),
    is_toolchain_rule = field(bool, False),
    doc = field(str, ""),
)
