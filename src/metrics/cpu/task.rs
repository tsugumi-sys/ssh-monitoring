use crate::metrics::cpu::model::CpuMetrics;
use crate::task::core::SshTask;
use tokio::time::Duration;

pub struct CpuTask;

impl SshTask for CpuTask {
    type Output = CpuMetrics;

    fn name(&self) -> &'static str {
        "cpu_metrics"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn command(&self) -> String {
        "echo $(nproc); echo $(ps -A -o %cpu | awk '{s+=$1} END {print s}')".into()
    }

    fn parse(&self, raw: &str) -> Result<Self::Output, String> {
        let mut lines = raw.lines();
        let core_count = lines
            .next()
            .ok_or_else(|| "missing core count".to_string())?
            .trim()
            .parse::<usize>()
            .map_err(|e| format!("parse core count: {e}"))?;
        let usage_percent = lines
            .next()
            .ok_or_else(|| "missing usage percent".to_string())?
            .trim()
            .parse::<f32>()
            .map_err(|e| format!("parse usage percent: {e}"))?;
        Ok(CpuMetrics {
            core_count,
            usage_percent,
        })
    }
}
