//! Logging boundaries: never format credential-bearing values or subprocess
//! arguments/output, even on failure. The original data is still sent unchanged.
pub(crate) fn config_update_log(key: &str) -> String {
    format!("Config entry updated: {} (value omitted)", key)
}

pub(crate) fn wifi_command_failure(program: &str, status: &str) -> String {
    format!(
        "{} failed status={} (arguments/output redacted)",
        program, status
    )
}

pub(crate) fn wifi_control_failure(failed: bool) -> &'static str {
    if failed {
        "wpactrl returned FAIL (command/output redacted)"
    } else {
        "wpactrl request failed (command/output redacted)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_updates_omit_all_values_including_custom_secrets() {
        for key in ["wpa_passphrase", "wasm.custom_token", "pkt_debug"] {
            let text = config_update_log(key);
            assert!(text.contains(key));
            assert!(text.contains("value omitted"));
            assert!(!text.contains("synthetic-secret-123"));
        }
    }

    #[test]
    fn failed_commands_expose_neither_arguments_nor_child_output() {
        for command in ["custom Wi-Fi join", "nmcli", "wpa_cli", "Wi-Fi shell"] {
            let text = wifi_command_failure(command, "exit status: 1");
            assert!(text.contains("exit status: 1"));
            assert!(text.contains("redacted"));
            assert!(!text.contains("synthetic-secret-123"));
        }
        assert!(!wifi_control_failure(true).contains("SET_NETWORK"));
        assert!(!wifi_control_failure(false).contains("psk"));
    }

    #[cfg(unix)]
    #[test]
    fn failure_injection_does_not_reemit_child_secrets() {
        let result = std::process::Command::new("sh")
            .args([
                "-c",
                "printf 'synthetic-psk-123'; printf 'synthetic-psk-123' >&2; exit 7",
            ])
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stdout).contains("synthetic-psk-123"));
        assert!(String::from_utf8_lossy(&result.stderr).contains("synthetic-psk-123"));
        let summary = wifi_command_failure("test Wi-Fi helper", &result.status.to_string());
        assert!(!summary.contains("synthetic-psk-123"));
        assert!(summary.contains('7'));
    }
}
