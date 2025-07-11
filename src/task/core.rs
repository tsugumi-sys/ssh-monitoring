use tokio::time::Duration;

/// Trait representing a unit of work executed over SSH.
///
/// An `SshTask` knows the command to execute on the remote host and
/// how to parse the produced output into a structured value.
pub trait SshTask: Send + Sync {
    type Output: Send + Sync;

    /// Name of this task for logging/debug purposes.
    fn name(&self) -> &'static str;

    /// Interval at which this task should run.
    fn interval(&self) -> Duration;

    /// Command string that will be executed on the remote host.
    fn command(&self) -> String;

    /// Parse the raw command output into the desired output type.
    fn parse(&self, raw: &str) -> Result<Self::Output, String>;
}
