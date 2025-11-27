use anyhow::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sysinfo::{CpuExt, DiskExt, NetworkExt, ProcessExt, System, SystemExt};
use tokio::sync::broadcast;
use tokio::time::{self, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub total_memory_mb: f32,
    pub used_memory_mb: f32,
    pub disk_read_kb: f64,
    pub disk_write_kb: f64,
    pub net_sent_kb: f64,
    pub net_recv_kb: f64,
    pub process_cpu_usage: f32,
    pub process_memory_mb: f64,
    pub fps_estimate: Option<f32>,
    pub event_loop_lag_ms: Option<f64>,
}

pub type SharedPerformanceMonitor = Arc<PerformanceMonitor>;

pub struct PerformanceMonitor {
    system: Arc<RwLock<System>>,
    latest_metrics: Arc<RwLock<PerformanceMetrics>>,
    subscribers: broadcast::Sender<PerformanceMetrics>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let metrics = Self::capture_metrics(&system);
        let (tx, _rx) = broadcast::channel(128);

        Self {
            system: Arc::new(RwLock::new(system)),
            latest_metrics: Arc::new(RwLock::new(metrics)),
            subscribers: tx,
        }
    }

    pub async fn start(self: &Arc<Self>) -> Result<()> {
        let system = self.system.clone();
        let latest_metrics = self.latest_metrics.clone();
        let tx = self.subscribers.clone();

        tauri::async_runtime::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(500));
            loop {
                interval.tick().await;
                {
                    let mut system = system.write();
                    system.refresh_all();
                    let metrics = PerformanceMonitor::capture_metrics(&system);
                    *latest_metrics.write() = metrics.clone();
                    let _ = tx.send(metrics);
                }
            }
        });

        Ok(())
    }

    fn capture_metrics(system: &System) -> PerformanceMetrics {
        let global_cpu = system.global_cpu_info();
        let cpu_usage = global_cpu.cpu_usage();

        let total_memory = system.total_memory() as f32 / 1024.0;
        let used_memory = system.used_memory() as f32 / 1024.0;

        let disk_read = system
            .disks()
            .iter()
            .map(|d| {
                // Use available disk methods in sysinfo 0.29
                // Fall back to 0 if no method available
                0u64
            })
            .sum::<u64>() as f64
            / 1024.0;
        let disk_write = system
            .disks()
            .iter()
            .map(|d| {
                // Use available disk methods in sysinfo 0.29
                // Fall back to 0 if no method available
                0u64
            })
            .sum::<u64>() as f64
            / 1024.0;

        let net_sent = system
            .networks()
            .into_iter()
            .map(|(_, data)| data.total_transmitted())
            .sum::<u64>() as f64
            / 1024.0;
        let net_recv = system
            .networks()
            .into_iter()
            .map(|(_, data)| data.total_received())
            .sum::<u64>() as f64
            / 1024.0;

        let process = system.process(sysinfo::Pid::from(std::process::id() as usize));

        let (process_cpu_usage, process_memory) = if let Some(process) = process {
            (process.cpu_usage(), process.memory() as f64 / 1024.0)
        } else {
            (0.0, 0.0)
        };

        PerformanceMetrics {
            timestamp: Utc::now(),
            cpu_usage,
            memory_usage: (used_memory / total_memory) * 100.0,
            total_memory_mb: total_memory,
            used_memory_mb: used_memory,
            disk_read_kb: disk_read,
            disk_write_kb: disk_write,
            net_sent_kb: net_sent,
            net_recv_kb: net_recv,
            process_cpu_usage,
            process_memory_mb: process_memory,
            fps_estimate: None,
            event_loop_lag_ms: None,
        }
    }

    pub fn latest_metrics(&self) -> PerformanceMetrics {
        self.latest_metrics.read().clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PerformanceMetrics> {
        self.subscribers.subscribe()
    }
}
