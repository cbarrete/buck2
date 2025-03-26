# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

# pyre-strict


from buck2.tests.e2e_util.api.buck import Buck
from buck2.tests.e2e_util.buck_workspace import buck_test


def assert_file_content_matches(file_path: str, exptected_content: str) -> None:
    with open(file_path, "r") as f:
        content = f.read()
        assert content == exptected_content


@buck_test()
async def test_streaming_output_ensured_artifact(buck: Buck) -> None:
    result = await buck.bxl(
        "//streaming.bxl:streaming_output_ensured_artifact",
    )

    lines = result.stdout.splitlines()

    streaming_output_artifact_idx = -1
    output_file_path = ""
    line_before_print_idx = -1

    for idx, line in enumerate(lines):
        # output by `ctx.output.print(ensured_output)`
        if "output.txt" in line and streaming_output_artifact_idx == -1:
            output_file_path = line
            streaming_output_artifact_idx = idx
        # output by `ctx.output.print("Line before streaming print")`
        if "Line before streaming print" in line:
            line_before_print_idx = idx

    assert (
        streaming_output_artifact_idx != -1
    ), "Cound not find the streaming print of ensured artifact"
    assert line_before_print_idx != -1, "Cound not find the normal ctx.output.print"

    assert (
        streaming_output_artifact_idx < line_before_print_idx
    ), "The streaming print is not before the normal print"

    assert_file_content_matches(output_file_path, "hello world!")
