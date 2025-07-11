use crate::metrics::mem::model::MemMetrics;
use crate::task::core::SshTask;
use tokio::time::Duration;

pub struct MemTask;

impl SshTask for MemTask {
    type Output = MemMetrics;

    fn name(&self) -> &'static str {
        "mem_metrics"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn command(&self) -> String {
        "echo 8192; echo 4096".into()
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
        Ok(MemMetrics { total, used })
    }
}
