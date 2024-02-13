/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Core data objects used in the protocol

mod convert;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::time::Duration;
use std::time::SystemTime;

use buck2_core::cells::name::CellName;
use buck2_core::fs::paths::abs_norm_path::AbsNormPathBuf;
use buck2_core::fs::paths::forward_rel_path::ForwardRelativePathBuf;
pub use buck2_test_proto::ExecutionDetails;
use derive_more::From;
use dupe::Dupe;
use host_sharing::HostSharingRequirements;
use sorted_vector_map::SortedVectorMap;

/// A handle generated by the TestOrchestrator. It can be used by the TestExecutor to access this
/// target.
#[derive(Debug, Copy, Clone, Dupe, From, Hash, Eq, PartialEq)]
pub struct ConfiguredTargetHandle(u64);

/// The Target of a test rule
#[derive(Debug, Clone, PartialEq)]
pub struct ConfiguredTarget {
    pub handle: ConfiguredTargetHandle,
    /// Structured data
    pub cell: String,
    pub package: String,
    pub target: String,
    pub configuration: String,
    pub package_project_relative_path: ForwardRelativePathBuf,
}

/// Metadata about the execution to display
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayMetadata {
    // Listing the test binary to discover tests. The String is the name of the suite at the binary
    Listing(String),
    // the name of the test(s) that we are running for the suite of a target
    Testing {
        suite: String,
        testcases: Vec<String>,
    },
}

#[derive(Clone, PartialEq)]
pub enum ExecutionStream {
    Inline(Vec<u8>),
}

impl Debug for ExecutionStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inline(d) => {
                write!(f, "{}", String::from_utf8_lossy(d))
            }
        }
    }
}

#[derive(Clone, Debug, Dupe, PartialEq)]
pub enum ExecutionStatus {
    Finished { exitcode: i32 },
    TimedOut { duration: Duration },
}

/// The result of running a test
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TestResult {
    // the target the test came from
    pub target: ConfiguredTargetHandle,
    // the name of the test
    pub name: String,
    // the status of running the test
    pub status: TestStatus,
    // additional optional messages
    pub msg: Option<String>,
    // the duration of the test run
    // TODO(skcd) should this be optional? why doesn't everything have duration
    pub duration: Option<Duration>,
    // the output of the test execution (combining stdout and stderr)
    pub details: String,
}

/// different possible test results
#[derive(PartialEq, Eq, Debug, Clone, Dupe)]
#[allow(non_camel_case_types)]
pub enum TestStatus {
    PASS,
    FAIL,
    SKIP,
    OMITTED,
    FATAL,
    TIMEOUT,
    // There is something called unknown, adding it here for now,
    // we can change it later on.
    UNKNOWN,
    // We also have re-runs
    RERUN,
    LISTING_SUCCESS,
    LISTING_FAILED,
}

/// The set of information about a test rule that is passed to the test executor
#[derive(Clone, Debug, PartialEq)]
pub struct ExternalRunnerSpec {
    /// Target the spec belongs to
    pub target: ConfiguredTarget,
    /// Type of test spec
    pub test_type: String,
    /// Base command used for further processing. A mix of verbatim arguments and
    /// opaque handles for more complex arguments.
    pub command: Vec<ExternalRunnerSpecValue>,
    /// Environment variables a specified by the rule. A mapping from keys to
    /// verbatim values or opaque handles for more complex values.
    pub env: HashMap<String, ExternalRunnerSpecValue>,
    /// Labels defined on the rule.
    pub labels: Vec<String>,
    /// Contacts defined on the rule.
    pub contacts: Vec<String>,
    /// Oncall for the test
    pub oncall: Option<String>,
    /// Cell of current working directory for test command.
    pub working_dir_cell: CellName,
}

/// Command line argument or environment variable value
///
/// It is either a verbatim string, or a reference to a more complex value that's opaque to the
/// test run coordinator.
#[derive(Clone, Debug, PartialEq)]
pub enum ExternalRunnerSpecValue {
    Verbatim(String),
    ArgHandle(ArgHandle),
    EnvHandle(EnvHandle),
}

/// Handle referring to a complex argument defined on the test rule
#[derive(Clone, Debug, Dupe, PartialEq, From)]
pub struct ArgHandle(pub usize);

impl TryFrom<i64> for ArgHandle {
    type Error = anyhow::Error;

    fn try_from(i: i64) -> Result<Self, Self::Error> {
        Ok(ArgHandle(i.try_into()?))
    }
}

/// Handle referring to a complex environment value defined on the test rule
#[derive(Clone, Debug, PartialEq, From)]
pub struct EnvHandle(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct ArgValue {
    pub content: ArgValueContent,
    pub format: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArgValueContent {
    ExternalRunnerSpecValue(ExternalRunnerSpecValue),
    DeclaredOutput(DeclaredOutput),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DeclaredOutput {
    pub name: ForwardRelativePathBuf,
}

impl DeclaredOutput {
    pub fn unchecked_new(name: String) -> Self {
        Self {
            name: ForwardRelativePathBuf::unchecked_new(name),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExecutorConfigOverride {
    pub name: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LocalResourceType {
    pub name: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RequiredLocalResources {
    pub resources: Vec<LocalResourceType>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecuteRequest2 {
    pub test_executable: TestExecutable,
    pub timeout: Duration,
    pub host_sharing_requirements: HostSharingRequirements,
    pub executor_override: Option<ExecutorConfigOverride>,
    pub required_local_resources: RequiredLocalResources,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    LocalPath(AbsNormPathBuf),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionResult2 {
    pub status: ExecutionStatus,
    pub stdout: ExecutionStream,
    pub stderr: ExecutionStream,
    pub outputs: HashMap<DeclaredOutput, Output>,
    pub start_time: SystemTime,
    pub execution_time: Duration,
    /// We don't try to convert this field, mostly because it shares with buck2.data, and that
    /// seems to have very little value. We just validate it's sent.
    pub execution_details: ExecutionDetails,
}

#[allow(clippy::large_enum_variant)]
pub enum ExecuteResponse {
    /// A result is available.
    Result(ExecutionResult2),

    /// The test run is being cancelled.
    Cancelled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestExecutable {
    pub display: DisplayMetadata,
    pub target: ConfiguredTargetHandle,
    pub cmd: Vec<ArgValue>,
    pub env: SortedVectorMap<String, ArgValue>,
    pub pre_create_dirs: Vec<DeclaredOutput>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrepareForLocalExecutionResult {
    pub cmd: Vec<String>,
    pub env: SortedVectorMap<String, String>,
    pub cwd: AbsNormPathBuf,
}

pub mod testing {
    use crate::data::ConfiguredTargetHandle;

    pub trait ConfiguredTargetHandleExt {
        fn testing_new(id: u64) -> Self;
    }

    impl ConfiguredTargetHandleExt for ConfiguredTargetHandle {
        fn testing_new(id: u64) -> Self {
            Self(id)
        }
    }
}
