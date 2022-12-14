//! Small helper fns

/// Transform a duration into a formatted timer string like "29:30" (mm:ss)
pub fn seconds_to_time(duration: u64) -> String {
    let min = duration / 60;
    let sec = duration % 60;
    format!("{:02}:{:02}", min, sec)
}
