use std::time::Duration;

pub fn get_max_age<'a>(cache_control: &'a str) -> Option<Duration> {
    let s = cache_control
        .split(",")
        .map(str::trim)
        .find(|s| s.starts_with("max-age"))?;
    let max_age = s.chars().skip(8).collect::<String>().parse::<u64>().ok()?;
    Some(Duration::from_secs(max_age))
}
