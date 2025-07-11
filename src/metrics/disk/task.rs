use crate::metrics::disk::model::DiskMetrics;
use crate::task::core::SshTask;
use tokio::time::Duration;

pub struct DiskTask;

impl SshTask for DiskTask {
    type Output = DiskMetrics;

    fn name(&self) -> &'static str {
        "disk_metrics"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn command(&self) -> String {
        "echo 1000000; echo 500000".into()
    }

    fn parse(&self, raw: &str) -> Result<Self::Output, String> {
        let mut lines = raw.lines();
        let total = lines
            .next()
            .ok_or_else(|| "missing total".to_string())?
            .trim()
            .parse::<u64>()
            .map_err(|e| format!("parse total: {e}"))?;
        let used = lines
            .next()
            .ok_or_else(|| "missing used".to_string())?
            .trim()
            .parse::<u64>()
            .map_err(|e| format!("parse used: {e}"))?;
        Ok(DiskMetrics { total, used })
    }
}
