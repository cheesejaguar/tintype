//! The public demo mode of `tintype serve --demo`: one photo per request,
//! no token, and rate limits that cannot be turned off.
//!
//! The limits live here and nowhere else in the code. The website, the
//! terms of use and the API documentation state them too; a test checks
//! that those pages say exactly what these constants enforce.
//!
//! Privacy: the limiter keys callers by a keyed hash of their network
//! address, with a key drawn at random when the process starts and never
//! written anywhere. Nothing here logs, stores or returns an address.

use anyhow::{ensure, Result};
use axum::http::{HeaderMap, HeaderValue};
use std::collections::{hash_map::RandomState, HashMap};
use std::hash::BuildHasher;
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;

/// What the demo allows. One set, compiled in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Photos per network in one short window.
    pub short: u32,
    /// The short window, in seconds. It starts with the first photo in it.
    pub short_window_s: u64,
    /// Photos per network per UTC day.
    pub daily_per_ip: u32,
    /// Photos across every caller per UTC day.
    pub daily_total: u32,
    /// The largest photo, in bytes.
    pub max_bytes: usize,
}

/// The demo's limits. Change them here, then update the site, the terms and
/// `docs/API.md`; `the_pages_state_the_limits_the_server_enforces` fails
/// until all three agree.
pub const LIMITS: Limits = Limits {
    short: 5,
    short_window_s: 600,
    daily_per_ip: 20,
    daily_total: 2_000,
    max_bytes: 10_000_000,
};

/// Where a visitor goes when the demo is at capacity.
pub const INSTALL_URL: &str = "https://cheesejaguar.github.io/tintype/#install";

const DAY: u64 = 86_400;

impl Limits {
    /// A demo without every limit in force does not start.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.short > 0 && self.daily_per_ip > 0 && self.daily_total > 0 && self.max_bytes > 0,
            "demo mode needs every rate limit and the size cap set above zero"
        );
        ensure!(
            (1..=DAY).contains(&self.short_window_s),
            "the demo's short window must be between one second and one day"
        );
        ensure!(
            self.short <= self.daily_per_ip && self.daily_per_ip <= self.daily_total,
            "the demo's limits must nest: short window, then per network per day, then in total per day"
        );
        Ok(())
    }

    /// The sentences the site, the terms and the API docs must contain:
    /// what one visitor may send.
    pub fn statements(&self) -> Vec<String> {
        vec![
            format!("{} photos per {} minutes", self.short, self.short_window_s / 60),
            format!("{} photos per day", self.daily_per_ip),
            format!("{} MB", self.max_bytes / 1_000_000),
        ]
    }

    /// The daily capacity, stated in the API docs. The website does not
    /// state it; it explains it when it is reached.
    pub fn capacity_statement(&self) -> String {
        format!("{} photos per day across all visitors", thousands(self.daily_total))
    }
}

fn thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Which limit a request ran into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    IpShort,
    IpDaily,
    GlobalDaily,
}

impl Limit {
    pub fn code(self) -> &'static str {
        match self {
            Limit::IpShort => "ip_short",
            Limit::IpDaily => "ip_daily",
            Limit::GlobalDaily => "global_daily",
        }
    }

    /// The policy name in the `RateLimit` and `RateLimit-Policy` fields.
    fn policy(self) -> &'static str {
        match self {
            Limit::IpShort => "ip-short",
            Limit::IpDaily => "ip-daily",
            Limit::GlobalDaily => "global-daily",
        }
    }
}

/// Where a caller stands against one limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quota {
    pub limit: Limit,
    pub quota: u32,
    pub window_s: u64,
    pub remaining: u32,
    /// Seconds until this limit's count starts again.
    pub reset_s: u64,
}

/// Where a caller stands against every limit, at one moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub quotas: [Quota; 3],
    pub now: u64,
}

impl Status {
    /// The exhausted limit with the longest wait, if any. On a tie the
    /// global cap wins, so the page can say the demo is at capacity.
    pub fn blocked(&self) -> Option<Quota> {
        self.quotas
            .iter()
            .filter(|q| q.remaining == 0)
            .max_by_key(|q| (q.reset_s, q.limit == Limit::GlobalDaily, q.limit == Limit::IpDaily))
            .copied()
    }

