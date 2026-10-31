//! Presentation boundary for opaque HU metadata. Never write decoded text back
//! into an SDR: protobuf bytes fields retain the HU's original octets.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::Value;
use std::borrow::Cow;

pub(crate) fn display_text(value: &[u8]) -> Cow<'_, str> {
    match std::str::from_utf8(value) {
        Ok(text) => Cow::Borrowed(text),
        // Do not guess EUC-KR or replace distinct invalid values with U+FFFD.
        Err(_) => Cow::Owned(format!("non-UTF8[{}]", hex::encode(value))),
    }
}

// protobuf JSON encodes bytes as base64. Keep the diagnostic HTTP snapshot's
// previous readable text representation for these former string fields only.
// Unrelated bytes (icons/vendor data/etc.) must remain base64.
pub(crate) fn restore_json_text(value: &mut Value) {
    const HU_FIELDS: &[&str] = &[
        "make",
        "model",
        "year",
        "vehicleId",
        "headUnitMake",
        "headUnitModel",
        "headUnitSoftwareBuild",
        "headUnitSoftwareVersion",
    ];
    restore_fields(value, HU_FIELDS);
    restore_fields(value, &["displayName"]);
    if let Some(hu) = value.get_mut("headunitInfo") {
        restore_fields(hu, HU_FIELDS);
    }
    if let Some(services) = value.get_mut("services").and_then(Value::as_array_mut) {
        for service in services {
            if let Some(bt) = service.get_mut("bluetoothService") {
                restore_fields(bt, &["carAddress"]);
            }
            if let Some(vendor) = service.get_mut("vendorExtensionService") {
                restore_fields(vendor, &["serviceName"]);
                if let Some(packages) = vendor
                    .get_mut("packageWhiteList")
                    .and_then(Value::as_array_mut)
                {
                    for package in packages {
                        restore_value(package);
                    }
                }
            }
            if let Some(wifi) = service.get_mut("wifiProjectionService") {
                restore_fields(wifi, &["carWifiBssid"]);
            }
        }
    }
}

fn restore_fields(value: &mut Value, fields: &[&str]) {
    for field in fields {
        if let Some(text) = value.get_mut(*field) {
            restore_value(text);
        }
    }
}

fn restore_value(value: &mut Value) {
    if let Some(raw) = value.as_str().and_then(|text| STANDARD.decode(text).ok()) {
        *value = Value::String(display_text(&raw).into_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mitm::protos::ServiceDiscoveryResponse;
    use protobuf::Message;

    #[test]
    fn invalid_metadata_is_distinct_and_valid_utf8_is_unchanged() {
        assert_eq!(display_text("한글 HU".as_bytes()), "한글 HU");
        assert_eq!(display_text(&[0xff, 0x00]), "non-UTF8[ff00]");
        assert_ne!(display_text(&[0xff]), display_text(&[0xfe]));
    }

    #[test]
    fn diagnostic_json_keeps_text_and_opaque_data_separate() {
        let mut msg = ServiceDiscoveryResponse::new();
        msg.set_make(b"Example HU".to_vec());
        msg.set_display_name(vec![0xff]);
        let mut hu = crate::mitm::protos::HeadUnitInfo::new();
        hu.set_model("한글".as_bytes().to_vec());
        msg.headunit_info = protobuf::MessageField::some(hu);
        let mut service = crate::mitm::protos::Service::new();
        service.set_id(250);
        let mut vendor = crate::mitm::protos::VendorExtensionService::new();
        vendor.set_service_name(b"test".to_vec());
        vendor.package_white_list.push(b"test.package".to_vec());
        vendor.set_data(vec![0xff]);
        service.vendor_extension_service = protobuf::MessageField::some(vendor);
        msg.services.push(service);
        let wire_before = msg.write_to_bytes().unwrap();
        let json = protobuf_json_mapping::print_to_string(&msg).unwrap();
        let mut value: Value = serde_json::from_str(&json).unwrap();
        restore_json_text(&mut value);
        assert_eq!(value["make"], "Example HU");
        assert_eq!(value["displayName"], "non-UTF8[ff]");
        assert_eq!(value["headunitInfo"]["model"], "한글");
        let vendor = &value["services"][0]["vendorExtensionService"];
        assert_eq!(vendor["serviceName"], "test");
        assert_eq!(vendor["packageWhiteList"][0], "test.package");
        assert_eq!(vendor["data"], "/w==");
        assert_eq!(msg.write_to_bytes().unwrap(), wire_before);
    }
}

/// Keep identity separate from presentation (FF and literal "non-UTF8[ff]").
/// Return None for all-valid metadata so existing profile fingerprints survive.
pub(crate) fn opaque_identity(fields: &[&[u8]]) -> Option<String> {
    use sha2::{Digest, Sha256};
    if fields
        .iter()
        .all(|value| std::str::from_utf8(value).is_ok())
    {
        return None;
    }
    let mut hash = Sha256::new();
    hash.update(b"aa-sdr-opaque-identity-v1");
    for value in fields {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value);
    }
    Some(format!("veh_{}", &hex::encode(hash.finalize())[..12]))
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    #[test]
    fn display_collision_does_not_merge_raw_vehicle_identities() {
        assert_eq!(display_text(&[0xff]), display_text(b"non-UTF8[ff]"));
        assert!(opaque_identity(&[b"non-UTF8[ff]"]).is_none());
        assert!(opaque_identity(&[&[0xff]]).is_some());
        assert_ne!(
            opaque_identity(&[&[0xff], b"ab", b"c"]),
            opaque_identity(&[&[0xff], b"a", b"bc"])
        );
        assert_ne!(opaque_identity(&[&[0xff]]), opaque_identity(&[&[0xfe]]));
    }
}
