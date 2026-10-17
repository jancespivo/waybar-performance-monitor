use std::fs::File;
use std::io::{self, ErrorKind, Write};
use std::os::unix::fs::FileExt;
use std::str;
use std::thread::sleep;
use std::time::Duration;

const PRESSURE_FILE_PATH: &str = "/proc/pressure/cpu";

// The Linux kernel calculates PSI averages every 2 seconds (PSI_FREQ = 2 * HZ).
// Polling at 2-second intervals matches the kernel update cadence and avoids redundant reads.
// Reference: https://docs.kernel.org/accounting/psi.html
const POLL_INTERVAL: Duration = Duration::from_secs(2);

const ALERT_INFO: &str = "info";
const ALERT_WARNING: &str = "warning";
const ALERT_CRITICAL: &str = "critical";

fn parse_avg10(contents: &str) -> Option<f64> {
    let some_line = contents.lines().find(|line| line.starts_with("some "))?;

    for token in some_line.split_whitespace() {
        if let Some(val_str) = token.strip_prefix("avg10=") {
            return val_str.parse::<f64>().ok();
        }
    }

    None
}

fn determine_alert_level(psi_percent: u64) -> Option<&'static str> {
    match psi_percent {
        0..5 => None,
        5..10 => Some(ALERT_INFO),
        10..=20 => Some(ALERT_WARNING),
        _ => Some(ALERT_CRITICAL),
    }
}

fn main() {
    let file = File::open(PRESSURE_FILE_PATH).expect("Failed to open /proc/pressure/cpu");
    let mut buffer = [0_u8; 256];
    let mut stdout = io::stdout().lock();

    loop {
        // Standard read_to_string requires lseek and two read syscalls per iteration.
        // Using read_at at offset 0 reads the procfs entry in a single pread64 syscall without heap allocation.
        let bytes_read = file
            .read_at(&mut buffer, 0)
            .expect("Failed to read /proc/pressure/cpu");

        let contents =
            str::from_utf8(&buffer[..bytes_read]).expect("Invalid UTF-8 in /proc/pressure/cpu");

        let avg10 = parse_avg10(contents).expect("Failed to parse avg10 from /proc/pressure/cpu");

        let psi_rounded = avg10.round() as u64;

        let write_result = if let Some(class) = determine_alert_level(psi_rounded) {
            writeln!(
                stdout,
                "{{\"text\": \"PSI {psi_rounded:02}%\", \"class\": \"{class}\"}}"
            )
        } else {
            writeln!(stdout, "{{\"text\": \"PSI {psi_rounded:02}%\"}}")
        };

        if let Err(err) = write_result {
            if err.kind() == ErrorKind::BrokenPipe {
                break;
            }
            panic!("Failed to write to stdout: {err}");
        }

        sleep(POLL_INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_avg10() {
        let sample = "some avg10=0.56 avg60=5.83 avg300=3.09 total=179048144\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
        assert_eq!(parse_avg10(sample), Some(0.56));
    }

    #[test]
    fn test_determine_alert_level() {
        assert_eq!(determine_alert_level(0), None);
        assert_eq!(determine_alert_level(4), None);
        assert_eq!(determine_alert_level(5), Some("info"));
        assert_eq!(determine_alert_level(9), Some("info"));
        assert_eq!(determine_alert_level(10), Some("warning"));
        assert_eq!(determine_alert_level(20), Some("warning"));
        assert_eq!(determine_alert_level(21), Some("critical"));
        assert_eq!(determine_alert_level(100), Some("critical"));
    }
}