    /// The limit closest to running out: the fewest remaining, then the
    /// longest wait. The single-limit headers describe this one.
    fn tightest(&self) -> Quota {
        *self
            .quotas
            .iter()
            .min_by_key(|q| (q.remaining, std::cmp::Reverse(q.reset_s)))
            .unwrap()
    }

    /// The standard rate-limit fields: `RateLimit-Policy` and `RateLimit`
    /// from the IETF httpapi draft, one item per limit, and the older
    /// single-limit `RateLimit-Limit`, `-Remaining` and `-Reset` fields,
    /// which describe the limit closest to running out.
    pub fn headers(&self) -> Vec<(&'static str, String)> {
        let join = |f: &dyn Fn(&Quota) -> String| {
            self.quotas.iter().map(f).collect::<Vec<_>>().join(", ")
        };
        let tightest = self.tightest();
        vec![
            (
                "ratelimit-policy",
                join(&|q| format!("\"{}\";q={};w={}", q.limit.policy(), q.quota, q.window_s)),
            ),
            (
                "ratelimit",
                join(&|q| format!("\"{}\";r={};t={}", q.limit.policy(), q.remaining, q.reset_s)),
            ),
            ("ratelimit-limit", tightest.quota.to_string()),
            ("ratelimit-remaining", tightest.remaining.to_string()),
            ("ratelimit-reset", tightest.reset_s.to_string()),
        ]
    }

    /// Seconds to wait, for `Retry-After` on a refusal.
    pub fn retry_after(&self) -> Option<u64> {
        self.blocked().map(|q| q.reset_s.max(1))
    }

    /// The JSON body of a 429: which limit, a plain sentence, and when.
    pub fn refusal(&self) -> Option<serde_json::Value> {
        let blocked = self.blocked()?;
        let wait = blocked.reset_s.max(1);
        let message = match blocked.limit {
            Limit::IpShort => format!(
                "You have used the {} photos allowed per {} minutes from your network. Try again in {}, at {}.",
                blocked.quota,
                blocked.window_s / 60,
                duration(wait),
                clock(self.now + wait)
            ),
            Limit::IpDaily => format!(
                "You have used the {} photos allowed per day from your network. The allowance starts again at midnight UTC, in {}.",
                blocked.quota,
                duration(wait)
            ),
            Limit::GlobalDaily => format!(
                "The demo is at capacity for today. To stay within its hosting budget, it scores a fixed number of photos per day across all visitors, and today's are used up. It starts again at midnight UTC, in {}. You can install Tintype and run it on your own computer without limits: {INSTALL_URL}",
                duration(wait)
            ),
        };
        Some(serde_json::json!({"error": {
            "code": "rate_limited",
            "limit": blocked.limit.code(),
            "message": message,
            "retry_after_s": wait,
            "reset_at": timestamp(self.now + wait),
        }}))
    }
}

/// "7 minutes", "5 hours 12 minutes": rounded up to the minute.
fn duration(seconds: u64) -> String {
    let minutes = seconds.div_ceil(60).max(1);
    let plural = |n: u64, unit: &str| format!("{n} {unit}{}", if n == 1 { "" } else { "s" });
    match (minutes / 60, minutes % 60) {
        (0, m) => plural(m, "minute"),
        (h, 0) => plural(h, "hour"),
        (h, m) => format!("{} {}", plural(h, "hour"), plural(m, "minute")),
    }
}

/// `HH:MM UTC`.
fn clock(unix: u64) -> String {
    let of_day = unix % DAY;
    format!("{:02}:{:02} UTC", of_day / 3600, of_day % 3600 / 60)
}

/// RFC 3339 in UTC, from Unix seconds.
pub fn timestamp(unix: u64) -> String {
    // Howard Hinnant's days-to-civil.
    let days = (unix / DAY) as i64;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    let of_day = unix % DAY;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        of_day / 3600,
        of_day % 3600 / 60,
        of_day % 60
    )
}

/// A caller's place in the limiter: a keyed hash of its network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key(u64);

#[derive(Default)]
struct Client {
    short_start: u64,
    short_count: u32,
    day_count: u32,
}

