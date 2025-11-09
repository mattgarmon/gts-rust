#[cfg(test)]
mod tests {
    use crate::gts::*;

    #[test]
    fn test_gts_id_valid() {
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert_eq!(id.id, "gts.x.core.events.event.v1~");
        assert!(id.is_type());
        assert_eq!(id.gts_id_segments.len(), 1);
    }

    #[test]
    fn test_gts_id_with_minor_version() {
        let id = GtsID::new("gts.x.core.events.event.v1.2~").unwrap();
        assert_eq!(id.id, "gts.x.core.events.event.v1.2~");
        assert!(id.is_type());
        let seg = &id.gts_id_segments[0];
        assert_eq!(seg.vendor, "x");
        assert_eq!(seg.package, "core");
        assert_eq!(seg.namespace, "events");
        assert_eq!(seg.type_name, "event");
        assert_eq!(seg.ver_major, 1);
        assert_eq!(seg.ver_minor, Some(2));
    }

    #[test]
    fn test_gts_id_instance() {
        let id = GtsID::new("gts.x.core.events.event.v1.0").unwrap();
        assert_eq!(id.id, "gts.x.core.events.event.v1.0");
        assert!(!id.is_type());
    }

    #[test]
    fn test_gts_id_invalid_uppercase() {
        let result = GtsID::new("gts.X.core.events.event.v1~");
        assert!(result.is_err());
    }

    #[test]
    fn test_gts_id_invalid_no_prefix() {
        let result = GtsID::new("x.core.events.event.v1~");
        assert!(result.is_err());
    }

    #[test]
    fn test_gts_id_invalid_hyphen() {
        let result = GtsID::new("gts.x-vendor.core.events.event.v1~");
        assert!(result.is_err());
    }

    #[test]
    fn test_gts_wildcard_simple() {
        let pattern = GtsWildcard::new("gts.x.core.events.*").unwrap();
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert!(id.wildcard_match(&pattern));
    }

    #[test]
    fn test_gts_wildcard_no_match() {
        let pattern = GtsWildcard::new("gts.x.core.events.*").unwrap();
        let id = GtsID::new("gts.y.core.events.event.v1~").unwrap();
        assert!(!id.wildcard_match(&pattern));
    }

    #[test]
    fn test_gts_wildcard_type_suffix() {
        // Wildcard after ~ should match type IDs
        let pattern = GtsWildcard::new("gts.x.core.events.*").unwrap();
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert!(id.wildcard_match(&pattern));
    }

    #[test]
    fn test_uuid_generation() {
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        let uuid1 = id.to_uuid();
        let uuid2 = id.to_uuid();
        // UUIDs should be deterministic
        assert_eq!(uuid1, uuid2);
        assert!(!uuid1.to_string().is_empty());
    }

    #[test]
    fn test_uuid_different_ids() {
        let id1 = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        let id2 = GtsID::new("gts.x.core.events.event.v2~").unwrap();
        assert_ne!(id1.to_uuid(), id2.to_uuid());
    }

    #[test]
    fn test_get_type_id() {
        // get_type_id is for chained IDs - returns None for single segment
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        let type_id = id.get_type_id();
        assert!(type_id.is_none());

        // For chained IDs, it returns the base type
        let chained = GtsID::new("gts.x.core.events.type.v1~vendor.app._.custom.v1~").unwrap();
        let base_type = chained.get_type_id();
        assert!(base_type.is_some());
        assert_eq!(base_type.unwrap(), "gts.x.core.events.type.v1~");
    }

    #[test]
    fn test_split_at_path() {
        let (gts, path) =
            GtsID::split_at_path("gts.x.core.events.event.v1~@field.subfield").unwrap();
        assert_eq!(gts, "gts.x.core.events.event.v1~");
        assert_eq!(path, Some("field.subfield".to_string()));
    }

    #[test]
    fn test_split_at_path_no_path() {
        let (gts, path) = GtsID::split_at_path("gts.x.core.events.event.v1~").unwrap();
        assert_eq!(gts, "gts.x.core.events.event.v1~");
        assert_eq!(path, None);
    }

