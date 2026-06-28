//! SSRF URL policy for browser navigation.
//!
//! `UrlPolicy` is a lightweight, pure-function guard that classifies a
//! navigation target as **allowed** or **blocked** before it is handed to a
//! browser driver. Every check operates on the URL's scheme and host literal
//! (after parsing); there are no DNS lookups, so DNS-rebind attacks are out of
//! scope and documented as future work.
//!
//! ## Modes
//!
//! | Constructor | Behaviour |
//! |---|---|
//! | `UrlPolicy::allow_all()` | Passes every URL through unchanged (backwards-compatible default). |
//! | `UrlPolicy::block_private()` | Blocks non-`http`/`https` schemes, loopback, RFC 1918 private, link-local (169.254.x.x / fe80::), and cloud-metadata addresses. |
//!
//! ## Wiring
//!
//! The policy type lives in the core `beater-browser` crate so every driver
//! backend (`beater-browser-cdp`, `beater-browser-playwright`,
//! `beater-browser-webdriver`) can import it without pulling in store or API
//! dependencies. **All live drivers enforce the policy** at the start of every
//! navigation entry point (`goto` and `act(Goto)`) via `policy.enforce(url)?`,
//! before issuing the real CDP/WebDriver/Playwright navigate command. The live
//! drivers default to [`UrlPolicy::block_private`] (secure by default); a caller
//! that must reach a trusted internal/loopback target opts in via each driver's
//! `with_policy(UrlPolicy::allow_all())` builder.
//!
//! `MockDriver` accepts an optional `UrlPolicy` via
//! [`crate::MockDriver::with_policy`] so tests can exercise policy enforcement
//! without a real browser.
//!
//! ## Future work
//!
//! - DNS-rebind mitigation: resolve the hostname and re-check the resulting IP
//!   after navigation, or use a DNS-over-HTTPS resolver before launch.
//! - Per-tenant allowlist: extend `UrlPolicy` with an explicit `Vec<String>`
//!   allowlist of domains that bypass the block list.
//! - CIDR-based allowlist: allow callers to opt specific private ranges back in
//!   (e.g. an on-prem app running on 192.168.x.x).

use std::net::IpAddr;

use crate::BrowserError;

/// Decides whether a navigation target URL may be visited.
///
/// Construct with [`UrlPolicy::allow_all`] (default, back-compat) or
/// [`UrlPolicy::block_private`] (secure default for production use).
#[derive(Clone, Debug)]
pub struct UrlPolicy {
    mode: PolicyMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PolicyMode {
    /// Every URL is permitted — legacy / test back-compat mode.
    AllowAll,
    /// Block non-http(s) schemes + private/loopback/link-local/metadata hosts.
    BlockPrivate,
}

/// The outcome of a policy check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyVerdict {
    /// The URL may be navigated to.
    Allow,
    /// The URL must not be navigated to; the reason string is human-readable.
    Block(String),
}

impl UrlPolicy {
    /// Permit any URL. Use for backwards compatibility or trusted callers.
    pub fn allow_all() -> Self {
        Self {
            mode: PolicyMode::AllowAll,
        }
    }

    /// Block private/loopback/link-local/metadata addresses and non-http(s)
    /// schemes. This is the recommended constructor for production drivers.
    pub fn block_private() -> Self {
        Self {
            mode: PolicyMode::BlockPrivate,
        }
    }

