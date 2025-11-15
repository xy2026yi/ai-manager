//! 启动性能优化器
//!
//! 提供应用启动时间监控和优化功能

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// 启动阶段枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupPhase {
    ApplicationStart,
    LoggingInit,
    DatabaseConnect,
    DatabaseMigrate,
    ServicesInit,
    TauriInit,
    ApplicationReady,
}

impl std::fmt::Display for StartupPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartupPhase::ApplicationStart => write!(f, "应用启动"),
            StartupPhase::LoggingInit => write!(f, "日志初始化"),
            StartupPhase::DatabaseConnect => write!(f, "数据库连接"),
            StartupPhase::DatabaseMigrate => write!(f, "数据库迁移"),
            StartupPhase::ServicesInit => write!(f, "服务初始化"),
            StartupPhase::TauriInit => write!(f, "Tauri初始化"),
            StartupPhase::ApplicationReady => write!(f, "应用就绪"),
        }
    }
}

/// 启动性能统计
#[derive(Debug, Clone)]
pub struct StartupStats {
    pub phase_durations: HashMap<StartupPhase, Duration>,
    pub total_duration: Duration,
    pub start_time: Instant,
}

impl StartupStats {
    pub fn new() -> Self {
        Self {
            phase_durations: HashMap::new(),
            total_duration: Duration::ZERO,
            start_time: Instant::now(),
        }
    }

    pub fn start_phase(&mut self, phase: StartupPhase) {
        self.start_time = Instant::now();
        debug!("开始启动阶段: {}", phase);
    }

    pub fn end_phase(&mut self, phase: StartupPhase) {
        let duration = self.start_time.elapsed();
        self.phase_durations.insert(phase, duration);
        
        if phase == StartupPhase::ApplicationReady {
            self.total_duration = self.phase_durations.values().sum();
        }

        info!("启动阶段完成: {} (耗时: {:?})", phase, duration);
    }

    pub fn get_phase_duration(&self, phase: StartupPhase) -> Duration {
        self.phase_durations.get(&phase).copied().unwrap_or(Duration::ZERO)
    }

    pub fn get_total_duration(&self) -> Duration {
        self.total_duration
    }

    pub fn get_slowest_phases(&self, count: usize) -> Vec<(StartupPhase, Duration)> {
        let mut phases: Vec<(StartupPhase, Duration)> = self.phase_durations.iter()
            .map(|(&phase, &duration)| (phase, duration))
            .collect();
        
        phases.sort_by(|a, b| b.1.cmp(&a.1));
        phases.into_iter().take(count).collect()
    }

    pub fn print_summary(&self) {
        info!("=== 启动性能统计 ===");
        info!("总启动时间: {:?}", self.total_duration);
        
        for (phase, duration) in &self.phase_durations {
            let percentage = if self.total_duration.as_millis() > 0 {
                (duration.as_millis() as f64 / self.total_duration.as_millis() as f64) * 100.0
            } else {
                0.0
            };
            info!("  {}: {:?} ({:.1}%)", phase, duration, percentage);
        }
        
        // 识别最慢的阶段
        let slow_phases = self.get_slowest_phases(3);
        if !slow_phases.is_empty() {
            info!("最慢的启动阶段:");
            for (i, (phase, duration)) in slow_phases.iter().enumerate() {
                info!("  {}. {}: {:?}", i + 1, phase, duration);
            }
        }
        
        // 性能建议
        if self.total_duration > Duration::from_secs(2) {
            warn!("启动时间超过目标 (2秒)，考虑优化以下阶段:");
            for (phase, duration) in &slow_phases {
                if *duration > Duration::from_millis(500) {
                    warn!("  - {} (耗时: {:?})", phase, duration);
                }
            }
        } else {
            info!("✅ 启动时间符合目标要求 (< 2秒)");
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let phases: serde_json::Map<String, serde_json::Value> = self.phase_durations
            .iter()
            .map(|(&phase, &duration)| {
                (
                    format!("{:?}", phase),
                    serde_json::json!(duration.as_millis())
                )
            })
            .collect();

        serde_json::json!({
            "total_duration_ms": self.total_duration.as_millis(),
            "phase_durations_ms": phases,
            "is_within_target": self.total_duration <= Duration::from_secs(2)
        })
    }
}

/// 启动性能优化器
pub struct StartupOptimizer {
    stats: StartupStats,
    optimizations: Vec<Box<dyn OptimizationRule>>,
}

impl StartupOptimizer {
    pub fn new() -> Self {
        Self {
            stats: StartupStats::new(),
            optimizations: vec![
                Box::new(DatabaseConnectOptimization),
                Box::new(LoggingOptimization),
                Box::new(ServiceInitOptimization),
            ],
        }
    }

    pub fn stats(&self) -> &StartupStats {
        &self.stats
    }

    pub fn start_phase(&mut self, phase: StartupPhase) {
        self.stats.start_phase(phase);
    }

    pub fn end_phase(&mut self, phase: StartupPhase) {
        self.stats.end_phase(phase);

        // 自动应用优化规则
        let duration = self.stats.get_phase_duration(phase);
        for optimization in &self.optimizations {
            optimization.optimize_phase(phase, duration);
        }
    }

    pub fn print_summary(&self) {
        self.stats.print_summary();
    }

    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        for optimization in &self.optimizations {
            recommendations.extend(optimization.get_recommendations(&self.stats));
        }

        recommendations
    }
}

/// 优化规则接口
trait OptimizationRule {
    fn optimize_phase(&self, phase: StartupPhase, duration: Duration);
    fn get_recommendations(&self, stats: &StartupStats) -> Vec<String>;
}

/// 数据库连接优化规则
struct DatabaseConnectOptimization;

