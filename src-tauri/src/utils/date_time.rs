//! 日期时间工具
//!
//! 提供常用的日期时间处理函数

use chrono::{DateTime, Local, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// 获取当前UTC时间戳（秒）
pub fn current_timestamp() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

/// 获取当前UTC时间戳（毫秒）
pub fn current_timestamp_millis() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64
}

/// 格式化时间为ISO 8601字符串
pub fn format_iso8601(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// 解析ISO 8601字符串为DateTime<Utc>
pub fn parse_iso8601(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    s.parse::<DateTime<Utc>>()
}

/// 格式化时间为用户友好的字符串
pub fn format_user_friendly(dt: &DateTime<Utc>) -> String {
    let local_dt = dt.with_timezone(&Local);
    local_dt.format("%Y年%m月%d日 %H:%M:%S").to_string()
}

/// 获取格式化的当前时间
pub fn current_time_formatted() -> String {
    let now = Utc::now();
    format_user_friendly(&now)
}

/// 计算两个时间点之间的持续时间（秒）
pub fn duration_seconds(start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
    (end - start).num_seconds()
}

/// 计算两个时间点之间的持续时间（毫秒）
pub fn duration_millis(start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
    (end - start).num_milliseconds()
}

/// 格式化持续时间为可读字符串
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}秒", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        if remaining_seconds == 0 {
            format!("{}分钟", minutes)
        } else {
            format!("{}分{}秒", minutes, remaining_seconds)
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let remaining_minutes = (seconds % 3600) / 60;
        if remaining_minutes == 0 {
            format!("{}小时", hours)
        } else {
            format!("{}小时{}分钟", hours, remaining_minutes)
        }
    } else {
        let days = seconds / 86400;
        let remaining_hours = (seconds % 86400) / 3600;
        if remaining_hours == 0 {
            format!("{}天", days)
        } else {
            format!("{}天{}小时", days, remaining_hours)
        }
    }
}

/// 验证日期字符串格式
pub fn validate_date_string(date_str: &str) -> Result<(), String> {
    match parse_iso8601(date_str) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("无效的日期格式: {}", e)),
    }
}

/// 获取今天的开始时间（UTC）
pub fn today_start_utc() -> DateTime<Utc> {
    let now = Utc::now();
    let date = now.date_naive();
    date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc()
}

/// 获取今天的结束时间（UTC）
pub fn today_end_utc() -> DateTime<Utc> {
    let start = today_start_utc();
    start + chrono::Duration::days(1) - chrono::Duration::milliseconds(1)
}

/// 检查时间是否在指定范围内
pub fn is_time_within_range(time: DateTime<Utc>, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
    time >= start && time <= end
}

/// 添加指定天数到当前时间
pub fn add_days_to_now(days: i64) -> DateTime<Utc> {
    Utc::now() + chrono::Duration::days(days)
}

/// 获取相对时间描述
pub fn get_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff_seconds = (now - dt).num_seconds();

    if diff_seconds < 60 {
        "刚刚".to_string()
    } else if diff_seconds < 3600 {
        let minutes = diff_seconds / 60;
        format!("{}分钟前", minutes)
    } else if diff_seconds < 86400 {
        let hours = diff_seconds / 3600;
        format!("{}小时前", hours)
    } else if diff_seconds < 2592000 {
        // 30天
        let days = diff_seconds / 86400;
        format!("{}天前", days)
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 1600000000); // 2020年之后的时间戳
    }

    #[test]
    fn test_format_iso8601() {
        let dt = Utc::now();
        let formatted = format_iso8601(&dt);
        assert!(formatted.contains('T'));
        assert!(formatted.contains('Z'));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30秒");
        assert_eq!(format_duration(90), "1分30秒");
        assert_eq!(format_duration(3600), "1小时");
        assert_eq!(format_duration(3720), "1小时2分钟");
    }

    #[test]
    fn test_is_time_within_range() {
        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);
        let within = start + chrono::Duration::minutes(30);

        assert!(is_time_within_range(within, start, end));
        assert!(!is_time_within_range(
            start - chrono::Duration::minutes(1),
            start,
            end
        ));
    }
}