    /// Evaluate `url` against the policy.
    ///
    /// Returns `PolicyVerdict::Allow` when the URL is safe to visit, or
    /// `PolicyVerdict::Block(reason)` when it must be rejected.
    ///
    /// A URL that cannot be parsed is always blocked (fail-closed).
    pub fn check(&self, url: &str) -> PolicyVerdict {
        if self.mode == PolicyMode::AllowAll {
            return PolicyVerdict::Allow;
        }
        // --- scheme check ---
        let Some((scheme, rest)) = url.split_once("://") else {
            return PolicyVerdict::Block(format!(
                "rejected: URL has no scheme (unparseable): {url}"
            ));
        };
        let scheme = scheme.to_ascii_lowercase();
        if scheme != "http" && scheme != "https" {
            return PolicyVerdict::Block(format!(
                "rejected: scheme '{scheme}' is not http or https"
            ));
        }

        // --- extract host (strip userinfo, port, path, query, fragment) ---
        // rest = [userinfo@]host[:port][/path][?query][#frag]
        let without_userinfo = match rest.split_once('@') {
            Some((_, after)) => after,
            None => rest,
        };
        // host may be IPv6 literal in brackets: [::1]:8080/path
        // Zone IDs are encoded as `%25<zone>` inside the brackets (RFC 6874).
        // Strip the zone-id before parsing so `fe80::1%25eth0` → `fe80::1`.
        let host_raw = if without_userinfo.starts_with('[') {
            // IPv6 bracketed literal
            match without_userinfo.split_once(']') {
                Some((bracketed, _)) => {
                    // bracketed = "[::1" or "[fe80::1%25eth0" → strip leading '['
                    let inner = &bracketed[1..];
                    // Strip URL-encoded zone-id (%25...) or raw zone-id (%...)
                    match inner.split_once("%25") {
                        Some((addr, _)) => addr,
                        None => match inner.split_once('%') {
                            Some((addr, _)) => addr,
                            None => inner,
                        },
                    }
                }
                None => {
                    return PolicyVerdict::Block(format!(
                        "rejected: malformed IPv6 literal in URL: {url}"
                    ));
                }
            }
        } else {
            // IPv4 or hostname: strip port and path
            let host_and_rest = match without_userinfo.split_once('/') {
                Some((h, _)) => h,
                None => without_userinfo,
            };
            match host_and_rest.split_once(':') {
                Some((h, _)) => h,
                None => host_and_rest,
            }
        };

        // Also strip query/fragment if they snuck through
        let host = host_raw
            .split_once('?')
            .map(|(h, _)| h)
            .unwrap_or(host_raw)
            .split_once('#')
            .map(|(h, _)| h)
            .unwrap_or(host_raw)
            .trim();

        if host.is_empty() {
            return PolicyVerdict::Block(format!("rejected: empty host in URL: {url}"));
        }

        // --- hostname checks ---
        let host_lower = host.to_ascii_lowercase();
        if host_lower == "localhost" || host_lower.ends_with(".localhost") {
            return PolicyVerdict::Block(format!(
                "rejected: 'localhost' hostname resolves to loopback: {url}"
            ));
        }

        // --- IP address checks ---
        //
        // First try the strict `IpAddr` parser (canonical dotted-quad IPv4 and
        // every IPv6 form). If that fails, fall back to the browser-style
        // "relaxed" IPv4 parser, which canonicalizes alternate encodings that a
        // real renderer would accept but `IpAddr` rejects — decimal
        // (`2130706433`), hex (`0x7f.0.0.1`), octal (`0177.0.0.1`), and
        // short-forms (`127.1`). Without this, those bypass the guard by
        // falling through as opaque "hostnames".
        let ip = host
            .parse::<IpAddr>()
            .ok()
            .or_else(|| parse_relaxed_ipv4(host).map(IpAddr::V4));
        if let Some(ip) = ip {
            if let Some(reason) = check_ip_blocked(&ip) {
                return PolicyVerdict::Block(format!("rejected: {reason}: {url}"));
            }
        }

        PolicyVerdict::Allow
    }

    /// Convenience wrapper: returns `Err(BrowserError::SsrfBlocked)` when
    /// `check` would block, or `Ok(())` when the URL is allowed. Suitable for
    /// use inside a driver's `goto` implementation via `policy.enforce(url)?`.
    pub fn enforce(&self, url: &str) -> Result<(), BrowserError> {
        match self.check(url) {
            PolicyVerdict::Allow => Ok(()),
            PolicyVerdict::Block(reason) => Err(BrowserError::SsrfBlocked(reason)),
        }
    }
}

