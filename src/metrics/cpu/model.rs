#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub core_count: usize,
    pub usage_percent: f32,
}