struct Counts {
    day: u64,
    total: u32,
    clients: HashMap<Key, Client>,
}

/// The demo's three limits, held in memory only. Counts start again when
/// the process restarts, so the instance cap of the deployment is the hard
/// ceiling on cost; these limits share it out fairly.
pub struct Limiter {
    limits: Limits,
    salt: RandomState,
    counts: Mutex<Counts>,
}

impl Limiter {
    pub fn new(limits: Limits) -> Result<Self> {
        limits.validate()?;
        Ok(Self {
            limits,
            salt: RandomState::new(),
            counts: Mutex::new(Counts { day: 0, total: 0, clients: HashMap::new() }),
        })
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// The key of an address: IPv4 as is, IPv6 by its /64 (one household
    /// or host usually holds a whole /64), hashed with the process's key.
    pub fn key(&self, ip: IpAddr) -> Key {
        let network = match ip {
            IpAddr::V4(v4) => u128::from(u32::from(v4)),
            IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
                Some(v4) => u128::from(u32::from(v4)),
                None => (u128::from(v6) >> 64 << 64) | 1 << 32,
            },
        };
        Key(self.salt.hash_one(network))
    }

    /// Where `key` stands, without counting anything.
    pub fn status(&self, key: Key, now: u64) -> Status {
        let mut counts = self.counts.lock().unwrap();
        self.roll(&mut counts, now);
        self.describe(&counts, key, now)
    }

    /// Count one photo for `key`, or refuse it with the status that says why.
    pub fn take(&self, key: Key, now: u64) -> std::result::Result<Status, Status> {
        let mut counts = self.counts.lock().unwrap();
        self.roll(&mut counts, now);
        let status = self.describe(&counts, key, now);
        if status.blocked().is_some() {
            return Err(status);
        }
        counts.total += 1;
        let client = counts.clients.entry(key).or_default();
        if client.short_count == 0 || now >= client.short_start + self.limits.short_window_s {
            client.short_start = now;
            client.short_count = 0;
        }
        client.short_count += 1;
        client.day_count += 1;
        Ok(self.describe(&counts, key, now))
    }

    /// Forget what a new UTC day no longer needs, even when nobody calls.
    /// The server runs this every minute.
    pub fn expire(&self, now: u64) {
        let mut counts = self.counts.lock().unwrap();
        self.roll(&mut counts, now);
    }

    /// How many networks the limiter holds: for tests and the hourly summary.
    pub fn tracked(&self) -> usize {
        self.counts.lock().unwrap().clients.len()
    }

    /// A new UTC day: daily counts start again, and every network whose
    /// short window has also ended is forgotten. Within a day, entries only
    /// come from counted photos, so there are at most `daily_total` of them.
    fn roll(&self, counts: &mut Counts, now: u64) {
        let day = now / DAY;
        if day == counts.day {
            return;
        }
        counts.day = day;
        counts.total = 0;
        let window = self.limits.short_window_s;
        counts.clients.retain(|_, c| now < c.short_start + window);
        for client in counts.clients.values_mut() {
            client.day_count = 0;
        }
    }

    fn describe(&self, counts: &Counts, key: Key, now: u64) -> Status {
        let l = &self.limits;
        let midnight = DAY - now % DAY;
        let client = counts.clients.get(&key);
        let (short_used, short_reset) = match client {
            Some(c) if c.short_count > 0 && now < c.short_start + l.short_window_s => {
                (c.short_count, c.short_start + l.short_window_s - now)
            }
            _ => (0, l.short_window_s),
        };
        let day_used = client.map_or(0, |c| c.day_count);
        Status {
            quotas: [
                Quota {
                    limit: Limit::IpShort,
                    quota: l.short,
                    window_s: l.short_window_s,
                    remaining: l.short.saturating_sub(short_used),
                    reset_s: short_reset,
                },
                Quota {
                    limit: Limit::IpDaily,
                    quota: l.daily_per_ip,
                    window_s: DAY,
                    remaining: l.daily_per_ip.saturating_sub(day_used),
                    reset_s: midnight,
                },
                Quota {
                    limit: Limit::GlobalDaily,
                    quota: l.daily_total,
                    window_s: DAY,
                    remaining: l.daily_total.saturating_sub(counts.total),
                    reset_s: midnight,
                },
            ],
            now,
        }
    }
}

