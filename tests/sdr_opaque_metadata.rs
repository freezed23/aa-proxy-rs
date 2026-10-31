//! Synthetic wire fixtures only. No private real-car logs or credentials.
use aa_proxy_rs::mitm::protos::{HeadUnitInfo, ServiceDiscoveryResponse};
use protobuf::Message;

#[test]
fn non_utf8_metadata_parses_and_survives_an_sdr_edit() {
    // make=FF00, headunit_info.model=FE; service 9 advertises ENUM navigation.
    let raw = [
        0x0a, 0x09, 0x08, 0x09, 0x42, 0x05, 0x08, 0xe8, 0x07, 0x10, 0x02, 0x12, 0x02, 0xff, 0x00,
        0x8a, 0x01, 0x03, 0x12, 0x01, 0xfe,
    ];
    // Build the navigation fixture explicitly (minimum_interval_ms/type required).
    // Keep an unknown top-level varint to check unknown-field retention too.
    let mut raw = raw.to_vec();
    raw.extend_from_slice(&[0xa0, 0x06, 0x07]);
    let mut msg = ServiceDiscoveryResponse::parse_from_bytes(&raw).unwrap();
    assert_eq!(msg.make(), &[0xff, 0x00]);
    assert_eq!(msg.headunit_info.model(), &[0xfe]);
    assert_eq!(msg.services[0].id(), 9);
    assert_eq!(
        msg.services[0]
            .navigation_status_service
            .minimum_interval_ms(),
        1000
    );
    msg.set_session_configuration(1);
    let reparsed =
        ServiceDiscoveryResponse::parse_from_bytes(&msg.write_to_bytes().unwrap()).unwrap();
    assert_eq!(reparsed.make(), msg.make());
    assert_eq!(reparsed.headunit_info.model(), msg.headunit_info.model());
    assert_eq!(reparsed.services, msg.services);
    assert_eq!(
        reparsed.special_fields.unknown_fields(),
        msg.special_fields.unknown_fields()
    );
}

#[test]
fn legacy_utf8_and_optional_presence_stay_intact() {
    let mut msg = ServiceDiscoveryResponse::parse_from_bytes(b"\x12\x06Google").unwrap();
    assert_eq!(msg.make(), b"Google");
    assert!(!msg.has_display_name());
    msg.set_display_name(Vec::new());
    let mut hu = HeadUnitInfo::new();
    hu.set_model("테스트".as_bytes().to_vec());
    msg.headunit_info = protobuf::MessageField::some(hu);
    let reparsed =
        ServiceDiscoveryResponse::parse_from_bytes(&msg.write_to_bytes().unwrap()).unwrap();
    assert!(reparsed.has_display_name());
    assert_eq!(reparsed.display_name(), b"");
    assert_eq!(reparsed.headunit_info.model(), "테스트".as_bytes());
}

#[test]
fn opaque_service_metadata_including_repeated_packages_round_trips() {
    // BluetoothService, VendorExtensionService and WifiProjectionService.
    let raw = [
        0x0a, 0x07, 0x08, 0x06, 0x32, 0x03, 0x0a, 0x01, 0xff, 0x0a, 0x0d, 0x08, 0x0f, 0x62, 0x09,
        0x0a, 0x01, 0xfe, 0x12, 0x01, 0xfd, 0x12, 0x01, 0xfc, 0x0a, 0x07, 0x08, 0x0c, 0x72, 0x03,
        0x0a, 0x01, 0xfb,
    ];
    let msg = ServiceDiscoveryResponse::parse_from_bytes(&raw).unwrap();
    assert_eq!(msg.services[0].bluetooth_service.car_address(), &[0xff]);
    assert_eq!(
        msg.services[1].vendor_extension_service.package_white_list,
        vec![vec![0xfd], vec![0xfc]]
    );
    assert_eq!(
        msg.services[2].wifi_projection_service.car_wifi_bssid(),
        &[0xfb]
    );
    let reparsed =
        ServiceDiscoveryResponse::parse_from_bytes(&msg.write_to_bytes().unwrap()).unwrap();
    assert_eq!(reparsed, msg);
}

#[test]
fn corrupt_lengths_and_missing_required_fields_still_fail() {
    assert!(ServiceDiscoveryResponse::parse_from_bytes(&[0x12, 0x02, 0xff]).is_err());
    // Service id=6 but BluetoothService required car_address is absent.
    assert!(
        ServiceDiscoveryResponse::parse_from_bytes(&[0x0a, 0x04, 0x08, 0x06, 0x32, 0x00]).is_err()
    );
}
