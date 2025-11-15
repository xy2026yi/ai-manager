//! 性能监控和统计模块
//!
//! 用于监控应用程序的性能指标，包括数据库查询、内存使用、启动时间等

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 性能指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// 数据库查询时间
    DatabaseQuery,
    /// 数据库连接获取时间
    DatabaseConnection,
    /// 加密/解密操作时间
    Cryptography,
    /// API响应时间
    ApiResponse,
    /// 启动时间
    Startup,
    /// 内存使用
    MemoryUsage,
    /// 自定义指标
    Custom(String),
}

/// 性能指标记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// 指标类型
    pub metric_type: MetricType,
    /// 操作名称
    pub operation: String,
    /// 持续时间
    pub duration: Duration,
    /// 时间戳
    // pub timestamp: Instant,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    /// 附加元数据
    pub metadata: HashMap<String, String>,
}

impl PerformanceMetric {
    /// 创建新的性能指标
    pub fn new(metric_type: MetricType, operation: impl Into<String>, duration: Duration) -> Self {
        Self {
            metric_type,
            operation: operation.into(),
            duration,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// 性能统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// 总操作数
    pub total_operations: usize,
    /// 平均持续时间
    pub average_duration: Duration,
    /// 最小持续时间
    pub min_duration: Duration,
    /// 最大持续时间
    pub max_duration: Duration,
    /// 最近100次操作的平均时间
    pub recent_average: Duration,
    /// 每秒操作数
    pub operations_per_second: f64,
}

/// 性能监控器
#[derive(Clone)]
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<Vec<PerformanceMetric>>>,
    start_time: Instant,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// 记录性能指标
    pub async fn record_metric(&self, metric: PerformanceMetric) {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);

        // 保持最近10000条记录，避免内存泄漏
        if metrics.len() > 10_000 {
            metrics.drain(0..5_000);
        }
    }

    /// 执行操作并记录性能
    pub async fn timed_operation<F, Fut, T>(
        &self,
        metric_type: MetricType,
        operation: impl Into<String>,
        f: F,
    ) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();

        let metric = PerformanceMetric::new(metric_type, operation, duration);
        self.record_metric(metric).await;

        result
    }

    /// 获取指定类型的性能统计
    pub async fn get_summary(&self, metric_type: &MetricType) -> Option<PerformanceSummary> {
        let metrics = self.metrics.read().await;
        let filtered: Vec<_> = metrics
            .iter()
            .filter(|m| match (&m.metric_type, metric_type) {
                (MetricType::Custom(a), MetricType::Custom(b)) => a == b,
                (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
            })
            .collect();

        if filtered.is_empty() {
            return None;
        }

        let total_operations = filtered.len();
        let total_duration: Duration = filtered.iter().map(|m| m.duration).sum();
        let average_duration = total_duration / total_operations as u32;

        let min_duration = filtered.iter().map(|m| m.duration).min().unwrap();
        let max_duration = filtered.iter().map(|m| m.duration).max().unwrap();

        // 最近100次操作的平均时间
        let recent_count = filtered.len().min(100);
        let recent_average = if recent_count > 0 {
            let recent_duration: Duration =
                filtered.iter().rev().take(recent_count).map(|m| m.duration).sum();
            recent_duration / recent_count as u32
        } else {
            Duration::ZERO
        };

        // 计算每秒操作数
        let elapsed = self.start_time.elapsed();
        let operations_per_second = total_operations as f64 / elapsed.as_secs_f64();

        Some(PerformanceSummary {
            total_operations,
            average_duration,
            min_duration,
            max_duration,
            recent_average,
            operations_per_second,
        })
    }

    /// 获取所有性能指标
    pub async fn get_all_metrics(&self) -> Vec<PerformanceMetric> {
        self.metrics.read().await.clone()
    }

    /// 清除所有性能指标
    pub async fn clear_metrics(&self) {
        self.metrics.write().await.clear();
    }

    /// 获取启动时间
    pub fn startup_time(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能定时器，用于自动记录操作时间
pub struct PerformanceTimer {
    monitor: PerformanceMonitor,
    metric_type: MetricType,
    operation: String,
    start_time: Instant,
}

impl PerformanceTimer {
    /// 创建新的性能定时器
    pub fn new(
        monitor: PerformanceMonitor,
        metric_type: MetricType,
        operation: impl Into<String>,
    ) -> Self {
        Self {
            monitor,
            metric_type,
            operation: operation.into(),
            start_time: Instant::now(),
        }
    }

    /// 完成计时并记录指标
    pub async fn finish(self) {
        let duration = self.start_time.elapsed();
        // 使用 clone 避免移动所有权
        let metric =
            PerformanceMetric::new(self.metric_type.clone(), self.operation.clone(), duration);
        self.monitor.record_metric(metric).await;
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let metric =
            PerformanceMetric::new(self.metric_type.clone(), self.operation.clone(), duration);

        // 在Drop中使用spawn避免阻塞
        let monitor = self.monitor.clone();
        tokio::spawn(async move {
            monitor.record_metric(metric).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();

        // 测试定时操作
        monitor
            .timed_operation(MetricType::DatabaseQuery, "SELECT * FROM users", || async {
                sleep(Duration::from_millis(10)).await;
                "result"
            })
            .await;

        // 检查统计
        let summary = monitor.get_summary(&MetricType::DatabaseQuery).await;
        assert!(summary.is_some());

        let summary = summary.unwrap();
        assert_eq!(summary.total_operations, 1);
        assert!(summary.average_duration >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_performance_timer() {
        let monitor = PerformanceMonitor::new();

        {
            let _timer = PerformanceTimer::new(
                monitor.clone(),
                MetricType::Custom("test_operation".to_string()),
                "test_operation",
            );

            sleep(Duration::from_millis(5)).await;
            // timer会在drop时自动记录
        }

        // 等待异步记录完成
        sleep(Duration::from_millis(10)).await;

        let summary = monitor.get_summary(&MetricType::Custom("test_operation".to_string())).await;
        assert!(summary.is_some());
    }

    #[tokio::test]
    async fn test_performance_summary() {
        let monitor = PerformanceMonitor::new();

        // 添加多个指标
        for i in 0..5 {
            let metric = PerformanceMetric::new(
                MetricType::ApiResponse,
                format!("api_call_{}", i),
                Duration::from_millis(100 + i * 10),
            );
            monitor.record_metric(metric).await;
        }

        let summary = monitor.get_summary(&MetricType::ApiResponse).await;
        assert!(summary.is_some());

        let summary = summary.unwrap();
        assert_eq!(summary.total_operations, 5);
        assert_eq!(summary.min_duration, Duration::from_millis(100));
        assert_eq!(summary.max_duration, Duration::from_millis(140));
    }
}
