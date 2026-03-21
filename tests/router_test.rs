use slipstream::input::router::{CommandRouter, RouteResult};

#[test]
fn test_route_known_command() {
    let router = CommandRouter::new();
    let result = router.route("!tunnel add --type socks -p 1080");
    assert_eq!(
        result,
        RouteResult::SlipstreamCommand {
            command: "tunnel".to_string(),
            args: "add --type socks -p 1080".to_string(),
        }
    );
}

#[test]
fn test_route_sessions_command() {
    let router = CommandRouter::new();
    let result = router.route("!sessions");
    assert_eq!(
        result,
        RouteResult::SlipstreamCommand {
            command: "sessions".to_string(),
            args: "".to_string(),
        }
    );
}

#[test]
fn test_route_help_alias() {
    let router = CommandRouter::new();
    let result = router.route("!?");
    assert_eq!(
        result,
        RouteResult::SlipstreamCommand {
            command: "?".to_string(),
            args: "".to_string(),
        }
    );
}

#[test]
fn test_route_unknown_bang_passes_through() {
    let router = CommandRouter::new();
    assert_eq!(router.route("!!"), RouteResult::Passthrough);
}

#[test]
fn test_route_bang_dollar_passes_through() {
    let router = CommandRouter::new();
    assert_eq!(router.route("!$"), RouteResult::Passthrough);
}

#[test]
fn test_route_normal_command_passes_through() {
    let router = CommandRouter::new();
    assert_eq!(router.route("ls -la /etc"), RouteResult::Passthrough);
}

#[test]
fn test_route_bare_bang_passes_through() {
    let router = CommandRouter::new();
    assert_eq!(router.route("!"), RouteResult::Passthrough);
}

#[test]
fn test_route_custom_prefix() {
    let router = CommandRouter::with_prefix("@");
    let result = router.route("@tunnel list");
    assert_eq!(
        result,
        RouteResult::SlipstreamCommand {
            command: "tunnel".to_string(),
            args: "list".to_string(),
        }
    );
}

#[test]
fn test_route_upload_with_flags() {
    let router = CommandRouter::new();
    let result = router.route("!upload --method scp linpeas.sh /tmp/");
    assert_eq!(
        result,
        RouteResult::SlipstreamCommand {
            command: "upload".to_string(),
            args: "--method scp linpeas.sh /tmp/".to_string(),
        }
    );
}
