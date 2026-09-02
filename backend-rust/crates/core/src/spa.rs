pub const ADMIN_BUNDLE_PREFIXES: [&str; 2] = ["/admin", "/event-mgmt"];
pub const ADMIN_BUNDLE_ENTRY: &str = "/admin.html";

pub fn wants_admin_bundle(path: &str) -> bool {
    ADMIN_BUNDLE_PREFIXES.iter().any(|prefix| {
        path == *prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_and_event_mgmt_roots_take_the_admin_bundle() {
        assert!(wants_admin_bundle("/admin"));
        assert!(wants_admin_bundle("/event-mgmt"));
    }

    #[test]
    fn nested_admin_routes_take_the_admin_bundle() {
        for path in [
            "/admin/users",
            "/admin/analytics",
            "/admin/sales",
            "/admin/system",
            "/event-mgmt/categories",
            "/event-mgmt/venues",
            "/event-mgmt/media/11",
        ] {
            assert!(
                wants_admin_bundle(path),
                "{path} must load the admin bundle"
            );
        }
    }

    #[test]
    fn public_routes_keep_the_public_bundle() {
        for path in [
            "/",
            "/login",
            "/register",
            "/events",
            "/events/11",
            "/events/11/participants",
            "/vvip",
            "/profile",
            "/profile/registrations",
        ] {
            assert!(
                !wants_admin_bundle(path),
                "{path} must load the public bundle"
            );
        }
    }

    #[test]
    fn a_prefix_only_matches_on_a_segment_boundary() {
        for path in [
            "/administrators",
            "/admin-backup",
            "/adminx/users",
            "/event-mgmtx",
            "/event-mgmt-archive",
        ] {
            assert!(
                !wants_admin_bundle(path),
                "{path} merely starts with the text of a prefix"
            );
        }
    }

    #[test]
    fn api_paths_are_never_the_admin_bundle() {
        for path in [
            "/api/admin/database/stats",
            "/api/events",
            "/api/auth/login",
        ] {
            assert!(!wants_admin_bundle(path), "{path} belongs to the API");
        }
    }
}
