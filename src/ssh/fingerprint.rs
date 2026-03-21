/// Parser for SSH fingerprint information from ssh -v output and known_hosts.
pub struct FingerprintParser;

impl FingerprintParser {
    /// Parse a single line of ssh -v stderr output.
    /// Looks for lines like: "debug1: Server host key: ssh-ed25519 SHA256:xxx"
    /// Returns the fingerprint (e.g. "SHA256:xxx") if found.
    pub fn parse_line(line: &str) -> Option<String> {
        // Match lines containing "Server host key:"
        let marker = "Server host key:";
        let pos = line.find(marker)?;
        let after = &line[pos + marker.len()..].trim();
        // Format is: <key-type> <fingerprint>
        // We want the fingerprint which starts with "SHA256:" or "MD5:"
        let mut parts = after.split_whitespace();
        // skip key type
        let _key_type = parts.next()?;
        let fingerprint = parts.next()?;
        if fingerprint.starts_with("SHA256:") || fingerprint.starts_with("MD5:") {
            Some(fingerprint.to_string())
        } else {
            None
        }
    }

    /// Scan multi-line stderr output for a fingerprint line.
    /// Returns the first fingerprint found, or None.
    pub fn parse_from_output(output: &str) -> Option<String> {
        for line in output.lines() {
            if let Some(fp) = Self::parse_line(line) {
                return Some(fp);
            }
        }
        None
    }

    /// Returns the command to look up a host in known_hosts.
    pub fn known_hosts_lookup_command(host: &str) -> Vec<&str> {
        vec!["ssh-keygen", "-F", host]
    }

    /// Parse the output of `ssh-keygen -F <host>` to extract a fingerprint.
    /// The output typically contains lines like:
    ///   # Host 10.10.10.5 found: line 1
    ///   10.10.10.5 ssh-ed25519 AAAA...
    /// We look for a SHA256 token in the output.
    pub fn parse_known_hosts_output(output: &str) -> Option<String> {
        for line in output.lines() {
            if line.starts_with('#') {
                continue;
            }
            for token in line.split_whitespace() {
                if token.starts_with("SHA256:") || token.starts_with("MD5:") {
                    return Some(token.to_string());
                }
            }
        }
        None
    }
}