/// Where the caller's address comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum ClientAddress {
    /// The TCP peer: right when nothing sits in front of the server.
    Peer,
    /// The last `X-Forwarded-For` entry, which the trusted front end in
    /// front of the server appends (Cloud Run's does). Entries before it
    /// come from the caller and are ignored, so they cannot be forged.
    LastForwardedFor,
}

impl ClientAddress {
    pub fn resolve(self, headers: &HeaderMap, peer: Option<SocketAddr>) -> Option<IpAddr> {
        match self {
            ClientAddress::Peer => peer.map(|p| p.ip()),
            ClientAddress::LastForwardedFor => {
                // Several header lines form one list, in order.
                let last = headers
                    .get_all("x-forwarded-for")
                    .iter()
                    .last()
                    .and_then(|v: &HeaderValue| v.to_str().ok())
                    .and_then(|v| v.rsplit(',').next())
                    .map(str::trim)
                    .and_then(|v| v.parse::<IpAddr>().ok());
                last.or_else(|| peer.map(|p| p.ip()))
            }
        }
    }
}

/// The demo's CORS policy: the one origin allowed to read responses.
#[derive(Clone, Debug)]
pub struct Origin(HeaderValue);

impl Origin {
    /// `https://host[:port]`, nothing more: no path, no trailing slash.
    pub fn parse(origin: &str) -> Result<Self> {
        let rest = origin
            .strip_prefix("https://")
            .or_else(|| origin.strip_prefix("http://").filter(|r| r.starts_with("localhost") || r.starts_with("127.0.0.1")));
        ensure!(
            rest.is_some_and(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b':'))),
            "--demo-origin must be an origin such as https://example.org (https, no path); plain http only for localhost"
        );
        Ok(Self(HeaderValue::from_str(origin)?))
    }

    pub fn value(&self) -> &HeaderValue {
        &self.0
    }

    /// A request may go on: it names no origin (a script, curl) or ours.
    pub fn allows(&self, headers: &HeaderMap) -> bool {
        headers.get("origin").is_none_or(|o| *o == self.0)
    }
}

/// The response headers a page reads: the rate-limit fields and `Retry-After`.
pub const EXPOSED: &str =
    "RateLimit, RateLimit-Policy, RateLimit-Limit, RateLimit-Remaining, RateLimit-Reset, Retry-After";

#[cfg(test)]
mod tests {
    use super::*;

    const START: u64 = 1_763_294_400; // 2025-11-16T12:00:00Z