/// Parse the "relaxed" / alternate textual forms of an IPv4 address that web
/// browsers accept but [`std::net::Ipv4Addr`]'s `FromStr` rejects. Returns
/// `None` when `host` is not a numeric IPv4 representation (i.e. it is a real
/// hostname or an IPv6 literal), so the caller falls through to ordinary
/// hostname handling.
///
/// Implements the WHATWG URL "IPv4 number parser": the host is split into
/// 1–4 dot-separated parts, each parsed with radix auto-detection (`0x`/`0X`
/// → hex, leading `0` → octal, otherwise decimal). All but the final part are
/// single octets (`< 256`); the final part absorbs the remaining low-order
/// bytes. This canonicalizes:
///
/// - decimal:    `2130706433`   → `127.0.0.1`
/// - hex:        `0x7f.0.0.1`   → `127.0.0.1`, `0x7f000001` → `127.0.0.1`
/// - octal:      `0177.0.0.1`   → `127.0.0.1`
/// - short-form: `127.1`        → `127.0.0.1`, `10.1` → `10.0.0.1`
fn parse_relaxed_ipv4(host: &str) -> Option<std::net::Ipv4Addr> {
    let mut parts: Vec<&str> = host.split('.').collect();
    // Tolerate a single trailing dot ("127.0.0.1.") as browsers do.
    if parts.len() > 1 && parts.last() == Some(&"") {
        parts.pop();
    }
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }

    let mut nums: Vec<u64> = Vec::with_capacity(parts.len());
    for part in &parts {
        nums.push(parse_ipv4_part(part)?);
    }

    let n = nums.len();
    // Every part except the last is a single octet and must fit in one byte.
    if nums[..n - 1].iter().any(|&v| v > 0xff) {
        return None;
    }
    // The final part fills the remaining `4 - (n - 1)` low-order bytes.
    let remaining_bytes = (4 - (n - 1)) as u32;
    let max_last: u64 = (1u64 << (8 * remaining_bytes)) - 1;
    let last = nums[n - 1];
    if last > max_last {
        return None;
    }

    let mut addr: u32 = 0;
    for (i, &v) in nums[..n - 1].iter().enumerate() {
        addr |= (v as u32) << (8 * (3 - i as u32));
    }
    addr |= last as u32;
    Some(std::net::Ipv4Addr::from(addr))
}

/// Parse one dot-separated part of a relaxed IPv4 literal with radix
/// auto-detection. Returns `None` if the part is not a valid number in its
/// detected radix (so the whole host is treated as a hostname).
fn parse_ipv4_part(part: &str) -> Option<u64> {
    if part.is_empty() {
        return None;
    }
    let (radix, digits) =
        if let Some(rest) = part.strip_prefix("0x").or_else(|| part.strip_prefix("0X")) {
            (16u32, rest)
        } else if part.len() > 1 && part.starts_with('0') {
            (8u32, &part[1..])
        } else {
            (10u32, part)
        };
    // "0", "0x" and "00" all denote zero.
    if digits.is_empty() {
        return Some(0);
    }
    u64::from_str_radix(digits, radix).ok()
}

