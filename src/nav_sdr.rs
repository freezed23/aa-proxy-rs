//! Experimental HU->phone SDR-only navigation type override.
//! Never touches the HU-facing stream, service IDs, or AA protocol version.
use crate::mitm::protos::navigation_status_service::InstrumentClusterType;
use crate::mitm::protos::ServiceDiscoveryResponse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavSdrTypeReport {
    pub service_id: i32,
    pub original_type: Option<i32>,
    pub phone_type: Option<i32>,
    pub image_options_before: bool,
    pub image_options_after: bool,
    pub changed: bool,
}

/// By default the SDR is byte-for-byte unchanged by this function. When opted in,
/// only a known IMAGE(1) NavigationStatusService becomes ENUM(2); other services,
/// unknown type numbers, service IDs, and minimum intervals are preserved.
pub fn rewrite_phone_navigation_to_enum(
    response: &mut ServiceDiscoveryResponse,
    enabled: bool,
) -> Vec<NavSdrTypeReport> {
    response
        .services
        .iter_mut()
        .filter_map(|service| {
            let service_id = service.id();
            let nav = service.navigation_status_service.as_mut()?;
            let original_type = nav.type_.as_ref().map(|value| value.value());
            let image_options_before = nav.image_options.is_some();
            let changed = enabled && original_type == Some(InstrumentClusterType::IMAGE as i32);
            if changed {
                nav.set_type(InstrumentClusterType::ENUM);
                // IMAGE dimensions are not valid ENUM capability metadata.
                nav.image_options = protobuf::MessageField::none();
            }
            Some(NavSdrTypeReport {
                service_id,
                original_type,
                phone_type: nav.type_.as_ref().map(|value| value.value()),
                image_options_before,
                image_options_after: nav.image_options.is_some(),
                changed,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mitm::protos::navigation_status_service::ImageOptions;
    use crate::mitm::protos::{NavigationStatusService, Service};
    use protobuf::{Message, MessageField};

    fn test_sdr(cluster_type: InstrumentClusterType, with_image: bool) -> ServiceDiscoveryResponse {
        let mut nav = NavigationStatusService::new();
        nav.set_minimum_interval_ms(500);
        nav.set_type(cluster_type);
        if with_image {
            let mut image = ImageOptions::new();
            image.set_height(480);
            image.set_width(480);
            image.set_colour_depth_bits(32);
            nav.image_options = MessageField::some(image);
        }
        let mut service = Service::new();
        service.set_id(9);
        service.navigation_status_service = MessageField::some(nav);
        let mut sensor = Service::new();
        sensor.set_id(3);
        let mut response = ServiceDiscoveryResponse::new();
        response.services.push(service);
        response.services.push(sensor);
        response
    }

    #[test]
    fn disabled_preserves_original_protobuf_bytes() {
        let mut sdr = test_sdr(InstrumentClusterType::IMAGE, true);
        let before = sdr.write_to_bytes().expect("serialize original");
        let reports = rewrite_phone_navigation_to_enum(&mut sdr, false);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].original_type, Some(1));
        assert_eq!(reports[0].phone_type, Some(1));
        assert!(!reports[0].changed);
        assert_eq!(sdr.write_to_bytes().expect("serialize unchanged"), before);
    }

    #[test]
    fn enabled_changes_only_navigation_type_and_removes_image_dimensions() {
        let mut sdr = test_sdr(InstrumentClusterType::IMAGE, true);
        let before_sensor = sdr.services[1].write_to_bytes().expect("sensor");
        let reports = rewrite_phone_navigation_to_enum(&mut sdr, true);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].service_id, 9);
        assert_eq!(reports[0].original_type, Some(1));
        assert_eq!(reports[0].phone_type, Some(2));
        assert!(reports[0].changed);
        assert!(reports[0].image_options_before);
        assert!(!reports[0].image_options_after);
        let roundtrip = ServiceDiscoveryResponse::parse_from_bytes(&sdr.write_to_bytes().unwrap())
            .expect("parse rewritten SDR");
        let nav = roundtrip.services[0].navigation_status_service.as_ref().unwrap();
        assert_eq!(nav.type_(), InstrumentClusterType::ENUM);
        assert_eq!(nav.minimum_interval_ms(), 500);
        assert!(nav.image_options.is_none());
        assert_eq!(roundtrip.services[0].id(), 9);
        assert_eq!(roundtrip.services[1].write_to_bytes().unwrap(), before_sensor);
    }

    #[test]
    fn already_enum_is_untouched() {
        let mut sdr = test_sdr(InstrumentClusterType::ENUM, false);
        let before = sdr.write_to_bytes().unwrap();
        let reports = rewrite_phone_navigation_to_enum(&mut sdr, true);
        assert_eq!(reports.len(), 1);
        assert!(!reports[0].changed);
        assert_eq!(sdr.write_to_bytes().unwrap(), before);
    }

    #[test]
    fn unknown_type_is_not_coerced() {
        let mut sdr = test_sdr(InstrumentClusterType::IMAGE, false);
        sdr.services[0].navigation_status_service.as_mut().unwrap().type_ =
            Some(protobuf::EnumOrUnknown::from_i32(77));
        let before = sdr.write_to_bytes().unwrap();
        let reports = rewrite_phone_navigation_to_enum(&mut sdr, true);
        assert_eq!(reports[0].original_type, Some(77));
        assert_eq!(reports[0].phone_type, Some(77));
        assert!(!reports[0].changed);
        assert_eq!(sdr.write_to_bytes().unwrap(), before);
    }
}