    fn small() -> Limits {
        Limits { short: 2, short_window_s: 60, daily_per_ip: 3, daily_total: 4, max_bytes: 10 }
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn the_shipped_limits_are_valid_and_a_demo_without_limits_does_not_start() {
        LIMITS.validate().unwrap();
        assert!(Limiter::new(LIMITS).is_ok());
        for broken in [
            Limits { short: 0, ..LIMITS },
            Limits { daily_per_ip: 0, ..LIMITS },
            Limits { daily_total: 0, ..LIMITS },
            Limits { max_bytes: 0, ..LIMITS },
            Limits { short_window_s: 0, ..LIMITS },
            Limits { short_window_s: DAY + 1, ..LIMITS },
            Limits { short: 30, ..LIMITS },
            Limits { daily_per_ip: 5_000, ..LIMITS },
        ] {
            assert!(Limiter::new(broken).is_err(), "{broken:?}");
        }
    }

    #[test]
    fn the_short_window_limits_a_network_and_resets_from_its_first_photo() {
        let limiter = Limiter::new(small()).unwrap();
        let a = limiter.key(ip("198.51.100.7"));
        let first = limiter.take(a, START).unwrap();
        assert_eq!(first.quotas[0].remaining, 1);
        assert_eq!(first.quotas[0].reset_s, 60);
        let second = limiter.take(a, START + 20).unwrap();
        assert_eq!(second.quotas[0].remaining, 0);
        assert_eq!(second.quotas[0].reset_s, 40);
        assert!(second.blocked().is_some(), "the next one would be refused");
        let refused = limiter.take(a, START + 30).unwrap_err();
        let blocked = refused.blocked().unwrap();
        assert_eq!((blocked.limit, blocked.reset_s), (Limit::IpShort, 30));
        // Another network is not affected.
        assert!(limiter.take(limiter.key(ip("203.0.113.9")), START + 30).is_ok());
        // The window ends 60 s after its first photo.
        assert!(limiter.take(a, START + 59).is_err());
        let again = limiter.take(a, START + 60).unwrap();
        assert_eq!((again.quotas[0].remaining, again.quotas[0].reset_s), (1, 60));
    }

    #[test]
    fn the_daily_limit_per_network_holds_across_windows_and_resets_at_midnight_utc() {
        let limiter = Limiter::new(small()).unwrap();
        let a = limiter.key(ip("198.51.100.7"));
        limiter.take(a, START).unwrap();
        limiter.take(a, START + 1).unwrap();
        let third = limiter.take(a, START + 100).unwrap();
        assert_eq!(third.quotas[1].remaining, 0);
        let refused = limiter.take(a, START + 200).unwrap_err();
        let blocked = refused.blocked().unwrap();
        let midnight = DAY - START % DAY;
        assert_eq!((blocked.limit, blocked.reset_s), (Limit::IpDaily, midnight - 200));
        let next_day = START + midnight;
        assert!(limiter.take(a, next_day - 1).is_err());
        let fresh = limiter.take(a, next_day).unwrap();
        assert_eq!(fresh.quotas[1].remaining, 2);
        assert_eq!(fresh.quotas[1].reset_s, DAY);
    }

    #[test]
    fn the_global_cap_refuses_everyone_until_midnight_utc() {
        let limiter = Limiter::new(small()).unwrap();
        for (i, addr) in ["192.0.2.1", "192.0.2.2", "192.0.2.3", "192.0.2.4"].iter().enumerate() {
            let status = limiter.take(limiter.key(ip(addr)), START + i as u64).unwrap();
            assert_eq!(status.quotas[2].remaining, 3 - i as u32);
        }
        let newcomer = limiter.key(ip("192.0.2.200"));
        let refused = limiter.take(newcomer, START + 10).unwrap_err();
        let blocked = refused.blocked().unwrap();
        assert_eq!(blocked.limit, Limit::GlobalDaily);
        assert_eq!(blocked.reset_s, DAY - (START + 10) % DAY);
        // Its own allowance is untouched; the cap is what refuses it.
        assert_eq!(refused.quotas[0].remaining, 2);
        assert!(limiter.take(newcomer, START + blocked.reset_s + 10).is_ok());
    }

    #[test]
    fn when_several_limits_are_spent_the_longest_wait_is_reported_and_capacity_wins_ties() {
        let limiter = Limiter::new(Limits { short: 2, short_window_s: 60, daily_per_ip: 2, daily_total: 2, max_bytes: 1 }).unwrap();
        let a = limiter.key(ip("192.0.2.1"));
        limiter.take(a, START).unwrap();
        let spent = limiter.take(a, START).unwrap();
        // All three are at zero: daily and global share midnight; global wins.
        assert_eq!(spent.blocked().unwrap().limit, Limit::GlobalDaily);
        assert_eq!(spent.refusal().unwrap()["error"]["limit"], "global_daily");
    }

    #[test]
    fn requests_are_forgotten_after_their_day_and_window() {
        let limiter = Limiter::new(small()).unwrap();
        limiter.take(limiter.key(ip("192.0.2.1")), START).unwrap();
        let late = START + DAY - START % DAY - 10; // 23:59:50
        limiter.take(limiter.key(ip("192.0.2.2")), late).unwrap();
        assert_eq!(limiter.tracked(), 2);
        // Just after midnight: the first is gone, the second's window is still open.
        limiter.status(limiter.key(ip("192.0.2.3")), late + 20);
        assert_eq!(limiter.tracked(), 1);
        limiter.expire(late + 20 + DAY);
        assert_eq!(limiter.tracked(), 0);
    }

    #[test]
    fn keys_hash_the_network_with_a_per_process_salt() {
        let one = Limiter::new(LIMITS).unwrap();
        let two = Limiter::new(LIMITS).unwrap();
        let v4 = ip("198.51.100.7");
        assert_eq!(one.key(v4), one.key(v4));
        assert_ne!(one.key(v4), two.key(v4), "a fresh salt per limiter");
        assert_eq!(one.key(v4), one.key(ip("::ffff:198.51.100.7")));
        assert_ne!(one.key(v4), one.key(ip("198.51.100.8")));
        // IPv6: one /64 is one network.
        assert_eq!(one.key(ip("2001:db8:1:2::1")), one.key(ip("2001:db8:1:2:ffff::9")));
        assert_ne!(one.key(ip("2001:db8:1:2::1")), one.key(ip("2001:db8:1:3::1")));
        assert_ne!(one.key(ip("::")), one.key(ip("0.0.0.0")));
    }

    #[test]
    fn headers_report_every_limit_and_the_tightest_one() {
        let limiter = Limiter::new(LIMITS).unwrap();
        let a = limiter.key(ip("192.0.2.1"));
        let status = limiter.take(a, START).unwrap();
        let headers: HashMap<_, _> = status.headers().into_iter().collect();
        assert_eq!(
            headers["ratelimit-policy"],
            format!("\"ip-short\";q=5;w=600, \"ip-daily\";q=20;w=86400, \"global-daily\";q={};w=86400", LIMITS.daily_total)
        );
        assert_eq!(
            headers["ratelimit"],
            format!("\"ip-short\";r=4;t=600, \"ip-daily\";r=19;t=43200, \"global-daily\";r={};t=43200", LIMITS.daily_total - 1)
        );
        assert_eq!(headers["ratelimit-limit"], "5");
        assert_eq!(headers["ratelimit-remaining"], "4");
        assert_eq!(headers["ratelimit-reset"], "600");
        assert_eq!(status.retry_after(), None);
        for t in 1..5 {
            limiter.take(a, START + t).unwrap();
        }
        let refused = limiter.take(a, START + 100).unwrap_err();
        let headers: HashMap<_, _> = refused.headers().into_iter().collect();
        assert_eq!(refused.retry_after(), Some(500));
        assert_eq!(headers["ratelimit-remaining"], "0");
        assert_eq!(headers["ratelimit-reset"], "500");
    }

    #[test]
    fn a_refusal_says_which_limit_and_when_to_try_again() {
        let limiter = Limiter::new(LIMITS).unwrap();
        let a = limiter.key(ip("192.0.2.1"));
        for t in 0..5 {
            limiter.take(a, START + t).unwrap();
        }
        let body = limiter.take(a, START + 180).unwrap_err().refusal().unwrap();
        let error = &body["error"];
        assert_eq!(error["code"], "rate_limited");
        assert_eq!(error["limit"], "ip_short");
        assert_eq!(error["retry_after_s"], 420);
        assert_eq!(error["reset_at"], "2025-11-16T12:10:00Z");
        assert_eq!(
            error["message"],
            "You have used the 5 photos allowed per 10 minutes from your network. Try again in 7 minutes, at 12:10 UTC."
        );

        let daily = Limiter::new(LIMITS).unwrap();
        let b = daily.key(ip("192.0.2.2"));
        for t in 0..20 {
            daily.take(b, START + t * 600).unwrap();
        }
        let now = START + 20 * 600; // 15:20 UTC
        let body = daily.take(b, now).unwrap_err().refusal().unwrap();
        assert_eq!(body["error"]["limit"], "ip_daily");
        assert_eq!(body["error"]["reset_at"], "2025-11-17T00:00:00Z");
        assert_eq!(
            body["error"]["message"],
            "You have used the 20 photos allowed per day from your network. The allowance starts again at midnight UTC, in 8 hours 40 minutes."
        );

        let global = Limiter::new(Limits { daily_total: 20, ..LIMITS }).unwrap();
        for i in 0..20u32 {
            global.take(global.key(IpAddr::from([10, 0, 0, i as u8])), START).unwrap();
        }
        let body = global.take(global.key(ip("192.0.2.3")), START).unwrap_err().refusal().unwrap();
        assert_eq!(body["error"]["limit"], "global_daily");
        let message = body["error"]["message"].as_str().unwrap();
        assert!(message.starts_with("The demo is at capacity for today"), "{message}");
        assert!(message.ends_with(INSTALL_URL), "{message}");
        assert!(Limiter::new(LIMITS).unwrap().status(a, START).refusal().is_none());
    }

    #[test]
    fn durations_and_times_read_plainly() {
        assert_eq!(duration(1), "1 minute");
        assert_eq!(duration(61), "2 minutes");
        assert_eq!(duration(3600), "1 hour");
        assert_eq!(duration(3601), "1 hour 1 minute");
        assert_eq!(duration(7 * 3600 + 120), "7 hours 2 minutes");
        assert_eq!(timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(timestamp(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(timestamp(START + 3_723), "2025-11-16T13:02:03Z");
        assert_eq!(clock(START + 3_723), "13:02 UTC");
        assert_eq!(thousands(2_000), "2,000");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn the_client_address_cannot_be_forged_through_forwarded_for() {
        let peer: SocketAddr = "169.254.1.1:5000".parse().unwrap();
        let mut headers = HeaderMap::new();
        assert_eq!(ClientAddress::LastForwardedFor.resolve(&headers, Some(peer)), Some(peer.ip()));
        headers.insert("x-forwarded-for", "203.0.113.5".parse().unwrap());
        assert_eq!(ClientAddress::LastForwardedFor.resolve(&headers, Some(peer)), Some(ip("203.0.113.5")));
        // A caller's own entries come first; the front end appends the real one.
        headers.insert("x-forwarded-for", "1.2.3.4, 5.6.7.8,203.0.113.5".parse().unwrap());
        assert_eq!(ClientAddress::LastForwardedFor.resolve(&headers, Some(peer)), Some(ip("203.0.113.5")));
        headers.append("x-forwarded-for", "2001:db8::7".parse().unwrap());
        assert_eq!(ClientAddress::LastForwardedFor.resolve(&headers, Some(peer)), Some(ip("2001:db8::7")));
        // Peer mode ignores the header entirely.
        assert_eq!(ClientAddress::Peer.resolve(&headers, Some(peer)), Some(peer.ip()));
        // Garbage in the last place falls back to the peer, never to an earlier entry.
        headers.insert("x-forwarded-for", "203.0.113.5, nonsense".parse().unwrap());
        assert_eq!(ClientAddress::LastForwardedFor.resolve(&headers, Some(peer)), Some(peer.ip()));
    }

    #[test]
    fn the_cors_origin_is_exact() {
        let origin = Origin::parse("https://cheesejaguar.github.io").unwrap();
        let mut headers = HeaderMap::new();
        assert!(origin.allows(&headers), "scripts send no Origin");
        headers.insert("origin", "https://cheesejaguar.github.io".parse().unwrap());
        assert!(origin.allows(&headers));
        for other in ["https://evil.example", "http://cheesejaguar.github.io", "https://cheesejaguar.github.io.evil.example", "null"] {
            headers.insert("origin", other.parse().unwrap());
            assert!(!origin.allows(&headers), "{other}");
        }
        for bad in ["", "*", "https://", "https://x.org/", "https://x.org/path", "http://example.org", "ftp://x.org"] {
            assert!(Origin::parse(bad).is_err(), "{bad}");
        }
        assert!(Origin::parse("http://localhost:8000").is_ok());
    }

    #[test]
    fn the_pages_state_the_limits_the_server_enforces() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for page in ["site/index.html", "site/terms.html", "docs/API.md"] {
            let text = std::fs::read_to_string(root.join(page)).unwrap();
            // Collapse whitespace so wrapped lines still match.
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if page == "site/index.html" {
                // The page checks the size before uploading, from this attribute.
                assert!(text.contains(&format!("data-max-bytes=\"{}\"", LIMITS.max_bytes)), "{page} must carry data-max-bytes");
            }
            for statement in LIMITS.statements() {
                assert!(text.contains(&statement), "{page} must say \"{statement}\"");
            }
            let capacity = LIMITS.capacity_statement();
            assert_eq!(text.contains(&capacity), page == "docs/API.md", "only the API docs state \"{capacity}\"");
        }
    }
}
