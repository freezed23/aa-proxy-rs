//! Only diagnostic controls are live. Protocol/SDR behavior remains a connection
//! snapshot so a web edit cannot unexpectedly rewrite an active AA session.
use crate::config_types::HexdumpLevel;

#[derive(Clone, Debug, PartialEq)]
pub struct PacketDebugConfig {
    pub pkt_debug: bool,
    pub hexdump_level: HexdumpLevel,
    pub pkt_debug_full_frame_enabled: bool,
    pub pkt_debug_filter_enabled: bool,
    pub pkt_debug_filter_proxy: String,
    pub pkt_debug_filter_stages: String,
    pub pkt_debug_filter_service_kinds: String,
    pub pkt_debug_filter_channels: String,
    pub pkt_debug_filter_exclude_channels: String,
    pub pkt_debug_filter_message_ids: String,
    pub pkt_debug_filter_exclude_message_ids: String,
    pub pkt_debug_filter_pretty_proto: bool,
    pub pkt_debug_filter_max_payload_bytes: usize,
    pub pkt_debug_capture_dir: String,
}

impl Default for PacketDebugConfig {
    fn default() -> Self {
        Self {
            pkt_debug: false,
            hexdump_level: HexdumpLevel::Disabled,
            pkt_debug_full_frame_enabled: false,
            pkt_debug_filter_enabled: false,
            pkt_debug_filter_proxy: "both".into(),
            pkt_debug_filter_stages: String::new(),
            pkt_debug_filter_service_kinds: String::new(),
            pkt_debug_filter_channels: String::new(),
            pkt_debug_filter_exclude_channels: String::new(),
            pkt_debug_filter_message_ids: String::new(),
            pkt_debug_filter_exclude_message_ids: String::new(),
            pkt_debug_filter_pretty_proto: true,
            pkt_debug_filter_max_payload_bytes: 2048,
            pkt_debug_capture_dir: String::new(),
        }
    }
}
