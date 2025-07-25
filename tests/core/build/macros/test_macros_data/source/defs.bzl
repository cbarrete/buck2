# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is dual-licensed under either the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree or the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree. You may select, at your option, one of the
# above-listed licenses.

def _echo(ctx):
    return [DefaultInfo(), RunInfo(args = cmd_args("echo", ctx.attrs.arg))]

echo_rule = rule(
    impl = _echo,
    attrs = {
        "arg": attrs.arg(),
    },
)

def _cat(ctx):
    return [DefaultInfo(), RunInfo(args = cmd_args("cat", ctx.attrs.arg))]

cat_rule = rule(
    impl = _cat,
    attrs = {
        "arg": attrs.arg(),
    },
)
