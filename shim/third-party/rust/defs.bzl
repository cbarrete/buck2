# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

def rust_library_from_crates(name):
    # @lint-ignore BUCKLINT: avoid "Direct usage of native rules is not allowed."
    native.export_file(name = name, src = "BUCK", visibility = ["PUBLIC"])

def rust_binary_from_crates(name):
    # @lint-ignore BUCKLINT: avoid "Direct usage of native rules is not allowed."
    native.genrule(name = name, cmd = "exit 1", executable = True, out = "out", visibility = ["PUBLIC"])