    #[test]
    fn test_split_at_path_empty_path_error() {
        let result = GtsID::split_at_path("gts.x.core.events.event.v1~@");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid() {
        assert!(GtsID::is_valid("gts.x.core.events.event.v1~"));
        assert!(!GtsID::is_valid("invalid"));
        assert!(!GtsID::is_valid("gts.X.core.events.event.v1~"));
    }

    #[test]
    fn test_version_flexibility_in_matching() {
        // Pattern without minor version should match any minor version
        let pattern = GtsWildcard::new("gts.x.core.events.event.v1~").unwrap();
        let id_no_minor = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        let id_with_minor = GtsID::new("gts.x.core.events.event.v1.0~").unwrap();

        assert!(id_no_minor.wildcard_match(&pattern));
        assert!(id_with_minor.wildcard_match(&pattern));
    }

    #[test]
    fn test_chained_identifiers() {
        let id = GtsID::new("gts.x.core.events.type.v1~vendor.app._.custom_event.v1~").unwrap();
        assert_eq!(id.gts_id_segments.len(), 2);
        assert_eq!(id.gts_id_segments[0].vendor, "x");
        assert_eq!(id.gts_id_segments[1].vendor, "vendor");
    }

    #[test]
    fn test_gts_id_segment_validation() {
        // Test invalid segment with special characters
        let result = GtsIdSegment::new(0, 0, "invalid-segment");
        assert!(result.is_err());

        // Test valid segment
        let result = GtsIdSegment::new(0, 0, "x.core.events.event.v1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_gts_id_with_underscore() {
        // Underscores are allowed in namespace
        let id = GtsID::new("gts.x.core._.event.v1~").unwrap();
        assert_eq!(id.gts_id_segments[0].namespace, "_");
    }

    #[test]
    fn test_gts_wildcard_exact_match() {
        let pattern = GtsWildcard::new("gts.x.core.events.event.v1~").unwrap();
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert!(id.wildcard_match(&pattern));
    }

    #[test]
    fn test_gts_wildcard_version_mismatch() {
        let pattern = GtsWildcard::new("gts.x.core.events.event.v2~").unwrap();
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert!(!id.wildcard_match(&pattern));
    }

    #[test]
    fn test_gts_wildcard_with_minor_version() {
        let pattern = GtsWildcard::new("gts.x.core.events.event.v1.0~").unwrap();
        let id = GtsID::new("gts.x.core.events.event.v1.0~").unwrap();
        assert!(id.wildcard_match(&pattern));
    }

    #[test]
    fn test_gts_wildcard_invalid_pattern() {
        let result = GtsWildcard::new("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_gts_id_invalid_version_format() {
        let result = GtsID::new("gts.x.core.events.event.vX~");
        assert!(result.is_err());
    }

    #[test]
    fn test_gts_id_missing_segments() {
        let result = GtsID::new("gts.x.core~");
        assert!(result.is_err());
    }

    #[test]
    fn test_gts_id_empty_segment() {
        let result = GtsID::new("gts.x..events.event.v1~");
        assert!(result.is_err());
    }

    #[test]
    fn test_gts_wildcard_multiple_wildcards_error() {
        let result = GtsWildcard::new("gts.*.*.*.*");
        assert!(result.is_err());
    }

    #[test]
    fn test_split_at_path_multiple_at_signs() {
        // Should only split at first @
        let (gts, path) =
            GtsID::split_at_path("gts.x.core.events.event.v1~@field@subfield").unwrap();
        assert_eq!(gts, "gts.x.core.events.event.v1~");
        assert_eq!(path, Some("field@subfield".to_string()));
    }

    #[test]
    fn test_gts_wildcard_instance_match() {
        let pattern = GtsWildcard::new("gts.x.core.events.*").unwrap();
        let id = GtsID::new("gts.x.core.events.event.v1.0").unwrap();
        assert!(id.wildcard_match(&pattern));
    }

    #[test]
    fn test_gts_id_whitespace_trimming() {
        let id = GtsID::new("  gts.x.core.events.event.v1~  ").unwrap();
        assert_eq!(id.id, "gts.x.core.events.event.v1~");
    }

    #[test]
    fn test_gts_wildcard_whitespace_trimming() {
        let pattern = GtsWildcard::new("  gts.x.core.events.*  ").unwrap();
        assert_eq!(pattern.id, "gts.x.core.events.*");
    }

    #[test]
    fn test_gts_id_long_chain() {
        let id = GtsID::new("gts.a.b.c.d.v1~e.f.g.h.v2~i.j.k.l.v3~").unwrap();
        assert_eq!(id.gts_id_segments.len(), 3);
    }

    #[test]
    fn test_gts_wildcard_only_at_end() {
        // Wildcard in middle should fail
        let result1 = GtsWildcard::new("gts.*.core.events.event.v1~");
        assert!(result1.is_err());

        // Wildcard at end should work
        let pattern2 = GtsWildcard::new("gts.x.core.events.*").unwrap();
        let id2 = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert!(id2.wildcard_match(&pattern2));
    }

    #[test]
    fn test_gts_id_version_without_minor() {
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert_eq!(id.gts_id_segments[0].ver_major, 1);
        assert_eq!(id.gts_id_segments[0].ver_minor, None);
    }

    #[test]
    fn test_gts_id_version_with_large_numbers() {
        let id = GtsID::new("gts.x.core.events.event.v99.999~").unwrap();
        assert_eq!(id.gts_id_segments[0].ver_major, 99);
        assert_eq!(id.gts_id_segments[0].ver_minor, Some(999));
    }

    #[test]
    fn test_gts_wildcard_no_wildcard_different_vendor() {
        let pattern = GtsWildcard::new("gts.x.core.events.event.v1~").unwrap();
        let id = GtsID::new("gts.y.core.events.event.v1~").unwrap();
        assert!(!id.wildcard_match(&pattern));
    }

    #[test]
    fn test_gts_id_invalid_double_tilde() {
        let result = GtsID::new("gts.x.core.events.event.v1~~");
        assert!(result.is_err());
    }

    #[test]
    fn test_split_at_path_with_hash() {
        // Hash is not a separator, should be part of the ID
        let (gts, path) = GtsID::split_at_path("gts.x.core.events.event.v1~#field").unwrap();
        assert_eq!(gts, "gts.x.core.events.event.v1~#field");
        assert_eq!(path, None);
    }
}
