"""Source integration guards supplement the executable focused Rust tests."""
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]


class IntegrationGuards(unittest.TestCase):
    def test_all_web_config_update_logs_use_value_free_boundary(self):
        text = (ROOT / "src/web.rs").read_text(encoding="utf-8")
        self.assertEqual(text.count("diagnostic_privacy::config_update_log("), 3)
        self.assertNotRegex(text, r'(?:WASM )?[Cc]onfig entry updated:.*=')
        # Values still reach persistence; masking must not change the saved value.
        self.assertIn('doc[&entry.key] = toml_val;', text)
        self.assertIn('.update_custom_config_entry(&key, value.clone())', text)

    def test_wifi_error_paths_use_redacted_boundaries(self):
        text = (ROOT / "src/bluetooth.rs").read_text(encoding="utf-8")
        for name in ["run_wifi_join_custom_cmd", "run_nmcli_wifi_join", "wpa_cli_output", "run_shell_for_wifi"]:
            body = text.split(f"async fn {name}(", 1)[1].split("\n}", 1)[0]
            self.assertIn("diagnostic_privacy::wifi_command_failure", body, name)
            self.assertNotIn('stdout.trim()', body, name)
            self.assertNotIn('stderr.trim()', body, name)
            self.assertNotRegex(body, r'NAME, label, command')
        body = text.split('fn wpactrl_cmd(', 1)[1].split('\n}', 1)[0]
        self.assertIn('wifi_control_failure(false)', body)
        self.assertIn('wifi_control_failure(true)', body)
        self.assertNotIn('format!', body)

    def test_original_decrypted_logging_precedes_endpoint_hooks(self):
        text = (ROOT / "src/mitm.rs").read_text(encoding="utf-8")
        body = text.split('// handling input data from the reader thread')[-1]
        start = body.index('Ok(_) =>')
        self.assertLess(body.index('pkt_debug_with_full_frame(', start), body.index('pkt_modify_hook(', start))
        self.assertIn('PacketFlow::FromEndpoint', body)
        self.assertEqual(text.count('refresh_packet_debug(&mut debug_cfg, &config).await;'), 4)

    def test_live_snapshot_contains_only_diagnostics_and_matches_all_fields(self):
        snapshot = (ROOT / 'src/packet_debug_config.rs').read_text(encoding='utf-8')
        fields = re.findall(r'pub (\w+):', snapshot)
        self.assertTrue(all(f.startswith('pkt_debug') or f == 'hexdump_level' for f in fields))
        config = (ROOT / 'src/config.rs').read_text(encoding='utf-8')
        adapter = config.split('impl From<&AppConfig> for crate::packet_debug_config::PacketDebugConfig')[1].split('\n}\n', 1)[0]
        matching = config.split('fn matches_app_config(')[1]
        for field in fields:
            self.assertIn(f'{field}: cfg.{field}', adapter)
            self.assertIn(f'self.{field} == cfg.{field}', matching)

    def test_capture_is_opt_in_and_config_json_remains_valid(self):
        data = json.loads((ROOT / 'static/config.json').read_text(encoding='utf-8'))
        self.assertIn('pkt_debug_capture_dir', json.dumps(data))
        config = (ROOT / 'src/config.rs').read_text(encoding='utf-8')
        self.assertIn('pkt_debug_capture_dir: String::new()', config)
        self.assertIn('doc["pkt_debug_capture_dir"]', config)


if __name__ == '__main__':
    unittest.main(verbosity=2)