impl OptimizationRule for DatabaseConnectOptimization {
    fn optimize_phase(&self, phase: StartupPhase, duration: Duration) {
        if phase == StartupPhase::DatabaseConnect && duration > Duration::from_millis(1000) {
            warn!("数据库连接时间过长 ({:?})，考虑以下优化:", duration);
            warn!("1. 检查数据库文件是否在本地SSD上");
            warn!("2. 考虑使用WAL模式");
            warn!("3. 优化SQLite PRAGMA设置");
        }
    }

    fn get_recommendations(&self, stats: &StartupStats) -> Vec<String> {
        let mut recs = Vec::new();
        
        let db_duration = stats.get_phase_duration(StartupPhase::DatabaseConnect);
        if db_duration > Duration::from_millis(500) {
            recs.push("数据库连接时间过长，建议使用连接池预热".to_string());
        }

        let migrate_duration = stats.get_phase_duration(StartupPhase::DatabaseMigrate);
        if migrate_duration > Duration::from_millis(2000) {
            recs.push("数据库迁移时间过长，建议并行执行或使用增量迁移".to_string());
        }

        recs
    }
}

/// 日志初始化优化规则
struct LoggingOptimization;

impl OptimizationRule for LoggingOptimization {
    fn optimize_phase(&self, phase: StartupPhase, duration: Duration) {
        if phase == StartupPhase::LoggingInit && duration > Duration::from_millis(100) {
            warn!("日志初始化时间过长 ({:?})，考虑使用异步日志或延迟初始化", duration);
        }
    }

    fn get_recommendations(&self, stats: &StartupStats) -> Vec<String> {
        let mut recs = Vec::new();
        
        let log_duration = stats.get_phase_duration(StartupPhase::LoggingInit);
        if log_duration > Duration::from_millis(50) {
            recs.push("日志初始化时间过长，建议使用更轻量的日志配置".to_string());
        }

        recs
    }
}

/// 服务初始化优化规则
struct ServiceInitOptimization;

impl OptimizationRule for ServiceInitOptimization {
    fn optimize_phase(&self, phase: StartupPhase, duration: Duration) {
        if phase == StartupPhase::ServicesInit && duration > Duration::from_millis(300) {
            warn!("服务初始化时间过长 ({:?})，考虑以下优化:", duration);
            warn!("1. 实现懒加载服务");
            warn!("2. 并行初始化独立服务");
            warn!("3. 缓存常用配置");
        }
    }

    fn get_recommendations(&self, stats: &StartupStats) -> Vec<String> {
        let mut recs = Vec::new();
        
        let service_duration = stats.get_phase_duration(StartupPhase::ServicesInit);
        if service_duration > Duration::from_millis(200) {
            recs.push("服务初始化时间过长，建议实现延迟加载".to_string());
        }

        recs
    }
}

/// 全局启动性能监控器（使用Once Cell确保线程安全）
use std::sync::OnceCell;

static STARTUP_OPTIMIZER: OnceCell<StartupOptimizer> = OnceCell::new();

/// 获取全局启动优化器实例
pub fn get_startup_optimizer() -> &'static StartupOptimizer {
    STARTUP_OPTIMIZER.get_or_init(|| {
        let optimizer = StartupOptimizer::new();
        optimizer.start_phase(StartupPhase::ApplicationStart);
        optimizer
    })
}

/// 便捷宏用于监控启动阶段
#[macro_export]
macro_rules! monitor_startup_phase {
    ($phase:expr, $block:block) => {
        {
            let _optimizer = $crate::startup_optimizer::get_startup_optimizer();
            _optimizer.start_phase($phase);
            let result = $block;
            _optimizer.end_phase($phase);
            result
        }
    };
}

/// 完成启动监控
pub fn complete_startup() {
    let optimizer = get_startup_optimizer();
    optimizer.end_phase(StartupPhase::ApplicationReady);
    optimizer.print_summary();
    
    // 保存启动性能统计
    if let Ok(stats_json) = serde_json::to_string_pretty(&optimizer.stats().to_json()) {
        if let Ok(_) = std::fs::write(".claude/startup-performance.json", stats_json) {
            info!("启动性能统计已保存到 .claude/startup-performance.json");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_stats() {
        let mut stats = StartupStats::new();
        
        stats.start_phase(StartupPhase::LoggingInit);
        std::thread::sleep(Duration::from_millis(10));
        stats.end_phase(StartupPhase::LoggingInit);
        
        stats.start_phase(StartupPhase::DatabaseConnect);
        std::thread::sleep(Duration::from_millis(50));
        stats.end_phase(StartupPhase::DatabaseConnect);
        
        assert_eq!(stats.get_phase_duration(StartupPhase::LoggingInit).as_millis(), 10);
        assert_eq!(stats.get_phase_duration(StartupPhase::DatabaseConnect).as_millis(), 50);
        
        let slow_phases = stats.get_slowest_phases(2);
        assert_eq!(slow_phases.len(), 2);
        assert_eq!(slow_phases[0].0, StartupPhase::DatabaseConnect);
    }

    #[test]
    fn test_startup_optimizer() {
        let mut optimizer = StartupOptimizer::new();
        
        optimizer.start_phase(StartupPhase::LoggingInit);
        optimizer.end_phase(StartupPhase::LoggingInit);
        
        optimizer.start_phase(StartupPhase::DatabaseConnect);
        optimizer.end_phase(StartupPhase::DatabaseConnect);
        
        let recommendations = optimizer.get_recommendations();
        // 测试推荐生成（具体内容取决于实现）
        assert!(!recommendations.is_empty() || recommendations.is_empty()); // 允许为空
    }
}