/// Returns `Some(reason)` if the given IP address is in a blocked range, or
/// `None` if it is a routable public address.
fn check_ip_blocked(ip: &IpAddr) -> Option<String> {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            // 127.0.0.0/8 — loopback
            if octets[0] == 127 {
                return Some(format!("{ip} is a loopback address (127.0.0.0/8)"));
            }
            // 10.0.0.0/8 — RFC 1918 private
            if octets[0] == 10 {
                return Some(format!("{ip} is in RFC 1918 private range (10.0.0.0/8)"));
            }
            // 172.16.0.0/12 — RFC 1918 private (172.16.0.0 – 172.31.255.255)
            if octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31) {
                return Some(format!("{ip} is in RFC 1918 private range (172.16.0.0/12)"));
            }
            // 192.168.0.0/16 — RFC 1918 private
            if octets[0] == 192 && octets[1] == 168 {
                return Some(format!(
                    "{ip} is in RFC 1918 private range (192.168.0.0/16)"
                ));
            }
            // 169.254.0.0/16 — link-local / cloud metadata (AWS IMDSv1, GCP, etc.)
            if octets[0] == 169 && octets[1] == 254 {
                return Some(format!(
                    "{ip} is a link-local / cloud-metadata address (169.254.0.0/16)"
                ));
            }
            // 0.0.0.0/8 — "this" network (unspecified)
            if octets[0] == 0 {
                return Some(format!("{ip} is an unspecified address (0.0.0.0/8)"));
            }
            None
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            // ::1 — loopback
            if *v6 == std::net::Ipv6Addr::LOCALHOST {
                return Some(format!("{ip} is the IPv6 loopback address (::1)"));
            }
            // :: — unspecified
            if *v6 == std::net::Ipv6Addr::UNSPECIFIED {
                return Some(format!("{ip} is the IPv6 unspecified address (::)"));
            }
            // fe80::/10 — link-local (first 10 bits = 1111 1110 10)
            // segments[0] in range [0xfe80, 0xfebf]
            if (segments[0] & 0xffc0) == 0xfe80 {
                return Some(format!("{ip} is an IPv6 link-local address (fe80::/10)"));
            }
            // fc00::/7 — unique-local (RFC 4193, analogous to RFC 1918)
            // segments[0] in range [0xfc00, 0xfdff]
            if (segments[0] & 0xfe00) == 0xfc00 {
                return Some(format!("{ip} is an IPv6 unique-local address (fc00::/7)"));
            }
            // ::ffff:0:0/96 — IPv4-mapped IPv6; re-check the embedded IPv4
            if segments[0] == 0
                && segments[1] == 0
                && segments[2] == 0
                && segments[3] == 0
                && segments[4] == 0
                && segments[5] == 0xffff
            {
                let embedded = v6.to_ipv4_mapped().unwrap_or_else(|| {
                    // Safety: we checked the mapping prefix above; this branch
                    // is unreachable in practice, but avoids unwrap.
                    std::net::Ipv4Addr::new(0, 0, 0, 0)
                });
                if let Some(reason) = check_ip_blocked(&IpAddr::V4(embedded)) {
                    return Some(format!(
                        "{ip} is an IPv4-mapped IPv6 address whose embedded IPv4 is blocked: {reason}"
                    ));
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helper ──────────────────────────────────────────────────────────────

    fn allows(policy: &UrlPolicy, url: &str) {
        assert_eq!(
            policy.check(url),
            PolicyVerdict::Allow,
            "expected ALLOW for {url:?}"
        );
    }

    fn blocks(policy: &UrlPolicy, url: &str) {
        assert!(
            matches!(policy.check(url), PolicyVerdict::Block(_)),
            "expected BLOCK for {url:?}"
        );
    }

    // ── allow_all ───────────────────────────────────────────────────────────

    #[test]
    fn allow_all_passes_everything() {
        let p = UrlPolicy::allow_all();
        // Public HTTPS
        allows(&p, "https://example.com/path?q=1");
        // Loopback — still allowed under allow_all
        allows(&p, "http://127.0.0.1");
        allows(&p, "http://localhost");
        allows(&p, "http://10.0.0.1");
        // Metadata endpoint
        allows(&p, "http://169.254.169.254/latest/meta-data/");
        // Non-http schemes — still allowed under allow_all
        allows(&p, "file:///etc/passwd");
        allows(&p, "gopher://evil.com");
        // Malformed — even unparseable URLs are allowed under allow_all
        allows(&p, "not-a-url");
    }

    // ── block_private: public URLs (allowed) ────────────────────────────────

    #[test]
    fn block_private_allows_public_https() {
        let p = UrlPolicy::block_private();
        allows(&p, "https://example.com");
        allows(&p, "https://example.com/path?q=1#anchor");
        allows(&p, "http://example.com");
        allows(&p, "https://sub.domain.example.com");
    }

    #[test]
    fn block_private_allows_public_ipv4() {
        let p = UrlPolicy::block_private();
        // Publicly-routable IPs
        allows(&p, "https://8.8.8.8"); // Google DNS
        allows(&p, "https://1.1.1.1"); // Cloudflare
        allows(&p, "https://203.0.113.5"); // TEST-NET-3 (documentation range, public)
        allows(&p, "https://198.51.100.1"); // TEST-NET-2 (documentation range, public)
    }

    #[test]
    fn block_private_allows_non_reserved_port() {
        let p = UrlPolicy::block_private();
        allows(&p, "https://example.com:8443/api");
        allows(&p, "http://example.com:3000/");
    }

    // ── block_private: loopback (blocked) ───────────────────────────────────

    #[test]
    fn block_private_blocks_ipv4_loopback() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://127.0.0.1");
        blocks(&p, "http://127.0.0.1:8080/path");
        blocks(&p, "https://127.0.0.1");
        // Entire 127.0.0.0/8
        blocks(&p, "http://127.1.2.3");
        blocks(&p, "http://127.255.255.255");
    }

    #[test]
    fn block_private_blocks_localhost_hostname() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://localhost");
        blocks(&p, "http://localhost:3000");
        blocks(&p, "https://localhost/admin");
        // Subdomains of localhost
        blocks(&p, "http://api.localhost");
        blocks(&p, "http://db.localhost:5432");
    }

    #[test]
    fn block_private_blocks_ipv6_loopback() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://[::1]");
        blocks(&p, "http://[::1]:8080");
        blocks(&p, "https://[::1]/admin");
    }

    // ── block_private: RFC 1918 (blocked) ───────────────────────────────────

    #[test]
    fn block_private_blocks_10_0_0_0_8() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://10.0.0.1");
        blocks(&p, "http://10.10.10.10");
        blocks(&p, "https://10.255.255.255");
        blocks(&p, "http://10.0.0.1:9200/"); // Elasticsearch-style internal
    }

    #[test]
    fn block_private_blocks_172_16_0_0_12() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://172.16.0.1");
        blocks(&p, "http://172.20.0.1");
        blocks(&p, "http://172.31.255.255");
        // Edge: 172.15.x.x is NOT in the range (public)
        allows(&p, "https://172.15.0.1");
        // Edge: 172.32.x.x is NOT in the range (public)
        allows(&p, "https://172.32.0.1");
    }

    #[test]
    fn block_private_blocks_192_168_0_0_16() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://192.168.0.1");
        blocks(&p, "http://192.168.1.1");
        blocks(&p, "https://192.168.255.255");
    }

    // ── block_private: link-local / metadata (blocked) ──────────────────────

    #[test]
    fn block_private_blocks_169_254_link_local_and_metadata() {
        let p = UrlPolicy::block_private();
        // AWS/GCP/Azure IMDS endpoint
        blocks(&p, "http://169.254.169.254");
        blocks(
            &p,
            "http://169.254.169.254/latest/meta-data/iam/security-credentials/",
        );
        // Generic 169.254.x.x
        blocks(&p, "http://169.254.0.1");
        blocks(&p, "http://169.254.255.255");
    }

    #[test]
    fn block_private_blocks_ipv6_link_local() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://[fe80::1]");
        blocks(&p, "http://[fe80::1%25eth0]"); // zone-id stripped during host parse
        blocks(&p, "https://[fe80::dead:beef]");
        // fe80::/10 covers up to febf::
        blocks(&p, "http://[fea0::1]");
        blocks(&p, "http://[febf::1]");
        // ff02:: is multicast, not link-local — should be allowed
        // (not in blocked ranges we cover)
    }

    #[test]
    fn block_private_blocks_ipv6_unique_local() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://[fc00::1]");
        blocks(&p, "http://[fd12:3456:789a::1]");
    }

    // ── block_private: non-http(s) schemes (blocked) ────────────────────────

    #[test]
    fn block_private_blocks_file_scheme() {
        let p = UrlPolicy::block_private();
        blocks(&p, "file:///etc/passwd");
        blocks(&p, "file:///C:/Windows/System32/drivers/etc/hosts");
        blocks(&p, "file://localhost/etc/shadow");
    }

    #[test]
    fn block_private_blocks_gopher_and_other_schemes() {
        let p = UrlPolicy::block_private();
        blocks(&p, "gopher://evil.com");
        blocks(&p, "ftp://ftp.example.com/pub");
        blocks(&p, "data:text/html,<h1>xss</h1>");
        blocks(&p, "javascript:alert(1)");
        blocks(&p, "dict://127.0.0.1:11111/");
    }

    // ── block_private: parse failures (blocked, fail-closed) ────────────────

    #[test]
    fn block_private_blocks_unparseable_urls() {
        let p = UrlPolicy::block_private();
        // No scheme at all
        blocks(&p, "not-a-url");
        blocks(&p, "example.com/path");
        blocks(&p, "");
        // Scheme but no host
        blocks(&p, "http://");
    }

    // ── enforce() wrapper ───────────────────────────────────────────────────

    #[test]
    fn enforce_returns_ok_for_allowed_url() {
        let p = UrlPolicy::block_private();
        assert!(p.enforce("https://example.com").is_ok());
    }

    #[test]
    fn enforce_returns_ssrf_blocked_error_for_blocked_url() {
        let p = UrlPolicy::block_private();
        let Err(err) = p.enforce("http://127.0.0.1") else {
            panic!("expected enforce to block loopback");
        };
        assert!(
            matches!(err, BrowserError::SsrfBlocked(_)),
            "expected SsrfBlocked, got: {err:?}"
        );
        // Error message should carry the reason
        let msg = err.to_string();
        assert!(
            msg.contains("127.0.0.1"),
            "error message should mention the blocked address, got: {msg}"
        );
    }

    #[test]
    fn enforce_returns_ssrf_blocked_for_metadata_endpoint() {
        let p = UrlPolicy::block_private();
        let Err(err) = p.enforce("http://169.254.169.254/latest/meta-data/") else {
            panic!("expected enforce to block metadata endpoint");
        };
        assert!(matches!(err, BrowserError::SsrfBlocked(_)));
    }

    // ── IPv4-mapped IPv6 (blocked) ───────────────────────────────────────────

    #[test]
    fn block_private_blocks_ipv4_mapped_ipv6_loopback() {
        let p = UrlPolicy::block_private();
        // ::ffff:127.0.0.1 — IPv4-mapped IPv6 loopback
        blocks(&p, "http://[::ffff:127.0.0.1]");
        blocks(&p, "http://[::ffff:7f00:1]"); // same in hex
    }

    #[test]
    fn block_private_blocks_ipv4_mapped_ipv6_private() {
        let p = UrlPolicy::block_private();
        // ::ffff:10.0.0.1
        blocks(&p, "http://[::ffff:10.0.0.1]");
        // ::ffff:192.168.1.1
        blocks(&p, "http://[::ffff:192.168.1.1]");
        // ::ffff:169.254.169.254
        blocks(&p, "http://[::ffff:169.254.169.254]");
    }

    // ── unspecified addresses (blocked) ─────────────────────────────────────

    #[test]
    fn block_private_blocks_unspecified_addresses() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://0.0.0.0");
        blocks(&p, "http://[::]/");
    }

    // ── alternate IPv4 encodings (blocked) ──────────────────────────────────

    /// Decimal, octal, hex and short-form encodings of loopback must all be
    /// canonicalized and blocked — these previously bypassed the guard by
    /// falling through as opaque hostnames.
    #[test]
    fn block_private_blocks_alternate_encodings_of_loopback() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://2130706433"); // decimal 127.0.0.1
        blocks(&p, "http://2130706433/latest/meta-data/"); // with path
        blocks(&p, "http://0177.0.0.1"); // octal first octet
        blocks(&p, "http://017700000001"); // octal whole address
        blocks(&p, "http://0x7f.0.0.1"); // hex first octet
        blocks(&p, "http://0x7f000001"); // hex whole address
        blocks(&p, "http://127.1"); // short-form -> 127.0.0.1
        blocks(&p, "http://127.0.1"); // 3-part short-form
        blocks(&p, "http://127.1:8080/admin"); // short-form + port + path
    }

    /// Alternate encodings of RFC 1918 private and cloud-metadata addresses.
    #[test]
    fn block_private_blocks_alternate_encodings_of_private_and_metadata() {
        let p = UrlPolicy::block_private();
        // 169.254.169.254 (IMDS) as decimal and hex.
        blocks(&p, "http://2852039166");
        blocks(&p, "http://0xA9FEA9FE");
        blocks(&p, "http://0xa9fea9fe");
        // 10.0.0.1 short-form / hex.
        blocks(&p, "http://10.1");
        blocks(&p, "http://0xa.0.0.1");
        // 192.168.0.1 decimal.
        blocks(&p, "http://3232235521");
    }

    /// The relaxed parser must not over-block: a numeric encoding of a *public*
    /// IP and ordinary hostnames (including ones with numeric labels) stay
    /// allowed.
    #[test]
    fn block_private_relaxed_parser_allows_public_and_hostnames() {
        let p = UrlPolicy::block_private();
        allows(&p, "http://134744072"); // 8.8.8.8 decimal -> public
        allows(&p, "http://1.1"); // 1.0.0.1 -> public short-form
        allows(&p, "https://example.com");
        allows(&p, "https://1.example.com"); // numeric label, still a hostname
        allows(&p, "https://192-0-2-1.example.com"); // not dotted, hostname
    }

    // ── case-insensitive scheme / hostname ──────────────────────────────────

    #[test]
    fn block_private_is_case_insensitive_for_scheme() {
        let p = UrlPolicy::block_private();
        // Uppercase scheme must still be blocked
        blocks(&p, "FILE:///etc/passwd");
        blocks(&p, "FTP://ftp.example.com");
        // HTTPS (uppercase) is allowed
        allows(&p, "HTTPS://example.com");
        allows(&p, "HTTP://example.com");
    }

    #[test]
    fn block_private_is_case_insensitive_for_localhost() {
        let p = UrlPolicy::block_private();
        blocks(&p, "http://LOCALHOST");
        blocks(&p, "http://Localhost:8080");
        blocks(&p, "http://LOCALHOST/admin");
    }
}
