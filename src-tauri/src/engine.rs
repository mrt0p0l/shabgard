// موتورِ شبگرد — پورتِ منطقِ پایتون به Rust: پارسِ کانفیگ، ساب، اتصالِ xray، پینگ، پراکسیِ سیستم.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const NOWIN: u32 = 0x0800_0000; // CREATE_NO_WINDOW
// ── منابعِ کانفیگ ──────────────────────────────────────────────────────────
// چند منبعِ مستقل: اگر یکی فیلتر/قطع شد، بقیه جبران می‌کنند. همه در load_servers
// «موازی» گرفته می‌شوند و ادغام (بدون تکرار) می‌شوند.
// sub.txt = همه‌ی کانفیگ‌های خودمان (چند کانفیگ per کشور) — اپ per کشور گروه می‌کند
// ⚠️ آدرسِ منبعِ اصلی به‌صورتِ base64 در آمده (نه plaintext) تا از رشته‌های باینری/
// لاگِ ISP مستقیم قابل‌خواندن نباشد — obfuscation ساده، نه رمزنگاری.
fn deobf(b64: &str) -> String { b64_str(b64) }
pub const CLOUD_SUB_B64: &str = "aHR0cHM6Ly9yYXcuZ2l0aHVidXNlcmNvbnRlbnQuY29tL21ydDBwMGwvY29uZmlnLWNsb3VkL21haW4vc3ViLnR4dA==";
pub const GAME_SUB_B64: &str = "aHR0cHM6Ly9yYXcuZ2l0aHVidXNlcmNvbnRlbnQuY29tL21ydDBwMGwvY29uZmlnLWNsb3VkL21haW4vZ2FtZS50eHQ=";
// منابعِ عمومیِ معروف (به‌روز و بزرگ) — تنوعِ سرور را چند برابر می‌کنند
const PUBLIC_SUBS: &[&str] = &[
    // barry-far — بزرگ‌ترین تجمیع‌گرِ ایرانی، هر روز آپدیت می‌شود
    "https://raw.githubusercontent.com/barry-far/V2ray-Config/main/All_Configs_Sub.txt",
    // Epodonios — تفکیک‌شده بر اساس پروتکل؛ vless + trojan پوشش کامل
    "https://raw.githubusercontent.com/Epodonios/v2ray-configs/main/Splitted-By-Protocol/vless.txt",
    "https://raw.githubusercontent.com/Epodonios/v2ray-configs/main/Splitted-By-Protocol/trojan.txt",
    // mahdibland Eternity — قدیمی و پایدار
    "https://raw.githubusercontent.com/mahdibland/ShadowsocksAggregator/master/Eternity.txt",
];

#[derive(Serialize, Deserialize, Clone)]
pub struct Server {
    pub link: String,
    pub name: String,
    #[serde(default)]
    pub cc: String,
    pub ping: Option<i32>,
    #[serde(default)]
    pub icon: String,
}

// child = xray (حالتِ پراکسی) ؛ sb = هندلِ پروسه‌ی sing-box یا رَپِرِ گیم‌بوست (حالتِ TUN/گیم، ادمین)
// stop = فایلِ پرچمِ توقف؛ در حالتِ گیم‌بوست با ساختنِ این فایل به رَپر می‌گوییم تنظیماتِ سیستم را برگرداند و ببندد
struct Eng { child: Option<Child>, exe: Option<PathBuf>, sb: Option<isize>, stop: Option<PathBuf>, connected: bool, link: Option<String>, cfg: Option<PathBuf>, port: u16, mport: u16, gen: u64 }

// Kill Switch: اگر xray ناخواسته بمیرد، تا وقتی «متصل» هستیم دوباره بالا می‌آورد.
// پراکسیِ سیستم روی همان پورت می‌ماند، پس در فاصله‌ی افت هم ترافیک نشت نمی‌کند (fail-closed).
fn watchdog(gen: u64) {
    let mut probe_fails: u32 = 0;
    let mut tick: u32 = 0;
    loop {
        // ۵ ثانیه (به‌جای ۲) — برای تشخیصِ افتادنِ تونل کافی است و بیدارباشِ
        // CPU را کم می‌کند (این نخ تا وقتی وصلی زنده است).
        std::thread::sleep(Duration::from_secs(5));
        tick += 1;
        let mut g = eng().lock().unwrap();
        if g.gen != gen || !g.connected { return; }
        // ── حالتِ TUN/گیم ──
        // اینجا نمی‌شود بی‌سروصدا دوباره بالا آورد (UAC لازم است)، و بدتر: وقتی sing-box
        // می‌میرد مسیرِ tun برداشته می‌شود و ترافیک مستقیم می‌رود. پس وضعیت را «قطع»
        // می‌کنیم تا رابط دروغ نگوید و کاربر بداند باید دوباره وصل شود.
        if let Some(h) = g.sb {
            if !handle_alive(h) {
                let stop = g.stop.take();
                g.sb = None; g.connected = false; g.link = None;
                if let Some(p) = g.cfg.take() { let _ = std::fs::remove_file(p); }
                drop(g);
                if let Some(flag) = stop { let _ = std::fs::write(&flag, b"stop"); } // بوست برگردد
                log("⚠️ تونلِ TUN افتاد — قطع شد (دوباره وصل شو)");
                return;
            }
            // هر دقیقه یک‌بار سلامتِ ترافیک چک شود — پروسهٔ زنده ≠ تونلِ زنده؛
            // سرور وسط جلسه فیلتر/مرده می‌شود و بدون این، UI تا ابد «متصل» نشان می‌دهد.
            if tick % 12 == 0 {
                let ok = probe_direct();
                if ok { probe_fails = 0; }
                else {
                    probe_fails += 1;
                    log(format!("⚠️ ترافیک از تونل رد نمی‌شود ({}/۳)", probe_fails));
                    if probe_fails >= 3 {
                        g.connected = false; g.link = None;
                        let h2 = g.sb; let stop = g.stop.take();
                        if let Some(p) = g.cfg.take() { let _ = std::fs::remove_file(p); }
                        drop(g);
                        if let Some(h) = h2 { stop_tun(h, &stop); }
                        log("⚠️ تونل مرده بود — قطع شد. سرور دیگری امتحان کن");
                        return;
                    }
                }
            }
            continue;
        }
        // حالت پراکسی هم همین سلامت‌سنجی را دارد (هر دقیقه)
        if tick % 12 == 0 && g.port > 0 {
            let port = g.port;
            if probe_proxy(port) { probe_fails = 0; }
            else {
                probe_fails += 1;
                if probe_fails >= 3 {
                    log("⚠️ سرور جواب نمی‌دهد (احتمالاً فیلتر شد) — قطع کردم");
                    g.connected = false; g.link = None;
                    let child = g.child.take();
                    if let Some(p) = g.cfg.take() { let _ = std::fs::remove_file(p); }
                    drop(g);
                    if let Some(mut c) = child { let _ = c.kill(); }
                    unset_proxy();
                    return;
                }
            }
        }
        let dead = match g.child.as_mut() {
            Some(c) => c.try_wait().ok().flatten().is_some(),
            None => true,
        };
        if dead {
            log("اتصال افت کرد — تلاشِ خودکار برای اتصالِ مجدد…");
            let exe = g.exe.clone().unwrap_or_else(|| bin("xray.exe"));
            let is_relay = exe.file_name().map(|n| n == "relay.exe").unwrap_or(false);
            if let Some(cfg) = g.cfg.clone() {
                // رله آرگومانِ متفاوتی دارد (`-c` بدونِ `run`) و باید CAِ ثابتش را ببیند
                let mut cmd = Command::new(&exe);
                if is_relay { cmd.arg("-c").arg(&cfg).env("DFT_CA_DIR", app_data_dir().join("relay-ca")); }
                else { cmd.arg("run").arg("-c").arg(&cfg); }
                if let Ok(child) = cmd
                    .env("XRAY_LOCATION_ASSET", asset_dir())
                    .creation_flags(NOWIN).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
                    .spawn()
                {
                    adopt(child.id());
                    g.child = Some(child);
                    log("دوباره متصل شد ✅");
                }
            }
        }
    }
}

// نسلِ اتصال — برای «کنسل»: هر بار که کاربر کنسل می‌زند این جلو می‌رود و
// اتصالِ در حالِ انجام در اولین ایستگاه خودش را می‌کُشد و برمی‌گردد.
fn connect_gen() -> &'static std::sync::atomic::AtomicU64 {
    static G: OnceLock<std::sync::atomic::AtomicU64> = OnceLock::new();
    G.get_or_init(|| std::sync::atomic::AtomicU64::new(0))
}
fn cur_gen() -> u64 { connect_gen().load(std::sync::atomic::Ordering::SeqCst) }
pub fn cancel_connect() {
    connect_gen().fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    log("کنسل شد");
}

fn eng() -> &'static Mutex<Eng> {
    static E: OnceLock<Mutex<Eng>> = OnceLock::new();
    E.get_or_init(|| Mutex::new(Eng { child: None, exe: None, sb: None, stop: None, connected: false, link: None, cfg: None, port: 0, mport: 0, gen: 0 }))
}

// پروسهٔ sing-box مشترکِ تستِ دسته‌ای (hy2/tuic/wg) — فقط در طولِ test_batch زنده است
fn sb_shared_child() -> &'static Mutex<Option<std::process::Child>> {
    static S: OnceLock<Mutex<Option<std::process::Child>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}
fn shared_cfg_file() -> &'static Mutex<Option<PathBuf>> {
    static F: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    F.get_or_init(|| Mutex::new(None))
}

// ==========================================================================
//   Job Object — بچه‌پروسه‌ها با مرگِ اپ می‌میرند
//   در ویندوز بچه‌پروسه با بسته‌شدنِ پدر نمی‌میرد. نتیجه: اگر اپ کرش کند یا با
//   Task Manager بسته شود، xray/sing-box/relay یتیم می‌مانند، فایل‌ها قفل می‌شوند
//   (خطای «Error opening file for writing» موقعِ نصبِ نسخه‌ی جدید) و پراکسی رها
//   می‌ماند. با گذاشتنِ هر بچه در یک Job با KILL_ON_JOB_CLOSE این مشکل ریشه‌کن می‌شود.
// ==========================================================================
#[link(name = "kernel32")]
extern "system" {
    fn CreateJobObjectW(attr: *mut core::ffi::c_void, name: *const u16) -> *mut core::ffi::c_void;
    fn SetInformationJobObject(job: *mut core::ffi::c_void, class: i32,
                               info: *mut core::ffi::c_void, len: u32) -> i32;
    fn AssignProcessToJobObject(job: *mut core::ffi::c_void, proc_: *mut core::ffi::c_void) -> i32;
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut core::ffi::c_void;
}
#[repr(C)]
#[derive(Default)]
struct JobBasicLimit {
    per_process_user_time: i64, per_job_user_time: i64,
    limit_flags: u32, min_working_set: usize, max_working_set: usize,
    active_process_limit: u32, affinity: usize, priority_class: u32, scheduling_class: u32,
}
#[repr(C)]
#[derive(Default)]
struct JobExtendedLimit {
    basic: JobBasicLimit,
    io_read_ops: u64, io_write_ops: u64, io_other_ops: u64,
    io_read_bytes: u64, io_write_bytes: u64, io_other_bytes: u64,
    process_memory_limit: usize, job_memory_limit: usize,
    peak_process_memory: usize, peak_job_memory: usize,
}

fn job() -> isize {
    static J: OnceLock<isize> = OnceLock::new();
    *J.get_or_init(|| unsafe {
        let h = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
        if h.is_null() { return 0; }
        let mut info = JobExtendedLimit::default();
        info.basic.limit_flags = 0x0000_2000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        SetInformationJobObject(h, 9 /* ExtendedLimitInformation */,
            &mut info as *mut _ as *mut core::ffi::c_void,
            std::mem::size_of::<JobExtendedLimit>() as u32);
        h as isize
    })
}

// هر بچه‌ای که می‌سازیم را داخلِ Job بگذار تا با اپ بمیرد
fn adopt(pid: u32) {
    let j = job();
    if j == 0 { return; }
    unsafe {
        // PROCESS_SET_QUOTA | PROCESS_TERMINATE
        let p = OpenProcess(0x0100 | 0x0001, 0, pid);
        if !p.is_null() {
            AssignProcessToJobObject(j as *mut _, p);
            CloseHandle(p);
        }
    }
}

// ---------- لاگِ پشت‌صحنه ----------
fn logs() -> &'static Mutex<Vec<String>> {
    static L: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(Vec::new()))
}
pub fn log(msg: impl Into<String>) {
    let secs = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs()).unwrap_or(0) + 3 * 3600 + 30 * 60) % 86400; // وقتِ ایران
    let line = format!("{:02}:{:02}:{:02}  {}", secs / 3600, (secs / 60) % 60, secs % 60, msg.into());
    let mut l = logs().lock().unwrap();
    l.push(line);
    let n = l.len();
    if n > 300 { l.drain(0..n - 300); }
}
pub fn get_log() -> Vec<String> { logs().lock().unwrap().clone() }

static BIN_DIR: OnceLock<PathBuf> = OnceLock::new();
pub fn set_bin_dir(p: PathBuf) { let _ = BIN_DIR.set(p); }
fn bin(name: &str) -> PathBuf {
    // ۱) resource_dir/binaries (نسخه‌ی نصبی)
    if let Some(d) = BIN_DIR.get() { let p = d.join(name); if p.exists() { return p; } }
    // ۲) کنارِ exe (binaries یا خودِ پوشه)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for cand in [dir.join("binaries").join(name), dir.join(name)] {
                if cand.exists() { return cand; }
            }
        }
    }
    // ۳) توسعه
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/binaries")).join(name)
}

// ---------- کمک‌ها ----------
fn b64_safe(s: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    let t = s.trim().replace('-', "+").replace('_', "/");
    let t = format!("{}{}", t, "=".repeat((4 - t.len() % 4) % 4));
    base64::engine::general_purpose::STANDARD.decode(t).ok()
}
fn b64_str(s: &str) -> String {
    if let Some(v) = b64_safe(s) { return String::from_utf8_lossy(&v).into_owned(); }
    // فرمتِ ساب‌های تلگرامی: چند بلوکِ base64 که با پدینگِ خودشان («=») به هم چسبیده‌اند
    // و newline ندارند. دیکودِ یکجا خطا می‌دهد (= وسطِ رشته)، پس بلوک‌بلوک می‌گیریم.
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_whitespace() { continue; }
        if c == '=' {
            if !cur.is_empty() {
                if let Some(mut v) = b64_safe(&cur) { out.append(&mut v); out.push(b'\n'); }
                cur.clear();
            }
        } else { cur.push(c); }
    }
    if !cur.is_empty() { if let Some(mut v) = b64_safe(&cur) { out.append(&mut v); } }
    String::from_utf8_lossy(&out).into_owned()
}

fn parse_qs(q: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for pair in q.split('&') {
        if pair.is_empty() { continue; }
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or("").to_string();
        let v = it.next().unwrap_or("");
        m.insert(k, urlencoding::decode(v).map(|c| c.into_owned()).unwrap_or_else(|_| v.to_string()));
    }
    m
}
fn g<'a>(m: &'a HashMap<String, String>, k: &str) -> &'a str { m.get(k).map(|s| s.as_str()).unwrap_or("") }

fn stream_settings(net: &str, sec: &str, p: &HashMap<String, String>, host: &str, sni_default: &str) -> Value {
    let net = if net.is_empty() { "tcp" } else { net };
    let mut ss = json!({ "network": net });
    let sec = sec.to_lowercase();
    // اثرِ انگشتِ TLS: اگر کانفیگ نداشت، پیش‌فرض chrome — تا مثلِ مرورگرِ واقعی باشد و
    // Cloudflare (Claude/سایت‌های محافظت‌شده) بلاک نکند. (این کاری است که v2rayN می‌کند.)
    let fp = if g(p, "fp").is_empty() { "chrome" } else { g(p, "fp") };
    if sec == "tls" || sec == "xtls" {
        ss["security"] = json!("tls");
        let mut tls = json!({ "fingerprint": fp });
        let sni = if !g(p, "sni").is_empty() { g(p, "sni") } else if !g(p, "host").is_empty() { g(p, "host") } else { sni_default };
        if !sni.is_empty() { tls["serverName"] = json!(sni); }
        if !g(p, "alpn").is_empty() { tls["alpn"] = json!(g(p, "alpn").split(',').collect::<Vec<_>>()); }
        if g(p, "allowInsecure") == "1" || g(p, "insecure") == "1" { tls["allowInsecure"] = json!(true); }
        ss["tlsSettings"] = tls;
    } else if sec == "reality" {
        ss["security"] = json!("reality");
        let mut r = json!({ "fingerprint": fp });
        let sni = if !g(p, "sni").is_empty() { g(p, "sni") } else { sni_default };
        if !sni.is_empty() { r["serverName"] = json!(sni); }
        if !g(p, "pbk").is_empty() { r["publicKey"] = json!(g(p, "pbk")); }
        if !g(p, "sid").is_empty() { r["shortId"] = json!(g(p, "sid")); }
        if !g(p, "spx").is_empty() { r["spiderX"] = json!(g(p, "spx")); }
        ss["realitySettings"] = r;
    } else {
        ss["security"] = json!("none");
    }
    match net {
        "ws" => {
            let mut w = json!({});
            if !g(p, "path").is_empty() { w["path"] = json!(g(p, "path")); }
            let h = if !g(p, "host").is_empty() { g(p, "host") } else { host };
            if !h.is_empty() { w["headers"] = json!({ "Host": h }); }
            ss["wsSettings"] = w;
        }
        "grpc" => {
            let sn = if !g(p, "serviceName").is_empty() { g(p, "serviceName") } else { g(p, "path") };
            let mut gg = json!({ "serviceName": sn });
            if g(p, "mode") == "multi" { gg["multiMode"] = json!(true); }
            ss["grpcSettings"] = gg;
        }
        "h2" | "http" => {
            ss["network"] = json!("http");
            let mut hh = json!({});
            if !g(p, "path").is_empty() { hh["path"] = json!(g(p, "path")); }
            let h = if !g(p, "host").is_empty() { g(p, "host") } else { host };
            if !h.is_empty() { hh["host"] = json!([h]); }
            ss["httpSettings"] = hh;
        }
        // XHTTP — ترنسپورتِ جدیدِ xray («بعد از REALITY»): ترافیک پشت CDN قایم می‌شود.
        // پارامترهای لینک: mode (auto/packet-up/stream-up/stream-one)، host، path،
        // extra (XMUX و downloadSettings در لینک‌های کامل)
        "xhttp" | "splithttp" => {
            ss["network"] = json!("xhttp");
            let mut xh = json!({});
            if !g(p, "path").is_empty() { xh["path"] = json!(g(p, "path")); }
            let h = if !g(p, "host").is_empty() { g(p, "host") } else { host };
            if !h.is_empty() { xh["host"] = json!(h); }
            let m = g(p, "mode");
            if !m.is_empty() && m != "auto" { xh["mode"] = json!(m); }
            if !g(p, "xmuxMaxConcurrency").is_empty() {
                xh["xmux"] = json!({ "maxConcurrency": g(p, "xmuxMaxConcurrency"),
                                     "maxConnections": g(p, "xmuxMaxConnections"),
                                     "cMaxReuseTimes": g(p, "xmuxCMaxReuseTimes") });
            }
            ss["xhttpSettings"] = xh;
        }
        "tcp" => {
            if g(p, "headerType") == "http" {
                let h = if !g(p, "host").is_empty() { g(p, "host") } else { host };
                ss["tcpSettings"] = json!({ "header": { "type": "http", "request": { "headers": { "Host": [h] } } } });
            }
        }
        _ => {}
    }
    ss
}

pub fn link_to_outbound(link: &str) -> Option<Value> {
    let low = link.to_lowercase();
    if low.starts_with("vmess://") {
        let j: Value = serde_json::from_str(&b64_str(&link[8..])).ok()?;
        let add = j.get("add")?.as_str()?.to_string();
        let port: u64 = j.get("port").and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))?;
        if add.is_empty() || port == 0 { return None; }
        let net = j.get("net").and_then(|v| v.as_str()).unwrap_or("tcp");
        let tlsv = j.get("tls").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        let sec = if tlsv == "tls" || tlsv == "reality" { "tls" } else { "none" };
        let gs = |k: &str| j.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut p = HashMap::new();
        for (a, b) in [("path", gs("path")), ("host", gs("host")), ("sni", gs("sni")),
            ("serviceName", gs("path")), ("headerType", gs("type")), ("alpn", gs("alpn")), ("fp", gs("fp"))] { p.insert(a.to_string(), b); }
        let aid: u64 = j.get("aid").and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0);
        let scy = gs("scy"); let scy = if scy.is_empty() { "auto".into() } else { scy };
        return Some(json!({
            "protocol": "vmess", "tag": "proxy",
            "settings": { "vnext": [{ "address": add, "port": port,
                "users": [{ "id": j.get("id")?.as_str()?, "alterId": aid, "security": scy }] }] },
            "streamSettings": stream_settings(net, sec, &p, &gs("host"), &add)
        }));
    }
    if low.starts_with("vless://") {
        let u = url::Url::parse(link).ok()?;
        let uid = u.username(); let add = u.host_str()?.to_string(); let port = u.port()?;
        if uid.is_empty() { return None; }
        let p = parse_qs(u.query().unwrap_or(""));
        let mut user = json!({ "id": uid, "encryption": if g(&p,"encryption").is_empty() {"none"} else {g(&p,"encryption")} });
        if !g(&p, "flow").is_empty() { user["flow"] = json!(g(&p, "flow")); }
        return Some(json!({
            "protocol": "vless", "tag": "proxy",
            "settings": { "vnext": [{ "address": add, "port": port, "users": [user] }] },
            "streamSettings": stream_settings(if g(&p,"type").is_empty(){"tcp"}else{g(&p,"type")},
                if g(&p,"security").is_empty(){"none"}else{g(&p,"security")}, &p, g(&p,"host"), &add)
        }));
    }
    if low.starts_with("trojan://") {
        let u = url::Url::parse(link).ok()?;
        let pwd = urlencoding::decode(u.username()).map(|c| c.into_owned()).unwrap_or_default();
        let add = u.host_str()?.to_string(); let port = u.port()?;
        if pwd.is_empty() { return None; }
        let p = parse_qs(u.query().unwrap_or(""));
        return Some(json!({
            "protocol": "trojan", "tag": "proxy",
            "settings": { "servers": [{ "address": add, "port": port, "password": pwd }] },
            "streamSettings": stream_settings(if g(&p,"type").is_empty(){"tcp"}else{g(&p,"type")},
                if g(&p,"security").is_empty(){"tls"}else{g(&p,"security")}, &p, g(&p,"host"), &add)
        }));
    }
    if low.starts_with("ss://") {
        let body = link[5..].split('#').next().unwrap_or("");
        let (method, password, add, port);
        if let Some(idx) = body.find('@') {
            let mut userinfo = body[..idx].to_string();
            let hostpart = body[idx + 1..].split('?').next().unwrap_or("").to_string();
            if !userinfo.contains(':') { userinfo = b64_str(&userinfo); }
            let mut mi = userinfo.splitn(2, ':');
            method = mi.next()?.to_string(); password = mi.next()?.to_string();
            let hp = hostpart.rsplit_once(':')?; add = hp.0.to_string();
            port = hp.1.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok()?;
        } else {
            let dec = b64_str(body);
            let (ui, hp) = dec.split_once('@')?;
            let mut mi = ui.splitn(2, ':'); method = mi.next()?.to_string(); password = mi.next()?.to_string();
            let hp2 = hp.rsplit_once(':')?; add = hp2.0.to_string();
            port = hp2.1.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok()?;
        }
        let _: u16 = port;
        return Some(json!({
            "protocol": "shadowsocks", "tag": "proxy",
            "settings": { "servers": [{ "address": add, "port": port, "method": method, "password": password }] },
            "streamSettings": { "network": "tcp" }
        }));
    }
    None
}

pub fn host_port_of(link: &str) -> Option<(String, u16)> {
    if let Some(ob) = link_to_outbound(link) {
        let s = &ob["settings"];
        if let Some(node) = s.get("vnext").or_else(|| s.get("servers")).and_then(|a| a.get(0)) {
            if let (Some(h), Some(p)) = (node.get("address").and_then(|v| v.as_str()),
                                         node.get("port").and_then(|v| v.as_u64())) {
                return Some((h.to_string(), p as u16));
            }
        }
    }
    // fallback: پروتکل‌های URL-form که xray پارس نمی‌کند (hysteria2/tuic)
    if let Ok(u) = url::Url::parse(link) {
        if let (Some(h), Some(p)) = (u.host_str(), u.port()) { return Some((h.to_string(), p)); }
    }
    None
}

pub fn remark_of(link: &str) -> String {
    if is_wireguard(link) { if let Some(n) = wg_name(link) { return n; } }
    if let Some(idx) = link.find('#') {
        let r = urlencoding::decode(&link[idx + 1..]).map(|c| c.into_owned()).unwrap_or_default();
        if !r.trim().is_empty() { return r.trim().to_string(); }
    }
    host_port_of(link).map(|(h, p)| format!("{h}:{p}")).unwrap_or_else(|| link.chars().take(40).collect())
}

// ---------- شبکه ----------
pub fn fetch_text(u: &str) -> Option<String> {
    let resp = ureq::get(u).set("User-Agent", "Mozilla/5.0").timeout(Duration::from_secs(20)).call().ok()?;
    resp.into_string().ok()
}

#[derive(Serialize, Clone)]
pub struct NetInfo { pub isp: String, pub ip: String, pub cc: String, pub country: String }

// اسمِ خامِ ISP را به نامِ آشنای فارسی (شبکه‌های ایران) تبدیل می‌کند.
fn friendly_isp(isp: &str) -> String {
    let l = isp.to_lowercase();
    // نکته: چند اپراتور روی شبکه‌ی مشترک‌اند (MVNO)؛ برچسبِ ترکیبی می‌دهیم چون کانفیگ‌های
    // کارآمدشان یکی است — سامانتل روی رایتل، شاتل‌موبایل/اپتل روی همراه‌اول.
    let table: &[(&[&str], &str)] = &[
        (&["rightel", "samantel", "saman tel", "sindad"], "رایتل / سامانتل"),
        (&["mci", "mobile communication company", "hamrah", "hamrahe aval",
           "shatel mobile", "shatelmobile", "aptel"], "همراه اول"),
        (&["irancell", "mtn"], "ایرانسل"),
        (&["telecommunication company of iran", "tci", "mokhaberat", "data communication",
           "iran telecommunication"], "مخابرات"),
        (&["asiatech", "asia tech"], "آسیاتک"),
        (&["parsonline", "pars online"], "پارس‌آنلاین"),
        (&["respina"], "رسپینا"),
        (&["mobin net", "mobinnet", "mobin"], "مبین‌نت"),
        (&["hiweb", "aria shatel", "aria eshatel"], "های‌وب"),
        (&["shatel"], "شاتل"),
        (&["zitel"], "زیتل"),
        (&["datak"], "داتک"),
        (&["fanava", "fanap"], "فن‌آوا"),
        (&["sabanet", "sabaidc"], "صبانت"),
        (&["afranet"], "افرانت"),
        (&["pishgaman"], "پیشگامان"),
        (&["arvan"], "آروان"),
        (&["cloudflare"], "کلادفلر"),
    ];
    for (keys, name) in table {
        if keys.iter().any(|k| l.contains(k)) { return name.to_string(); }
    }
    isp.to_string() // ناشناخته → همان اسمِ خام
}

// نتِ کاربر را تشخیص می‌دهد (چند سرویسِ HTTPS با fallback چون بعضی از ایران فیلترند).
pub fn detect_net() -> NetInfo {
    let urls = [
        "https://api.ip.sb/geoip",
        "https://ipwho.is/",
        "http://ip-api.com/json/?fields=query,isp,org,countryCode,country",
    ];
    for url in urls {
        let Some(txt) = fetch_text(url) else { continue };
        let Ok(j) = serde_json::from_str::<Value>(&txt) else { continue };
        let ip = j["ip"].as_str().or_else(|| j["query"].as_str()).unwrap_or("").to_string();
        let isp_raw = j["isp"].as_str()
            .or_else(|| j["connection"]["isp"].as_str())
            .or_else(|| j["organization"].as_str())
            .or_else(|| j["connection"]["org"].as_str())
            .or_else(|| j["org"].as_str())
            .unwrap_or("");
        if isp_raw.is_empty() { continue; } // فقط منبعی که واقعاً ISP داد
        let cc = j["country_code"].as_str().or_else(|| j["countryCode"].as_str()).or_else(|| j["country"].as_str()).unwrap_or("").to_string();
        let country = j["country"].as_str().or_else(|| j["country_name"].as_str()).unwrap_or("").to_string();
        let isp = if isp_raw.is_empty() { "نامشخص".to_string() } else { friendly_isp(isp_raw) };
        log(format!("نتِ تو: {} ({}) [خام: {}]", isp, ip, isp_raw));
        return NetInfo { isp, ip, cc: if cc.len() == 2 { cc } else { String::new() }, country };
    }
    log("تشخیصِ نت نشد (همه‌ی سرویس‌ها ناموفق)");
    NetInfo { isp: "نامشخص".into(), ip: String::new(), cc: String::new(), country: String::new() }
}

// ==========================================================================
//   تحلیل‌گرِ نت — می‌فهمد نتِ تو *چطور* فیلتر شده و چه چیزی روی آن بهتر کار می‌کند
//   نتیجه: هم فوراً در همین اپ استفاده می‌شود (انتخابِ نوعِ کانفیگِ درست)،
//   هم می‌تواند (بی‌نام) به گیت‌هاب برود تا سابِ مخصوصِ همان اپراتور ساخته شود.
// ==========================================================================
#[derive(Serialize, Clone, Default)]
pub struct NetProbe {
    pub isp: String,
    pub udp_dns: bool,      // آیا UDP اصلاً رد می‌شود؟ (DNS روی 53/udp)
    pub quic: bool,         // آیا QUIC/UDP-443 باز است؟ (شرطِ کارکردنِ hysteria2/tuic)
    pub tls_direct: bool,   // TLS مستقیم به یک سایتِ عادی
    pub sni_block: bool,    // آیا بر اساسِ SNI بلاک می‌کند؟ (نشانه‌ی DPI)
    pub dns_poison: bool,   // آیا DNS مسموم است؟
    pub ports: Vec<u16>,    // کدام پورت‌های بیرونی باز است
    pub cdn_ok: bool,       // آیا IPهای کلادفلر مستقیماً جواب می‌دهند
    pub advice: String,     // نتیجه‌گیریِ عملی (چه نوع کانفیگی برای این نت بهتر است)
}

fn tcp_reach(host: &str, port: u16, ms: u64) -> bool {
    let Ok(mut it) = std::net::ToSocketAddrs::to_socket_addrs(&format!("{host}:{port}")) else { return false };
    let Some(sa) = it.next() else { return false };
    TcpStream::connect_timeout(&sa, Duration::from_millis(ms)).is_ok()
}

// یک پرسشِ DNS خام روی UDP — هم «UDP رد می‌شود؟» را می‌گوید هم «جواب مسموم است؟»
fn dns_udp_query(server: &str, domain: &str) -> Option<Vec<std::net::Ipv4Addr>> {
    use std::net::UdpSocket;
    let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.set_read_timeout(Some(Duration::from_millis(2500))).ok()?;
    let mut q: Vec<u8> = vec![0x13, 0x37, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0];
    for label in domain.split('.') {
        q.push(label.len() as u8);
        q.extend_from_slice(label.as_bytes());
    }
    q.push(0);
    q.extend_from_slice(&[0, 1, 0, 1]); // A, IN
    sock.send_to(&q, (server, 53)).ok()?;
    let mut buf = [0u8; 512];
    let (n, _) = sock.recv_from(&mut buf).ok()?;
    if n < 12 { return None; }
    let answers = u16::from_be_bytes([buf[6], buf[7]]) as usize;
    // از انتهای سوال عبور کن
    let mut i = 12;
    while i < n && buf[i] != 0 { i += buf[i] as usize + 1; }
    i += 5;
    let mut ips = vec![];
    for _ in 0..answers {
        if i + 12 > n { break; }
        if buf[i] & 0xC0 == 0xC0 { i += 2; } else { while i < n && buf[i] != 0 { i += buf[i] as usize + 1; } i += 1; }
        if i + 10 > n { break; }
        let rtype = u16::from_be_bytes([buf[i], buf[i + 1]]);
        let rdlen = u16::from_be_bytes([buf[i + 8], buf[i + 9]]) as usize;
        i += 10;
        if rtype == 1 && rdlen == 4 && i + 4 <= n {
            ips.push(std::net::Ipv4Addr::new(buf[i], buf[i + 1], buf[i + 2], buf[i + 3]));
        }
        i += rdlen;
    }
    Some(ips)
}

// آیا QUIC (UDP/443) رد می‌شود؟ یک Initial packet خام می‌فرستیم و منتظرِ هر پاسخی می‌مانیم.
fn quic_reachable(ip: &str) -> bool {
    use std::net::UdpSocket;
    let Ok(sock) = UdpSocket::bind("0.0.0.0:0") else { return false };
    let _ = sock.set_read_timeout(Some(Duration::from_millis(2200)));
    // بسته‌ی شبه-Initial؛ سرورِ واقعی معمولاً با Version Negotiation جواب می‌دهد
    let mut pkt = vec![0xC0, 0x00, 0x00, 0x00, 0x01, 0x08];
    pkt.extend_from_slice(&[0x11; 8]); // DCID
    pkt.push(0x00);                    // SCID len
    pkt.extend_from_slice(&[0x00, 0x44, 0x9E]);
    pkt.resize(1200, 0);               // QUIC حداقل ۱۲۰۰ بایت
    if sock.send_to(&pkt, (ip, 443)).is_err() { return false; }
    let mut buf = [0u8; 1500];
    sock.recv_from(&mut buf).is_ok()
}

// یک ClientHello با SNI مشخص می‌فرستد و می‌بیند دست‌دهی می‌شکند یا نه (تشخیصِ DPI روی SNI)
fn tls_sni_ok(host: &str) -> bool {
    let Ok(mut it) = std::net::ToSocketAddrs::to_socket_addrs(&format!("{host}:443")) else { return false };
    let Some(sa) = it.next() else { return false };
    let Ok(mut s) = TcpStream::connect_timeout(&sa, Duration::from_millis(3000)) else { return false };
    let _ = s.set_read_timeout(Some(Duration::from_millis(3000)));
    // ClientHello مینیمال با SNI
    let hb = host.as_bytes();
    let mut ext_sni = vec![0x00, 0x00];                                   // server_name
    let sni_body_len = hb.len() + 3;
    ext_sni.extend_from_slice(&((sni_body_len + 2) as u16).to_be_bytes());
    ext_sni.extend_from_slice(&(sni_body_len as u16).to_be_bytes());
    ext_sni.push(0x00);
    ext_sni.extend_from_slice(&(hb.len() as u16).to_be_bytes());
    ext_sni.extend_from_slice(hb);
    let exts = ext_sni;
    let mut hello = vec![0x03, 0x03];
    hello.extend_from_slice(&[0x42u8; 32]);   // random
    hello.push(0x00);                          // session id
    hello.extend_from_slice(&[0x00, 0x02, 0x13, 0x01]); // cipher: TLS_AES_128_GCM
    hello.extend_from_slice(&[0x01, 0x00]);    // compression
    hello.extend_from_slice(&(exts.len() as u16).to_be_bytes());
    hello.extend_from_slice(&exts);
    let mut hs = vec![0x01, 0x00];
    hs.extend_from_slice(&((hello.len() >> 8) as u8).to_be_bytes());
    hs.push((hello.len() & 0xff) as u8);
    hs.extend_from_slice(&hello);
    let mut rec = vec![0x16, 0x03, 0x01];
    rec.extend_from_slice(&(hs.len() as u16).to_be_bytes());
    rec.extend_from_slice(&hs);
    use std::io::Write;
    if s.write_all(&rec).is_err() { return false; }
    let mut buf = [0u8; 8];
    match s.read(&mut buf) {
        Ok(n) if n > 0 => buf[0] == 0x16,   // ServerHello = عبور کرد؛ RST/بسته‌شدن = بلاک
        _ => false,
    }
}

// نتِ کاربر را کامل تحلیل می‌کند (چند ثانیه طول می‌کشد، همه موازی)
pub fn analyze_net() -> NetProbe {
    let mut p = NetProbe::default();
    p.isp = detect_net().isp;

    // UDP: پرسشِ DNS به 8.8.8.8 (اگر جواب نیاید یعنی UDP بسته/محدود است)
    let g = dns_udp_query("8.8.8.8", "example.com");
    p.udp_dns = g.as_ref().map(|v| !v.is_empty()).unwrap_or(false);

    // مسمومیتِ DNS: دامنه‌ی معمولاً فیلترشده را از DNSِ خودِ ISP بپرس؛ IPِ ایرانی/۱۰.۱۰.۳۴.۳۴ = مسموم
    if let Some(ips) = dns_udp_query("8.8.8.8", "www.youtube.com") {
        p.dns_poison = ips.iter().any(|ip| {
            let o = ip.octets();
            o[0] == 10 || (o[0] == 127) || (o[0] == 0)   // 10.10.34.34 و مشابه = صفحه‌ی فیلترینگ
        });
    }

    p.quic = quic_reachable("1.1.1.1");
    p.tls_direct = tls_sni_ok("www.cloudflare.com");
    // اگر TLS عادی جواب بدهد ولی SNIِ فیلترشده نه → DPI روی SNI فعال است
    p.sni_block = p.tls_direct && !tls_sni_ok("www.youtube.com");
    p.cdn_ok = tcp_reach("104.16.132.229", 443, 2500);

    for port in [443u16, 80, 8443, 2053, 2087, 8080] {
        if tcp_reach("cloudflare.com", port, 2200) { p.ports.push(port); }
    }

    // نتیجه‌گیریِ عملی — کد می‌دهیم تا رابط هر زبانی را خودش بنویسد
    let mut a: Vec<&str> = vec![];
    if p.quic && p.udp_dns { a.push("udp_ok"); }
    else if p.udp_dns { a.push("udp_limited"); }
    else { a.push("udp_blocked"); }
    if p.sni_block { a.push("sni_dpi"); }
    if p.dns_poison { a.push("dns_poison"); }
    if !p.ports.contains(&443) { a.push("no_443"); }
    if p.cdn_ok { a.push("cdn_ok"); }
    p.advice = a.join(",");
    log(format!("تحلیلِ نت: UDP={} QUIC={} SNI-block={} DNS-poison={} پورت‌ها={:?}",
        p.udp_dns, p.quic, p.sni_block, p.dns_poison, p.ports));
    p
}

pub fn list_from_subs(subs: &[String], manual: &[String]) -> Vec<Server> {
    list_from_subs_mode(subs, manual, false)
}

// game=true → اول از سابِ گیم بگیر (کم‌پینگ، کم‌جیتر، ترجیحاً hy2/tuic)، بعد بقیه
// کدِ کشور از پرچمِ ایموجی یا پسوندِ اسم — سمتِ سرور تا همه‌جا یکسان باشد
fn cc_of_name(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut i = 0;
    while i + 1 < chars.len() {
        let (a, b) = (chars[i] as u32, chars[i + 1] as u32);
        if (0x1F1E6..=0x1F1FF).contains(&a) && (0x1F1E6..=0x1F1FF).contains(&b) {
            let cc: String = [
                char::from_u32(a - 0x1F1E6 + 'A' as u32).unwrap_or('?'),
                char::from_u32(b - 0x1F1E6 + 'A' as u32).unwrap_or('?'),
            ].iter().collect();
            return cc;
        }
        i += 1;
    }
    String::new()
}

// ── آپدیتِ خودکار از GitHub Releases ────────────────────────────────────────
// نسخهٔ فعلی از env! CARGO_PKG_VERSION. ریپو با متغیرِ محیطی SHABGARD_REPO قابل
// تغییر است (پیش‌فرض برای buildهای عمومی) — بدونِ هاردکدِ هویت.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
fn update_repo() -> String {
    std::env::var("SHABGARD_REPO").unwrap_or_else(|_| "shabgard-app/shabgard".into())
}

// آخرین نسخهٔ منتشرشده: tag باید شبیه v1.2.3 باشد → "1.2.3"
pub fn check_update() -> Result<Option<String>, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", update_repo());
    let resp = ureq::get(&url)
        .set("User-Agent", "Shabgard")
        .timeout(Duration::from_secs(10))
        .call().map_err(|e| e.to_string())?;
    let j: Value = resp.into_json().map_err(|e| e.to_string())?;
    let tag = j["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
    if tag.is_empty() { return Ok(None); }
    // مقایسهٔ عددیِ هر بخش
    let newer = {
        let cur: Vec<u32> = APP_VERSION.split('.').filter_map(|s| s.parse().ok()).collect();
        let new: Vec<u32> = tag.split('.').filter_map(|s| s.parse().ok()).collect();
        new > cur
    };
    if newer { Ok(Some(tag)) } else { Ok(None) }
}

// دانلود installer جدید و اجرا (نصاب خودش اپ را می‌بندد و آپدیت می‌کند)
pub fn download_update() -> Result<PathBuf, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", update_repo());
    let resp = ureq::get(&url)
        .set("User-Agent", "Shabgard")
        .timeout(Duration::from_secs(15))
        .call().map_err(|e| e.to_string())?;
    let j: Value = resp.into_json().map_err(|e| e.to_string())?;
    // asset اول که setup.exe است
    let mut dl_url = None;
    if let Some(assets) = j["assets"].as_array() {
        for a in assets {
            if let Some(name) = a["name"].as_str() {
                if name.contains("setup") && name.ends_with(".exe") {
                    dl_url = a["browser_download_url"].as_str().map(|s| s.to_string());
                    break;
                }
            }
        }
    }
    let Some(url) = dl_url else { return Err("فایل نصب در Releases پیدا نشد".into()) };
    let dest = std::env::temp_dir().join("shabgard-update-setup.exe");
    let resp = ureq::get(&url)
        .set("User-Agent", "Shabgard")
        .timeout(Duration::from_secs(600))
        .call().map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();
    let mut f = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut f).map_err(|e| e.to_string())?;
    drop(f);
    Ok(dest)
}

pub fn list_from_subs_mode(subs: &[String], manual: &[String], game: bool) -> Vec<Server> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    let push = |l: &str, out: &mut Vec<Server>, seen: &mut std::collections::HashSet<String>| {
        let l = l.trim();
        if l.contains("://") && !seen.contains(l) && (is_wireguard(l) || link_to_outbound(l).is_some() || singbox_outbound(l).is_some()) {
            seen.insert(l.to_string());
            let name = remark_of(l);
            out.push(Server { link: l.to_string(), cc: cc_of_name(&name), name, ping: None, icon: String::new() });
        }
    };
    for l in manual { push(l, &mut out, &mut seen); }
    // در حالتِ گیم، سابِ گیم اول می‌آید تا بهترین‌های بازی بالای لیست باشند
    let (cloud, game_sub) = (deobf(CLOUD_SUB_B64), deobf(GAME_SUB_B64));
    let mut all: Vec<String> = if game {
        vec![game_sub, cloud]
    } else {
        vec![cloud]
    };
    // منابعِ عمومی — تنوعِ سرور؛ هر کدام که در دسترس بود اضافه می‌شود
    all.extend(PUBLIC_SUBS.iter().map(|s| s.to_string()));
    all.extend(subs.iter().cloned());
    // گرفتنِ «موازی» همهٔ ساب‌ها — سریالی بودن یعنی اگر سابِ اول کند باشد بقیه هم عقب می‌افتند
    let results: Vec<(String, Option<String>)> = std::thread::scope(|s| {
        let hs: Vec<_> = all.iter().map(|u| s.spawn(move || (u.clone(), fetch_text(u)))).collect();
        hs.into_iter().map(|h| h.join().unwrap_or((String::new(), None))).collect()
    });
    for (_, txt) in results {
        if let Some(txt) = txt {
            let dec = if txt.contains("://") { txt } else { b64_str(&txt) };
            for l in dec.lines() { push(l, &mut out, &mut seen); }
        }
    }
    out
}

// دو پورتِ آزادِ *متفاوت* (هر دو listener هم‌زمان باز می‌شوند تا تلاقی نشود) — باگِ قبلی این بود.
fn free_ports() -> (u16, u16) {
    let l1 = std::net::TcpListener::bind("127.0.0.1:0").expect("bind1");
    let l2 = std::net::TcpListener::bind("127.0.0.1:0").expect("bind2");
    let p1 = l1.local_addr().unwrap().port();
    let p2 = l2.local_addr().unwrap().port();
    drop(l1); drop(l2);
    (p1, p2)
}
fn port_up(port: u16, ms: u64) -> bool {
    let dl = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < dl {
        if TcpStream::connect_timeout(&format!("127.0.0.1:{port}").parse().unwrap(), Duration::from_millis(200)).is_ok() { return true; }
        std::thread::sleep(Duration::from_millis(120));
    }
    false
}

// تأییدِ واقعیِ اتصال از طریقِ پراکسیِ محلی (نه فقط «پورت باز است»).
// یک درخواستِ واقعی می‌زند؛ اگر جواب نداد یعنی سرور/کانفیگ مرده و نباید «متصل» بگوییم.
fn probe_proxy(port: u16) -> bool {
    let Ok(px) = ureq::Proxy::new(&format!("http://127.0.0.1:{port}")) else { return false };
    let agent = ureq::AgentBuilder::new().proxy(px).timeout(Duration::from_secs(8)).build();
    for url in ["http://www.gstatic.com/generate_204", "http://cp.cloudflare.com/generate_204"] {
        if let Ok(r) = agent.get(url).call() {
            if r.status() == 204 || r.status() == 200 { return true; }
        }
    }
    false
}
// ==========================================================================
//   نگهبانِ خروجی — «وصل شدم ولی از ایران بیرون نرفتم» را شکست حساب کن
//   WARP کاربر را به نزدیک‌ترین لبه‌ی کلادفلر می‌برد و لبه‌ی نزدیکِ ایران هم
//   داخلِ ایران است؛ آن‌وقت اپ می‌گوید «متصل» ولی هیچ فیلترشکنی‌ای رخ نداده.
//   این بدتر از وصل‌نشدن است، چون کاربر خیال می‌کند محافظت دارد.
// ==========================================================================
fn exit_cc_via(port: u16) -> String {
    let Ok(px) = ureq::Proxy::new(&format!("http://127.0.0.1:{port}")) else { return String::new() };
    let agent = ureq::AgentBuilder::new().proxy(px).timeout(Duration::from_secs(8)).build();
    for url in ["https://api.ip.sb/geoip", "http://ip-api.com/json/?fields=countryCode"] {
        let Ok(r) = agent.get(url).set("User-Agent", "Mozilla/5.0").call() else { continue };
        let Ok(txt) = r.into_string() else { continue };
        let Ok(j) = serde_json::from_str::<Value>(&txt) else { continue };
        let cc = j["country_code"].as_str().or_else(|| j["countryCode"].as_str()).unwrap_or("");
        if cc.len() == 2 { return cc.to_uppercase(); }
    }
    String::new()
}

// اگر خروجی داخلِ ایران باشد، این اتصال بی‌فایده است
fn exits_iran(port: u16) -> bool {
    let cc = exit_cc_via(port);
    if cc == "IR" { log("⚠️ خروجی داخلِ ایران است — این اتصال فیلترشکنی نمی‌کند"); return true; }
    if !cc.is_empty() { log(format!("خروجی: {cc}")); }
    false
}

// تأییدِ اتصالِ TUN (مستقیم — چون کلِ سیستم از تونل می‌رود)
fn probe_direct() -> bool {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(8)).build();
    for url in ["http://www.gstatic.com/generate_204", "http://cp.cloudflare.com/generate_204"] {
        if let Ok(r) = agent.get(url).call() {
            if r.status() == 204 || r.status() == 200 { return true; }
        }
    }
    false
}

pub fn ping_tcp(link: &str) -> Option<i32> {
    let (host, port) = host_port_of(link)?;
    let addr = format!("{host}:{port}");
    // اگر host دامنه باشد، to_socket_addrs آن را resolve می‌کند
    let mut addrs = std::net::ToSocketAddrs::to_socket_addrs(&addr).ok()?;
    let sa = addrs.next()?;
    TcpStream::connect_timeout(&sa, Duration::from_millis(1800)).ok()?;
    Some(0)
}

// تستِ واقعی: xray را بالا می‌آورد و یک درخواستِ واقعی از تونل می‌زند (نه فقط TCP).
// اول یک TCP سریع برای ردکردنِ مرده‌ها تا بی‌خود xray بالا نیاید.
pub fn real_ping(link: &str) -> Option<i32> {
    let singbox_only = is_singbox_only(link);
    // TCP prefilter فقط برای پروتکل‌های TCP؛ hy2/tuic روی UDP گوش می‌دهند (connectِ TCP بی‌معنی)
    if !singbox_only { ping_tcp(link)?; }
    let port = free_port();
    let (exe, cfg) = if is_wireguard(link) {
        (bin("sing-box.exe"), build_wg_config(link, port)?)
    } else if singbox_only {
        (bin("sing-box.exe"), build_singbox_proxy_config(link, port)?)
    } else {
        (bin("xray.exe"), build_config(link, port, false, false, 0)?)
    };
    let tmp = std::env::temp_dir().join(format!("sbtest_{}_{}.json", std::process::id(), port));
    std::fs::write(&tmp, serde_json::to_vec(&cfg).ok()?).ok()?;
    let mut child = Command::new(&exe).arg("run").arg("-c").arg(&tmp)
        .env("XRAY_LOCATION_ASSET", asset_dir())
        .creation_flags(NOWIN).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().ok()?;
    adopt(child.id());
    let result = (|| {
        // hy2/tuic کمی دیرتر بالا می‌آیند (دستِ‌دهیِ QUIC)
        if !port_up(port, if singbox_only { 3500 } else { 2200 }) { return None; }
        let agent = ureq::AgentBuilder::new()
            .proxy(ureq::Proxy::new(&format!("http://127.0.0.1:{port}")).ok()?)
            .timeout(Duration::from_secs(5)).build();
        // ۱) درخواستِ گرم‌کننده — هزینه‌ی یک‌بارِ راه‌اندازیِ هسته، DNS و هندشیکِ TLS با سرور
        //    را می‌پردازد. اگر این را بشماریم، عدد ۵ برابرِ واقعیت می‌شود (باگِ «۸۰۰ms»).
        if agent.get("http://www.gstatic.com/generate_204").call().is_err() { return None; }
        // ۲) اندازه‌گیریِ واقعی روی اتصالِ گرم. چون تست دانه‌دانه است و رقابتی روی
        //    CPU/پهنای‌باند نیست، یک نمونه کافی و دقیق است (و سریع‌تر هم هست).
        let t0 = Instant::now();
        match agent.get("http://www.gstatic.com/generate_204").call() {
            Ok(r) if r.status() == 204 || r.status() == 200 => Some(t0.elapsed().as_millis() as i32),
            _ => None,
        }
    })();
    let _ = child.kill();
    let _ = std::fs::remove_file(&tmp);
    result
}

// بهترین لینکِ شناخته‌شده برای «وصل‌شدن از سینی»:
// از فایلِ نتایجِ تست (که test_batch بعد از هر تست کامل ذخیره می‌کند) کمترین
// پینگِ زندهٔ اخیر را برمی‌گرداند. None یعنی هنوز تستی انجام نشده.
pub fn best_cached_link() -> Option<String> {
    let txt = std::fs::read_to_string(app_data_dir().join("last_results.json")).ok()?;
    let v: Value = serde_json::from_str(&txt).ok()?;
    let best = v["results"].as_array()?
        .iter()
        .filter_map(|r| {
            let link = r["link"].as_str()?;
            let ping = r["ping"].as_i64()?;
            if ping >= 0 { Some((ping, link.to_string())) } else { None }
        })
        .min_by_key(|(p, _)| *p)?;
    Some(best.1)
}

// ── موتورِ سریعِ تست: یک پروسهٔ هسته، N اینباند ──────────────────────────────
// ترفندِ v2rayN: به‌جای spawn کردنِ یک xray کامل برای هر کانفیگ (راه‌اندازیِ ~۱
// ثانیه + رمِ هر پروسه)، همهٔ کانفیگ‌های TCP-محور در *یک* پروسهٔ xray بالا
// می‌آیند؛ هر کانفیگ اینباندِ mixed خودش روی پورتِ جدا. sing-box-onlyها (hy2/tuic/wg)
// هم در یک پروسهٔ sing-box مشترک. نتیجه: شروعِ فاز۲ از ~۹۰×۱s به ~۱s می‌رسد.
fn build_multi_inbound_config(entries: &[(String, u16)]) -> Option<Value> {
    let mut outs = vec![json!({ "tag": "direct", "protocol": "freedom" })];
    let mut rules = vec![
        json!({ "type": "field", "network": "udp", "port": "443", "outboundTag": "block" }),
    ];
    let mut seen_tags = std::collections::HashSet::new();
    let mut inbounds = Vec::new();
    for (idx, (link, port)) in entries.iter().enumerate() {
        let Some(mut ob) = link_to_outbound(link) else { continue };
        ob["mux"] = json!({ "enabled": false, "concurrency": -1 });
        let tag = format!("t{idx}");
        if !seen_tags.insert(tag.clone()) { continue; }
        ob["tag"] = json!(tag.clone());
        outs.insert(outs.len().saturating_sub(2), ob);   // قبل از direct/block بماند
        inbounds.push(json!({
            "tag": tag, "listen": "127.0.0.1", "port": port, "protocol": "mixed",
            "settings": { "auth": "noauth", "udp": false },
            "sniffing": { "enabled": false }
        }));
        rules.push(json!({ "type": "field", "inboundTag": [tag], "outboundTag": format!("t{idx}") }));
    }
    if inbounds.is_empty() { return None; }
    Some(json!({
        "log": { "loglevel": "error" },
        "dns": { "servers": ["https://1.1.1.1/dns-query", "1.1.1.1"] },
        "inbounds": inbounds,
        "outbounds": outs,
        "routing": { "domainStrategy": "AsIs", "rules": rules }
    }))
}

fn build_multi_singbox_config(entries: &[(String, u16)]) -> Option<Value> {
    let mut outbounds = vec![json!({ "type": "direct", "tag": "direct" })];
    let mut rules = vec![
        json!({ "protocol": "dns", "action": "hijack-dns" }),
    ];
    let mut inbounds = Vec::new();
    for (idx, (link, port)) in entries.iter().enumerate() {
        let Some(mut ob) = singbox_outbound(link) else { continue };
        ob["tag"] = json!(format!("t{idx}"));
        outbounds.insert(outbounds.len() - 1, ob);
        inbounds.push(json!({ "type": "mixed", "tag": format!("in{idx}"), "listen": "127.0.0.1", "listen_port": port }));
        rules.push(json!({ "inbound": [format!("in{idx}")], "outbound": format!("t{idx}") }));
    }
    if inbounds.is_empty() { return None; }
    Some(json!({
        "log": { "level": "error" },
        "dns": { "servers": [ { "tag": "l", "type": "local" } ], "final": "l" },
        "inbounds": inbounds,
        "outbounds": outbounds,
        "route": { "final": "direct", "default_domain_resolver": { "server": "l" }, "rules": rules }
    }))
}

// پینگ از داخلِ پروسهٔ مشترک: گرم‌کن + اندازه‌گیری (همان منطقِ real_ping)
fn ping_via_shared(port: u16) -> Option<i32> {
    if !port_up(port, 1500) { return None; }
    let Ok(px) = ureq::Proxy::new(&format!("http://127.0.0.1:{port}")) else { return None };
    let agent = ureq::AgentBuilder::new().proxy(px).timeout(Duration::from_secs(5)).build();
    if agent.get("http://www.gstatic.com/generate_204").call().is_err() { return None; }
    let t0 = Instant::now();
    match agent.get("http://www.gstatic.com/generate_204").call() {
        Ok(r) if r.status() == 204 || r.status() == 200 => Some(t0.elapsed().as_millis() as i32),
        _ => None,
    }
}

// تستِ دسته‌ای: اول TCP موازی (سریع، مرده‌ها کنار)، بعد تستِ واقعیِ فقط زنده‌ها (سقف‌دار).
// `on(index, ping)` بعد از هر کانفیگ صدا زده می‌شود تا رابط زنده به‌روز شود.
fn test_running() -> &'static std::sync::atomic::AtomicBool {
    static R: std::sync::OnceLock<std::sync::atomic::AtomicBool> = std::sync::OnceLock::new();
    R.get_or_init(|| std::sync::atomic::AtomicBool::new(false))
}
pub fn test_batch<F: Fn(usize, i32) + Send + Sync + 'static>(links: &[String], on: F) -> Vec<i32> {
    use std::sync::atomic::Ordering;
    // ورودیِ دوم ممنوع — پروسهٔ مشترک و اسلاتِ sing-box فقط یک‌جا قابل مدیریت‌اند
    if test_running().swap(true, Ordering::SeqCst) {
        log("تست دیگری در جریان است");
        return vec![-1; links.len()];
    }
    let out = test_batch_inner(links, on);
    test_running().store(false, Ordering::SeqCst);
    out
}
fn test_batch_inner<F: Fn(usize, i32) + Send + Sync + 'static>(links: &[String], on: F) -> Vec<i32> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let n = links.len();
    if n == 0 { return vec![]; }
    log(format!("شروعِ تستِ {} سرور (TCP + هستهٔ مشترک)…", n));
    let links = Arc::new(links.to_vec());
    let res = Arc::new(Mutex::new(vec![-1i32; n]));

    // فاز ۱: TCP موازی
    let alive = Arc::new(Mutex::new(vec![false; n]));
    {
        let idx = Arc::new(AtomicUsize::new(0));
        let mut hs = vec![];
        for _ in 0..32.min(n) {
            let (alive, idx, links) = (alive.clone(), idx.clone(), links.clone());
            hs.push(std::thread::spawn(move || loop {
                let i = idx.fetch_add(1, Ordering::SeqCst);
                if i >= links.len() { break; }
                // hy2/tuic روی UDP هستند؛ TCP prefilter ردشان می‌کند → مستقیم زنده فرض کن (فاز۲ واقعی تست می‌کند)
                if is_singbox_only(&links[i]) || ping_tcp(&links[i]).is_some() { alive.lock().unwrap()[i] = true; }
            }));
        }
        for h in hs { let _ = h.join(); }
    }
    let alive_flags = alive.lock().unwrap().clone();
    let mut alive_idx: Vec<usize> = (0..n).filter(|&i| alive_flags[i]).collect();
    // چون تست دانه‌دانه است، سقف را واقع‌بینانه می‌گیریم (وگرنه انتظار طولانی می‌شود).
    // رابط قبلش با manifest فیلترِ «مناسبِ نتِ تو» زده، پس این‌ها بهترین کاندیداهایند.
    alive_idx.truncate(90);
    // مرده‌های TCP را فوراً منتشر کن (‎-1‎) تا کشورِ همه‌مرده زودتر غیب شود
    let alive_set: std::collections::HashSet<usize> = alive_idx.iter().copied().collect();
    for i in 0..n { if !alive_set.contains(&i) { on(i, -1); } }

    let my_gen = cur_gen();

    // ── فاز ۲: دو پروسهٔ مشترک (xray برای TCP-محورها + sing-box برای hy2/tuic/wg) ──
    // هر کانفیگ اینباندِ اختصاصی روی پورتِ جدا دارد → تستِ موازی بدونِ ۹۰ پروسه.
    let xray_entries: Vec<(usize, String)> = {
        let mut v: Vec<(usize, String)> = vec![];
        for &i in &alive_idx {
            if is_wireguard(&links[i]) || is_singbox_only(&links[i]) { continue; }
            v.push((i, links[i].clone()));
        }
        v
    };
    let sb_entries: Vec<(usize, String)> = {
        let mut v = vec![];
        for &i in &alive_idx {
            if is_wireguard(&links[i]) || is_singbox_only(&links[i]) { v.push((i, links[i].clone())); }
        }
        v
    };

    // پورت‌های اختصاصی هر کانفیگ (همه از قبل bind می‌شوند تا تلاقی نشود)
    fn alloc_ports(k: usize) -> Vec<u16> {
        let mut listeners = Vec::with_capacity(k);
        let mut ports = Vec::with_capacity(k);
        for _ in 0..k {
            match std::net::TcpListener::bind("127.0.0.1:0") {
                Ok(l) => { ports.push(l.local_addr().map(|a| a.port()).unwrap_or(0)); listeners.push(l); }
                Err(_) => { ports.push(0); listeners.pop(); }   // بدونِ unwrap — اتمامِ سوکت نباید panic کند
            }
        }
        drop(listeners);
        ports
    }

    // ── xray مشترک (کانفیگ‌های TCP-محور) ──
    let xr_ports = alloc_ports(xray_entries.len());
    let xr_pairs: Vec<(String, u16)> = xray_entries.iter().cloned()
        .map(|(_, l)| l).zip(xr_ports.iter().copied()).collect();
    let xr_pairs: Vec<(String, u16)> = xr_pairs.into_iter().filter(|(_, p)| *p > 0).collect();
    let mut shared_child: Option<std::process::Child> = None;
    let mut xr_cfg_file: Option<PathBuf> = None;
    if !xr_pairs.is_empty() {
        let entries: Vec<(String, u16)> = xr_pairs.clone();
        if let Some(cfg) = build_multi_inbound_config(&entries) {
            let tmp = std::env::temp_dir().join(format!("sbmulti_xray_{}.json", std::process::id()));
            if std::fs::write(&tmp, serde_json::to_vec(&cfg).unwrap_or_default()).is_ok() {
                if let Ok(c) = Command::new(bin("xray.exe")).arg("run").arg("-c").arg(&tmp)
                    .env("XRAY_LOCATION_ASSET", asset_dir())
                    .creation_flags(NOWIN).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
                    .spawn() {
                    adopt(c.id());
                    shared_child = Some(c);
                    xr_cfg_file = Some(tmp);
                    // صبر تا همهٔ اینباندها بالا بیایند (یک بار، نه per-config)
                    let dl = Instant::now() + Duration::from_secs(6);
                    while Instant::now() < dl && xr_ports.iter().any(|p| *p > 0 && !port_up(*p, 50)) {
                        if cur_gen() != my_gen { break; }
                        std::thread::sleep(Duration::from_millis(120));
                    }
                }
            }
        }
    }

    // نتایجِ فاز۲: نگاشتِ index کانفیگ → پورتِ اختصاصی‌اش در پروسهٔ مشترک
    let mut port_of: HashMap<usize, u16> = HashMap::new();
    for ((gi, _), p) in xray_entries.iter().zip(xr_ports.iter()) {
        if *p > 0 { port_of.insert(*gi, *p); }
    }

    // worker pool روی همهٔ کانفیگ‌های زنده (هر دو نوع) با ۶ ترد
    {
        struct Job { gi: usize, port: u16 }
        let mut jobs: Vec<Job> = vec![];
        for (gi, p) in xray_entries.iter().zip(xr_ports.iter()).map(|((gi, _), p)| (*gi, p)) {
            if *p > 0 { jobs.push(Job { gi, port: *p }); }
        }
        // sing-box مشترک برای hy2/tuic/wg
        let sb_ports = alloc_ports(sb_entries.len());
        let sb_ok: Vec<(usize, u16)> = sb_entries.iter().zip(sb_ports.iter())
            .filter(|((_, _), p)| **p > 0).map(|((gi, _), p)| (*gi, *p)).collect();
        if !sb_ok.is_empty() {
            let entries: Vec<(String, u16)> = sb_entries.iter().zip(sb_ports.iter())
                .filter(|(_, p)| **p > 0).map(|((_, l), p)| (l.clone(), *p)).collect();
            if let Some(cfg) = build_multi_singbox_config(&entries) {
                let tmp = std::env::temp_dir().join(format!("sbmulti_sb_{}.json", std::process::id()));
                if std::fs::write(&tmp, serde_json::to_vec(&cfg).unwrap_or_default()).is_ok() {
                    if let Ok(c) = Command::new(bin("sing-box.exe")).arg("run").arg("-c").arg(&tmp)
                        .creation_flags(NOWIN).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
                        .spawn() {
                        adopt(c.id());
                        // QUIC handshake دیرتر جواب می‌دهد؛ کمی بیشتر صبر
                        let dl = Instant::now() + Duration::from_secs(8);
                        while Instant::now() < dl && sb_ok.iter().any(|(_, p)| !port_up(*p, 50)) {
                            if cur_gen() != my_gen { break; }
                            std::thread::sleep(Duration::from_millis(150));
                        }
                        // پروسه را زنده نگه دار تا پایانِ تست (در اسکوپِ بعدی kill می‌شود)
                        for (gi, p) in sb_ok { jobs.push(Job { gi, port: p }); }
                        // ذخیره برای kill در پایان
                        sb_shared_child().lock().unwrap().replace(c);
                        shared_cfg_file().lock().unwrap().replace(tmp);
                    }
                }
            }
        }

        let jobs = Arc::new(Mutex::new(jobs));
        let on = Arc::new(on);   // closure بینِ workerها share شود (Clone لازم نیست)
        let mut workers = vec![];
        for _ in 0..6.min(jobs.lock().unwrap().len().max(1)) {
            let (jobs, res, on) = (jobs.clone(), res.clone(), on.clone());
            workers.push(std::thread::spawn(move || loop {
                if cur_gen() != my_gen { break; }
                let job = { let mut j = jobs.lock().unwrap(); j.pop() };
                let Some(job) = job else { break };
                let p = ping_via_shared(job.port).unwrap_or(-1);
                res.lock().unwrap()[job.gi] = p;
                on(job.gi, p);
            }));
        }
        for w in workers { let _ = w.join(); }
        // پاکسازیِ sing-box مشترک
        if let Some(mut c) = sb_shared_child().lock().unwrap().take() { let _ = c.kill(); }
        if let Some(f) = shared_cfg_file().lock().unwrap().take() { let _ = std::fs::remove_file(f); }
    }
    if let Some(mut c) = shared_child.take() { let _ = c.kill(); }
    if let Some(f) = xr_cfg_file.take() { let _ = std::fs::remove_file(f); }

    if cur_gen() != my_gen { log("تست کنسل شد"); }
    let out = res.lock().unwrap().clone();
    let working = out.iter().filter(|&&p| p >= 0).count();
    log(format!("تست تمام شد — {} سرورِ واقعاً کارآمد پیدا شد", working));
    // ذخیرهٔ نتایج برای «وصل‌شدن از سینی» (best_cached_link)
    let results: Vec<Value> = links.iter().zip(out.iter())
        .map(|(l, p)| json!({ "link": l, "ping": p })).collect();
    let _ = std::fs::write(app_data_dir().join("last_results.json"),
        serde_json::to_vec(&json!({ "at": chrono_now(), "results": results })).unwrap_or_default());
    out
}

fn free_port() -> u16 { free_ports().0 }

fn asset_dir() -> PathBuf {
    bin("xray.exe").parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
}

// ---------- کانفیگِ xray (سبکِ v2rayN: تک‌پورتِ mixed + dns + بلاکِ QUIC) ----------
fn build_config(link: &str, port: u16, fragment: bool, bypass_iran: bool, metrics_port: u16) -> Option<Value> {
    let mut ob = link_to_outbound(link)?;
    ob["mux"] = json!({ "enabled": false, "concurrency": -1 });
    let mut extra = vec![];
    if fragment {
        // ضدِ DPI قوی‌تر: تکه‌های کوچک‌ترِ ClientHello (۱۰–۴۰ بایت) با فاصله‌ی کم‌تر،
        // به‌علاوه‌ی «نویزِ» UDP — الگوی TLS را برای DPI ناخوانا می‌کند. علی گزارش داد
        // با Fragment سرعتش بهتر می‌شود (یعنی ISP دارد شکل‌دهی/throttle می‌کند و این دورش می‌زند).
        ob["streamSettings"]["sockopt"] = json!({ "dialerProxy": "fragment" });
        extra.push(json!({ "tag": "fragment", "protocol": "freedom",
            "settings": {
                "fragment": { "packets": "tlshello", "length": "10-40", "interval": "5-15" },
                "noises": [ { "type": "rand", "packet": "10-20", "delay": "10-16" } ]
            } }));
    }
    let mut outs = vec![ob];
    outs.extend(extra);
    outs.push(json!({ "tag": "direct", "protocol": "freedom" }));
    outs.push(json!({ "tag": "block", "protocol": "blackhole" }));
    // قوانینِ مسیریابی
    let mut rules = vec![
        // بلاکِ QUIC (UDP 443) تا مرورگر برود روی TCP — کلیدِ بازشدنِ یوتوب/گوگل
        json!({ "type": "field", "network": "udp", "port": "443", "outboundTag": "block" }),
    ];
    if bypass_iran {
        // IPهای ایران مستقیم بروند (سریع‌تر + سرور لو نرود)
        rules.push(json!({ "type": "field", "ip": ["geoip:ir", "geoip:private"], "outboundTag": "direct" }));
    }
    // AsIs (مثلِ v2rayN) برای حالتِ عادی = مطمئن‌تر (سرور خودش دامنه را resolve می‌کند).
    // فقط برای بایپسِ ایران به IPIfNonMatch نیاز داریم تا geoip:ir را روی IP تشخیص دهد.
    let domain_strategy = if bypass_iran { "IPIfNonMatch" } else { "AsIs" };
    // ترافیکِ ورودیِ آمار نباید از تونل برود — به خودِ سرویسِ metrics می‌رسد
    if metrics_port > 0 {
        rules.insert(0, json!({ "type": "field", "inboundTag": ["metrics_in"], "outboundTag": "metrics_in" }));
    }
    let mut inbounds = vec![json!({
        "tag": "in", "listen": "127.0.0.1", "port": port, "protocol": "mixed",
        "settings": { "auth": "noauth", "udp": true, "allowTransparent": false },
        "sniffing": { "enabled": true, "destOverride": ["http", "tls", "quic"], "routeOnly": false }
    })];
    if metrics_port > 0 {
        inbounds.push(json!({ "tag": "metrics_in", "listen": "127.0.0.1", "port": metrics_port,
                              "protocol": "dokodemo-door", "settings": { "address": "127.0.0.1" } }));
    }
    Some(json!({
        // لاگِ دسترسی: برای «فعالیتِ اینترنت» (چه سایتی باز می‌شود). فقط محلی و
        // موقتی است و موقعِ قطع پاک می‌شود.
        "log": { "loglevel": "warning", "access": access_log_path().to_string_lossy() },
        "dns": { "servers": ["https://1.1.1.1/dns-query", "1.1.1.1", "8.8.8.8"] },
        "inbounds": inbounds,
        "outbounds": outs,
        "routing": { "domainStrategy": domain_strategy, "rules": rules },
        // آمارِ مصرفِ داده: xray یک endpointِ سبکِ HTTP می‌دهد (/debug/vars) — بدونِ gRPC.
        // این همان چیزی است که «مصرفِ داده»ی پایینِ صفحه را واقعی می‌کند.
        "stats": {},
        "metrics": { "tag": "metrics_in" },
        "policy": { "system": { "statsOutboundUplink": true, "statsOutboundDownlink": true } }
    }))
}

// ==========================================================================
//   TUN / حالتِ گیم  (sing-box + WinTun) — کلِ ترافیکِ سیستم از تونل، نه فقط مرورگر
//   معماری: خودِ sing-box کانفیگ را native می‌گیرد (نه tun→socks→xray) تا حلقه‌ی
//   routing نسازد؛ sing-box آدرسِ سرور را خودش bypass می‌کند. نیاز به دسترسیِ ادمین.
// ==========================================================================

// TLS/uTLS برای sing-box از پارامترهای لینک
fn sb_tls(sec: &str, p: &HashMap<String, String>, sni_default: &str) -> Option<Value> {
    let sec = sec.to_lowercase();
    if !(sec == "tls" || sec == "xtls" || sec == "reality") { return None; }
    let fp = if g(p, "fp").is_empty() { "chrome" } else { g(p, "fp") };
    let sni = if !g(p, "sni").is_empty() { g(p, "sni") }
        else if !g(p, "host").is_empty() { g(p, "host") } else { sni_default };
    let mut tls = json!({ "enabled": true, "utls": { "enabled": true, "fingerprint": fp } });
    if !sni.is_empty() { tls["server_name"] = json!(sni); }
    if !g(p, "alpn").is_empty() && sec != "reality" {
        tls["alpn"] = json!(g(p, "alpn").split(',').collect::<Vec<_>>());
    }
    if g(p, "allowInsecure") == "1" || g(p, "insecure") == "1" { tls["insecure"] = json!(true); }
    if sec == "reality" {
        let mut r = json!({ "enabled": true });
        if !g(p, "pbk").is_empty() { r["public_key"] = json!(g(p, "pbk")); }
        if !g(p, "sid").is_empty() { r["short_id"] = json!(g(p, "sid")); }
        tls["reality"] = r;
    }
    // ECH — Encrypted Client Hello: SNI از دیدِ ISP/DPI رمز می‌شود (sing-box دارد؛
    // کلید از DNS یا پارامترِ ech= لینک). اگر سرور ECH نداشته باشد خودش fallback می‌کند.
    if g(p, "ech") == "1" || !g(p, "echconfig").is_empty() {
        let mut e = json!({ "enabled": true });
        let ec = g(p, "echconfig");
        if !ec.is_empty() { e["config"] = json!(ec); }
        tls["ech"] = e;
    }
    Some(tls)
}

fn sb_transport(net: &str, p: &HashMap<String, String>, host_default: &str) -> Option<Value> {
    match net {
        "ws" => {
            let path = if g(p, "path").is_empty() { "/" } else { g(p, "path") };
            let mut w = json!({ "type": "ws", "path": path });
            let h = if !g(p, "host").is_empty() { g(p, "host") } else { host_default };
            if !h.is_empty() { w["headers"] = json!({ "Host": h }); }
            Some(w)
        }
        "grpc" => {
            let sn = if !g(p, "serviceName").is_empty() { g(p, "serviceName") } else { g(p, "path") };
            Some(json!({ "type": "grpc", "service_name": sn }))
        }
        "h2" | "http" => {
            let mut hh = json!({ "type": "http" });
            if !g(p, "path").is_empty() { hh["path"] = json!(g(p, "path")); }
            let h = if !g(p, "host").is_empty() { g(p, "host") } else { host_default };
            if !h.is_empty() { hh["host"] = json!([h]); }
            Some(hh)
        }
        // ⚠️ sing-box 1.13 ترنسپورتِ xhttp ندارد (تست شد) — لینک‌های XHTTP باید با
        // هستهٔ xray بروند؛ اینجا None برمی‌گردانیم تا مسیر TUN پیام مناسب بدهد.
        // (در حالت پراکسی، xray کامل پشتیبانی دارد.)
        _ => None, // tcp = بدونِ transport
    }
}

// لینک → outboundِ sing-box (موازیِ link_to_outbound ولی فرمتِ sing-box)
pub fn singbox_outbound(link: &str) -> Option<Value> {
    let low = link.to_lowercase();
    // XHTTP فقط در xray پشتیبانی می‌شود (sing-box 1.13 ندارد — تست شد).
    // None → TUN پیامِ «این کانفیگ برای TUN پشتیبانی نمی‌شود» می‌دهد؛ پراکسی با xray کار می‌کند.
    if low.contains("type=xhttp") || low.contains("type=splithttp") { return None; }
    if low.starts_with("vmess://") {
        let j: Value = serde_json::from_str(&b64_str(&link[8..])).ok()?;
        let add = j.get("add")?.as_str()?.to_string();
        let port: u64 = j.get("port").and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))?;
        if add.is_empty() || port == 0 { return None; }
        let net = j.get("net").and_then(|v| v.as_str()).unwrap_or("tcp");
        let tlsv = j.get("tls").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        let sec = if tlsv == "tls" || tlsv == "reality" { "tls" } else { "none" };
        let gs = |k: &str| j.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut p = HashMap::new();
        for (a, b) in [("path", gs("path")), ("host", gs("host")), ("sni", gs("sni")),
            ("serviceName", gs("path")), ("alpn", gs("alpn")), ("fp", gs("fp"))] { p.insert(a.to_string(), b); }
        let aid: u64 = j.get("aid").and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0);
        let scy = gs("scy"); let scy = if scy.is_empty() { "auto".into() } else { scy };
        let mut o = json!({ "type": "vmess", "tag": "proxy", "server": add, "server_port": port,
            "uuid": j.get("id")?.as_str()?, "alter_id": aid, "security": scy, "packet_encoding": "xudp" });
        if let Some(t) = sb_tls(sec, &p, &add) { o["tls"] = t; }
        if let Some(tr) = sb_transport(net, &p, &add) { o["transport"] = tr; }
        return Some(o);
    }
    if low.starts_with("vless://") {
        let u = url::Url::parse(link).ok()?;
        let uid = u.username(); let add = u.host_str()?.to_string(); let port = u.port()?;
        if uid.is_empty() { return None; }
        let p = parse_qs(u.query().unwrap_or(""));
        let net = if g(&p, "type").is_empty() { "tcp" } else { g(&p, "type") };
        let sec = if g(&p, "security").is_empty() { "none" } else { g(&p, "security") };
        // packet_encoding=xudp: UDPِ بازی/برنامه‌ها را بهینه از پراکسی رد می‌کند (full-cone NAT، کم‌تأخیر)
        let mut o = json!({ "type": "vless", "tag": "proxy", "server": add, "server_port": port, "uuid": uid, "packet_encoding": "xudp" });
        // flow (xtls-rprx-vision) فقط با tcp/reality معتبر است، با ws/grpc نه
        if !g(&p, "flow").is_empty() && (net == "tcp" || net == "raw") { o["flow"] = json!(g(&p, "flow")); }
        if let Some(t) = sb_tls(sec, &p, &add) { o["tls"] = t; }
        if let Some(tr) = sb_transport(net, &p, &add) { o["transport"] = tr; }
        return Some(o);
    }
    if low.starts_with("trojan://") {
        let u = url::Url::parse(link).ok()?;
        let pwd = urlencoding::decode(u.username()).map(|c| c.into_owned()).unwrap_or_default();
        let add = u.host_str()?.to_string(); let port = u.port()?;
        if pwd.is_empty() { return None; }
        let p = parse_qs(u.query().unwrap_or(""));
        let net = if g(&p, "type").is_empty() { "tcp" } else { g(&p, "type") };
        let sec = if g(&p, "security").is_empty() { "tls" } else { g(&p, "security") };
        let mut o = json!({ "type": "trojan", "tag": "proxy", "server": add, "server_port": port, "password": pwd });
        if let Some(t) = sb_tls(sec, &p, &add) { o["tls"] = t; }
        if let Some(tr) = sb_transport(net, &p, &add) { o["transport"] = tr; }
        return Some(o);
    }
    if low.starts_with("ss://") {
        let ob = link_to_outbound(link)?; // پارسِ ss را دوباره ننویس، از همان استفاده کن
        let s = &ob["settings"]["servers"][0];
        return Some(json!({ "type": "shadowsocks", "tag": "proxy",
            "server": s["address"], "server_port": s["port"], "method": s["method"], "password": s["password"] }));
    }
    // Hysteria2 (hy2/hysteria2) — مبتنی بر QUIC/UDP: بهترین برای بازی (کنترلِ ازدحام، جبرانِ پکت‌لاس)
    if low.starts_with("hysteria2://") || low.starts_with("hy2://") {
        let u = url::Url::parse(link).ok()?;
        let pwd = urlencoding::decode(u.username()).map(|c| c.into_owned()).unwrap_or_default();
        let add = u.host_str()?.to_string(); let port = u.port()?;
        if add.is_empty() { return None; }
        let p = parse_qs(u.query().unwrap_or(""));
        let sni = if !g(&p, "sni").is_empty() { g(&p, "sni") } else { &add };
        let mut tls = json!({ "enabled": true, "server_name": sni });
        if g(&p, "insecure") == "1" || g(&p, "allowInsecure") == "1" { tls["insecure"] = json!(true); }
        if !g(&p, "alpn").is_empty() { tls["alpn"] = json!(g(&p, "alpn").split(',').collect::<Vec<_>>()); }
        let mut o = json!({ "type": "hysteria2", "tag": "proxy", "server": add, "server_port": port, "password": pwd, "tls": tls });
        // obfs (salamander) اگر بود
        if g(&p, "obfs").eq_ignore_ascii_case("salamander") && !g(&p, "obfs-password").is_empty() {
            o["obfs"] = json!({ "type": "salamander", "password": g(&p, "obfs-password") });
        }
        return Some(o);
    }
    // TUIC — مبتنی بر QUIC/UDP: عالی برای بازی (BBR + رله‌ی native UDP)
    if low.starts_with("tuic://") {
        let u = url::Url::parse(link).ok()?;
        let uuid = urlencoding::decode(u.username()).map(|c| c.into_owned()).unwrap_or_default();
        let pwd = urlencoding::decode(u.password().unwrap_or("")).map(|c| c.into_owned()).unwrap_or_default();
        let add = u.host_str()?.to_string(); let port = u.port()?;
        if uuid.is_empty() { return None; }
        let p = parse_qs(u.query().unwrap_or(""));
        let sni = if !g(&p, "sni").is_empty() { g(&p, "sni") } else { &add };
        let alpn: Vec<&str> = if g(&p, "alpn").is_empty() { vec!["h3"] } else { g(&p, "alpn").split(',').collect() };
        let mut tls = json!({ "enabled": true, "server_name": sni, "alpn": alpn });
        if g(&p, "allow_insecure") == "1" || g(&p, "insecure") == "1" { tls["insecure"] = json!(true); }
        let cc = if g(&p, "congestion_control").is_empty() { "bbr" } else { g(&p, "congestion_control") };
        let mode = if g(&p, "udp_relay_mode").is_empty() { "native" } else { g(&p, "udp_relay_mode") };
        return Some(json!({ "type": "tuic", "tag": "proxy", "server": add, "server_port": port,
            "uuid": uuid, "password": pwd, "congestion_control": cc, "udp_relay_mode": mode, "tls": tls }));
    }
    // AnyTLS — ضدِ اثرِ انگشتِ TLS-in-TLS (پدینگ + چندپlex). sing-box client دارد.
    // فرمتِ لینک: anytls://password@host:port?sni=...&insecure=1#name
    if low.starts_with("anytls://") {
        let u = url::Url::parse(link).ok()?;
        let pwd = urlencoding::decode(u.username()).map(|c| c.into_owned()).unwrap_or_default();
        let add = u.host_str()?.to_string(); let port = u.port()?;
        if pwd.is_empty() || add.is_empty() { return None; }
        let p = parse_qs(u.query().unwrap_or(""));
        let sni = if !g(&p, "sni").is_empty() { g(&p, "sni") } else { &add };
        let mut o = json!({ "type": "anytls", "tag": "proxy", "server": add, "server_port": port,
            "password": pwd });
        // anytls بدونِ TLS رد می‌شود («TLS required») — همیشه TLS روشن است
        let mut tls = json!({ "enabled": true, "server_name": sni });
        if g(&p, "insecure") == "1" || g(&p, "allowInsecure") == "1" { tls["insecure"] = json!(true); }
        o["tls"] = tls;
        return Some(o);
    }
    None
}

// پروتکل‌هایی که فقط sing-box می‌فهمد (xray نه) — QUIC/UDP-محور، عالی برای بازی
fn is_singbox_only(link: &str) -> bool {
    let l = link.to_lowercase();
    l.starts_with("hysteria2://") || l.starts_with("hy2://") || l.starts_with("tuic://")
        || l.starts_with("wg://") || l.starts_with("anytls://")
}

// کانفیگِ پراکسیِ محلیِ sing-box (mixed inbound، بدونِ TUN، بدونِ ادمین) — برای وصل/تستِ hy2/tuic
fn build_singbox_proxy_config(link: &str, port: u16) -> Option<Value> {
    let ob = singbox_outbound(link)?;
    Some(json!({
        "log": { "level": "warn" },
        "dns": {
            "servers": [
                { "tag": "proxy-dns", "type": "https", "server": "1.1.1.1", "detour": "proxy" },
                { "tag": "local-dns", "type": "local" }
            ],
            "final": "proxy-dns", "strategy": "prefer_ipv4"
        },
        "inbounds": [{ "type": "mixed", "tag": "in", "listen": "127.0.0.1", "listen_port": port }],
        "outbounds": [ ob, { "type": "direct", "tag": "direct" } ],
        "route": {
            "final": "proxy", "default_domain_resolver": { "server": "local-dns" },
            "rules": [
                { "protocol": "dns", "action": "hijack-dns" },
                { "network": "udp", "port": 443, "action": "reject" }
            ]
        }
    }))
}

// کانفیگِ TUN برای sing-box 1.13 (اسکیمای جدید: dns type-based + default_domain_resolver)
// قواعدِ per-app (پرفراپ): برنامه → proxy / direct / block.
// فرمتِ ذخیره: هر قاعده «<مقدار>:<mode>:<kind>» است:
//   kind = n → process_name (chrome.exe)   |   kind = p → process_path_regex
//   mode = p/d/b  (proxy/direct/block). «سیستم» یعنی بدونِ قاعده — ذخیره نمی‌شود.
fn app_rule_entries(specs: &[String]) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    fn escape_re(s: &str) -> String {
        s.chars().map(|c| if "\\.^$|?*+()[]{}".contains(c) { format!("\\{c}") } else { c.to_string() }).collect()
    }
    let (mut px, mut dr, mut bl) = (vec![], vec![], vec![]);
    for s in specs {
        // فرمت‌های قدیمی هم قبول باشند: "chrome.exe:p"
        let parts: Vec<&str> = s.split(':').collect();
        let (val, mode, kind) = match parts.as_slice() {
            [v, m] => (v.trim(), *m, "n"),
            [v, m, k] => (v.trim(), *m, *k),
            _ => continue,
        };
        if val.is_empty() { continue; }
        let rule = if kind == "p" {
            // مسیرِ کامل → regex با case-insensitive (UWPها مسیرشان WindowsApps است)
            json!({ "process_path_regex": [format!("(?i){}", escape_re(val))] })
        } else {
            let name = val.to_lowercase();
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-') { continue; }
            json!({ "process_name": [name] })
        };
        match mode {
            "p" => px.push(rule),
            "d" => dr.push(rule),
            "b" => bl.push(rule),
            _ => {}
        }
    }
    (px, dr, bl)
}

fn build_tun_config(link: &str, game: bool, bypass_iran: bool) -> Option<Value> {
    build_tun_config_apps(link, game, bypass_iran, &[])
}

fn build_tun_config_apps(link: &str, game: bool, bypass_iran: bool, apps: &[String]) -> Option<Value> {
    let ob = singbox_outbound(link)?;
    // گیم: استکِ system سریع‌تر و کم‌تأخیرتر است + MTU کمی پایین‌تر (کاهشِ فرگمنت/جیتر)
    let stack = if game { "system" } else { "gvisor" };
    let mtu = if game { 1400 } else { 1500 };
    let mut rules = vec![
        json!({ "action": "sniff" }),
        json!({ "protocol": "dns", "action": "hijack-dns" }),
        json!({ "ip_is_private": true, "outbound": "direct" }),
        // پخشِ محلی/چندپخشی هم مستقیم (وگرنه بعضی برنامه‌ها گیر می‌کنند)
        json!({ "ip_cidr": ["224.0.0.0/3", "255.255.255.255/32"], "outbound": "direct" }),
    ];
    // ── پرفراپ: قواعدِ per-app قبل از بقیه (اولویت دارند) ──
    // هر قاعده یک rule کامل است (process_name یا process_path_regex برای UWP).
    let (mut px, mut dr, mut bl) = app_rule_entries(apps);
    if !bl.is_empty() { rules.append(&mut bl); }
    if !px.is_empty() { rules.append(&mut px); }
    if !dr.is_empty() {
        rules.append(&mut dr);
        // DNSِ برنامه‌های direct هم باید مستقیم resolve شود وگرنه IP تونل میگیرند
        let names: Vec<String> = apps.iter().filter_map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            match parts.as_slice() {
                [v, "d"] | [v, "d", "n"] => Some(v.trim().to_lowercase()),
                _ => None,
            }
        }).collect();
        if !names.is_empty() {
            rules.push(json!({ "process_name": names, "protocol": "dns", "action": "hijack-dns" }));
        }
    }
    // بایپسِ ایران: سایت‌های ایرانی مستقیم بروند (سریع‌تر + نیم‌بها + سرور لو نرود)
    if bypass_iran {
        rules.push(json!({ "rule_set": "geoip-ir", "outbound": "direct" }));
    }
    // ⚠️ اینجا QUIC را بلاک نمی‌کنیم.
    // در حالتِ پراکسی بلاکِ UDP/443 درست است (مرورگر خودش به TCP برمی‌گردد)، ولی در
    // TUN کلِ سیستم رد می‌شود و برنامه‌هایی مثلِ دیسکورد فالبک ندارند → قطع می‌شوند.
    // (علی گزارش داد: با TUN دیسکورد باز نمی‌شود — دلیلش همین بود.)
    // منبعِ IPهای ایران (فقط وقتی بایپس روشن است) — از مخزنِ رسمیِ sing-box، کش‌شده
    let rule_sets = if bypass_iran {
        json!([{ "type": "remote", "tag": "geoip-ir", "format": "binary",
                 "url": "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-ir.srs",
                 "download_detour": "proxy" }])
    } else { json!([]) };
    Some(json!({
        "log": { "level": "warn" },
        "dns": {
            "servers": [
                { "tag": "proxy-dns", "type": "https", "server": "1.1.1.1", "detour": "proxy" },
                { "tag": "local-dns", "type": "local" }
            ],
            // فقط v4: رکوردهای AAAA باعث می‌شد برنامه‌ها به IPv6ِ خارجِ تونل برسند
            "final": "proxy-dns", "strategy": "ipv4_only"
        },
        "inbounds": [{
            "type": "tun", "tag": "tun-in",
            // ⚠️ هر دو خانواده: اگر فقط v4 بدهیم، ترافیکِ IPv6 از کارتِ واقعی
            // می‌رود → گوگل دو IP مختلف می‌بیند («unusual traffic … 5.x ≠ 51.x»)
            // و کپچا/بلاک می‌اندازد. با v6 روی تونل، همه‌چیز از یک خروجی می‌رود.
            "address": ["172.19.0.1/30", "fdfe:dcba:9876::1/126"],
            "mtu": mtu, "auto_route": true, "strict_route": true, "stack": stack,
            // strict_route=true جلوی نشتی را می‌گیرد (قبلاً false بود = اجازهٔ نشتی)
        }],
        "outbounds": [ ob, { "type": "direct", "tag": "direct" } ],
        "route": {
            "auto_detect_interface": true, "final": "proxy",
            "default_domain_resolver": { "server": "local-dns" },
            "rules": rules,
            "rule_set": rule_sets
        }
    }))
}

// ---------- اجرای بالادست (ادمین) برای sing-box ----------
#[repr(C)]
struct ShExecInfoW {
    cb_size: u32, f_mask: u32, hwnd: *mut core::ffi::c_void,
    lp_verb: *const u16, lp_file: *const u16, lp_params: *const u16, lp_dir: *const u16,
    n_show: i32, h_inst: *mut core::ffi::c_void, lp_idlist: *mut core::ffi::c_void,
    lp_class: *const u16, hkey_class: *mut core::ffi::c_void, dw_hotkey: u32,
    h_icon: *mut core::ffi::c_void, h_process: *mut core::ffi::c_void,
}
#[link(name = "shell32")]
extern "system" { fn ShellExecuteExW(info: *mut ShExecInfoW) -> i32; }
#[link(name = "kernel32")]
extern "system" {
    fn TerminateProcess(h: *mut core::ffi::c_void, code: u32) -> i32;
    fn WaitForSingleObject(h: *mut core::ffi::c_void, ms: u32) -> u32;
    fn CloseHandle(h: *mut core::ffi::c_void) -> i32;
}
fn wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}
// UAC می‌گیرد؛ None یعنی کاربر رد کرد یا خطا. با SEE_MASK_NOCLOSEPROCESS هندلِ پروسه را می‌گیریم.
fn spawn_elevated(exe: &std::path::Path, params: &str) -> Option<isize> {
    let verb = wide("runas");
    let file = wide(&exe.to_string_lossy());
    let par = wide(params);
    let mut info: ShExecInfoW = unsafe { std::mem::zeroed() };
    info.cb_size = std::mem::size_of::<ShExecInfoW>() as u32;
    info.f_mask = 0x0000_0040; // SEE_MASK_NOCLOSEPROCESS
    info.lp_verb = verb.as_ptr();
    info.lp_file = file.as_ptr();
    info.lp_params = par.as_ptr();
    info.n_show = 0; // SW_HIDE
    let ok = unsafe { ShellExecuteExW(&mut info) };
    if ok == 0 || info.h_process.is_null() { return None; }
    Some(info.h_process as isize)
}
fn handle_alive(h: isize) -> bool { unsafe { WaitForSingleObject(h as *mut _, 0) == 0x0000_0102 } } // WAIT_TIMEOUT
fn kill_handle(h: isize) { unsafe { TerminateProcess(h as *mut _, 0); CloseHandle(h as *mut _); } }

// ==========================================================================
//   گیم‌بوست — بهینه‌سازیِ شبکه‌ی سیستم موقعِ بازی (همه برگشت‌پذیر)
//   کاری که می‌کند (ادمین): (۱) کاهشِ تأخیر: NetworkThrottlingIndex=ffffffff،
//   SystemResponsiveness=0، خاموش‌کردنِ Nagle (TcpAckFrequency/TCPNoDelay) روی
//   کارت‌های فعال. (۲) آزادسازیِ پهنای باند: نگه‌داشتنِ موقتِ سرویس‌های آپدیت
//   (wuauserv, BITS, DoSvc). (۳) اختیاری: بالا بردنِ اولویتِ پروسه‌ی بازی به High.
//   موقعِ قطعِ گیم همه‌چیز از فایلِ state برمی‌گردد. اگر ناتمیز بسته شود:
//   تویک‌های رجیستری بی‌ضررند و آن سرویس‌ها خودشان on-demand دوباره روشن می‌شوند،
//   ضمناً اجرای بعدیِ گیم اول state کهنه را revert می‌کند.
//   نکته: اولویت‌دهیِ QoS به پکت‌ها روی اینترنت از سمتِ کلاینت شدنی نیست (روتر/ISP
//   تصمیم می‌گیرند)، پس آن را الکی وعده نمی‌دهیم.
// ==========================================================================
const GAMEBOOST_PS: &str = r#"param([string]$Cfg,[string]$SingBox,[string]$StopFlag,[string]$StateFile,[string]$GameExe="")
$ErrorActionPreference='SilentlyContinue'
$mm='HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile'
$mmReg='HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile'
function Apply-Boost {
  $state=[ordered]@{nti=$null;sr=$null;ifaces=@();svc=@{}}
  $state.nti=(Get-ItemProperty $mm -Name NetworkThrottlingIndex).NetworkThrottlingIndex
  $state.sr =(Get-ItemProperty $mm -Name SystemResponsiveness).SystemResponsiveness
  reg add $mmReg /v NetworkThrottlingIndex /t REG_DWORD /d 0xffffffff /f | Out-Null
  reg add $mmReg /v SystemResponsiveness /t REG_DWORD /d 0 /f | Out-Null
  $base='HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces'
  foreach($k in Get-ChildItem $base){
    $p=Get-ItemProperty $k.PSPath
    if($p.DhcpIPAddress -or $p.IPAddress){
      $state.ifaces+=(New-Object psobject -Property @{path=$k.PSPath;taf=$p.TcpAckFrequency;tnd=$p.TCPNoDelay})
      Set-ItemProperty $k.PSPath -Name TcpAckFrequency -Value 1 -Type DWord
      Set-ItemProperty $k.PSPath -Name TCPNoDelay -Value 1 -Type DWord
    }
  }
  foreach($s in 'wuauserv','BITS','DoSvc'){
    $svc=Get-Service $s
    if($svc){ $state.svc[$s]=$svc.Status.ToString(); if($svc.Status -eq 'Running'){ Stop-Service $s -Force } }
  }
  $state|ConvertTo-Json -Depth 6|Set-Content -Path $StateFile -Encoding UTF8
}
function Revert-Boost {
  if(-not (Test-Path $StateFile)){return}
  $state=Get-Content $StateFile -Raw|ConvertFrom-Json
  if($state.nti -is [int] -and $state.nti -ge 0){ reg add $mmReg /v NetworkThrottlingIndex /t REG_DWORD /d $state.nti /f|Out-Null } else { reg delete $mmReg /v NetworkThrottlingIndex /f|Out-Null }
  if($state.sr -is [int] -and $state.sr -ge 0){ reg add $mmReg /v SystemResponsiveness /t REG_DWORD /d $state.sr /f|Out-Null }
  foreach($i in $state.ifaces){
    if($null -ne $i.taf){ Set-ItemProperty $i.path -Name TcpAckFrequency -Value ([int]$i.taf) -Type DWord } else { Remove-ItemProperty $i.path -Name TcpAckFrequency }
    if($null -ne $i.tnd){ Set-ItemProperty $i.path -Name TCPNoDelay -Value ([int]$i.tnd) -Type DWord } else { Remove-ItemProperty $i.path -Name TCPNoDelay }
  }
  if($state.svc){ foreach($s in $state.svc.PSObject.Properties.Name){ if($state.svc.$s -eq 'Running'){ Start-Service $s } } }
  Remove-Item $StateFile -Force
}
Revert-Boost
Remove-Item $StopFlag -Force -ErrorAction SilentlyContinue
try{
  Apply-Boost
  Start-Job -ScriptBlock{ param($extra,$flag)
    $games=@('cs2','csgo','FiveM','FiveM_GTAProcess','FiveM_b2802_GTAProcess','FiveM_b3095_GTAProcess','GTA5','GTA5_Enhanced','PlayGTAV','RDR2','VALORANT-Win64-Shipping','valorant','LeagueClient','LeagueClientUx','dota2','r5apex','r5apex_dx12','ModernWarfare','cod','BlackOps6','TslGame','FortniteClient-Win64-Shipping','RainbowSix','RainbowSixGame','RustClient','Rust','Overwatch','RocketLeague','EscapeFromTarkov','EscapeFromTarkov_BE','destiny2','Cyberpunk2077','eldenring','ffxiv_dx11','Wow','HaloInfinite','DeadByDaylight','DBDGame','ForzaHorizon5','TheFinals','Warframe','bf2042','Warzone','samp','gta_sa','eurotrucks2','WorldOfTanks','minecraft','javaw','bedrock')
    if($extra -ne ''){ $n=[IO.Path]::GetFileNameWithoutExtension($extra); if($n){ $games+=$n } }
    while(-not (Test-Path $flag)){
      Get-Process -Name $games -ErrorAction SilentlyContinue|ForEach-Object{ try{ if($_.PriorityClass -ne 'High'){ $_.PriorityClass='High' } }catch{} }
      Start-Sleep -Seconds 3
    }
  } -ArgumentList $GameExe,$StopFlag|Out-Null
  $p=Start-Process -FilePath $SingBox -ArgumentList @('run','-c',$Cfg) -WindowStyle Hidden -PassThru
  while(-not (Test-Path $StopFlag) -and -not $p.HasExited){ Start-Sleep -Milliseconds 400 }
  if(-not $p.HasExited){ try{$p.Kill()}catch{} }
}
finally{
  Get-Job|Stop-Job -ErrorAction SilentlyContinue
  Get-Job|Remove-Job -Force -ErrorAction SilentlyContinue
  Revert-Boost
  Remove-Item $StopFlag -Force -ErrorAction SilentlyContinue
}
"#;

// اسمِ پروسه/فایلِ اجراییِ بازی را پاک‌سازی می‌کند (فقط کاراکترهای امن → جلوی تزریق به PowerShell)
fn sanitize_exe(name: &str) -> Option<String> {
    let n = name.trim();
    if n.is_empty() || n.len() > 80 { return None; }
    if !n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '.' | '_' | '-')) { return None; }
    Some(n.to_string())
}

// اتصالِ TUN/گیم — sing-box را با دسترسیِ ادمین بالا می‌آورد (کلِ سیستم از تونل)
// apps: قواعدِ per-app ["chrome.exe:p", "steam.exe:d", "torrent.exe:b"]
pub fn connect_tun(link: &str, game: bool, bypass_iran: bool, boost: bool, game_exe: Option<String>, apps: Vec<String>) -> Result<(), String> {
    disconnect();
    let my_gen = cur_gen();
    let cfg = build_tun_config_apps(link, game, bypass_iran, &apps)
        .ok_or_else(|| { log("خطا: کانفیگ برای TUN پشتیبانی نمی‌شود"); "این کانفیگ برای TUN پشتیبانی نمی‌شود".to_string() })?;
    let tmp = std::env::temp_dir().join(format!("shabgard_tun_{}.json", std::process::id()));
    std::fs::write(&tmp, serde_json::to_vec_pretty(&cfg).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let use_boost = game && boost;
    log(format!("{} به: {}", if game { if use_boost { "🎮 حالتِ گیم + بوست (TUN)" } else { "🎮 حالتِ گیم (TUN)" } } else { "🛡 حالتِ TUN" }, remark_of(link)));
    // اول بدونِ ادمین اعتبارسنجی کن تا اگر کانفیگ خراب است بی‌خود UAC نپرسیم
    if let Ok(out) = Command::new(bin("sing-box.exe")).arg("check").arg("-c").arg(&tmp)
        .creation_flags(NOWIN).output()
    {
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            let last = err.lines().last().unwrap_or("").to_string();
            log(format!("کانفیگِ TUN نامعتبر: {}", last));
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("کانفیگِ TUN نامعتبر: {last}"));
        }
    }
    let pid = std::process::id();
    let (h, stop_flag) = if use_boost {
        // گیم‌بوست: یک رَپرِ PowerShell (ادمین) که تویک‌ها را می‌زند، sing-box را اجرا می‌کند و
        // موقعِ دیدنِ فایلِ StopFlag همه‌چیز را برمی‌گرداند (پس با یک UAC هم بوست هم تونل).
        let ps = std::env::temp_dir().join(format!("shabgard_boost_{pid}.ps1"));
        let flag = std::env::temp_dir().join(format!("shabgard_stop_{pid}.flag"));
        let state = std::env::temp_dir().join(format!("shabgard_boost_{pid}.state.json"));
        let _ = std::fs::remove_file(&flag);
        std::fs::write(&ps, GAMEBOOST_PS).map_err(|e| e.to_string())?;
        let sb = bin("sing-box.exe");
        let exe_arg = game_exe.and_then(|g| sanitize_exe(&g)).unwrap_or_default();
        let params = format!(
            "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File \"{}\" -Cfg \"{}\" -SingBox \"{}\" -StopFlag \"{}\" -StateFile \"{}\" -GameExe \"{}\"",
            ps.to_string_lossy(), tmp.to_string_lossy(), sb.to_string_lossy(),
            flag.to_string_lossy(), state.to_string_lossy(), exe_arg
        );
        let h = spawn_elevated(std::path::Path::new("powershell.exe"), &params)
            .ok_or_else(|| { log("گیم لغو شد (دسترسیِ ادمین رد شد)"); "برای گیم‌مود باید «Yes» را در پنجره‌ی ادمین (UAC) بزنی".to_string() })?;
        (h, Some(flag))
    } else {
        let params = format!("run -c \"{}\"", tmp.to_string_lossy());
        let h = spawn_elevated(&bin("sing-box.exe"), &params)
            .ok_or_else(|| { log("TUN لغو شد (دسترسیِ ادمین رد شد)"); "برای TUN باید «Yes» را در پنجره‌ی ادمین (UAC) بزنی".to_string() })?;
        (h, None)
    };
    // اگر پروسه در چند ثانیه بمیرد یعنی درایورِ WinTun نیست یا سرور مرده است
    std::thread::sleep(Duration::from_millis(2800));
    if !handle_alive(h) {
        stop_tun(h, &stop_flag);
        let _ = std::fs::remove_file(&tmp);
        log("خطا: TUN بالا نیامد (درایور/کانفیگ/سرور)");
        return Err("TUN بالا نیامد — درایورِ WinTun یا کانفیگ/سرور مشکل دارد".into());
    }
    // تأییدِ واقعیِ اتصال (کلِ سیستم از تونل می‌رود، پس درخواستِ مستقیم هم از تونل خارج می‌شود).
    // چند تلاش می‌دهیم چون بالا آمدنِ کاملِ تونل کمی طول می‌کشد. اگر جواب نداد، «متصل» نگو.
    let mut ok = false;
    for _ in 0..3 {
        if cur_gen() != my_gen { stop_tun(h, &stop_flag); let _ = std::fs::remove_file(&tmp); return Err("کنسل شد".into()); }
        if probe_direct() { ok = true; break; }
        std::thread::sleep(Duration::from_millis(1200));
    }
    if cur_gen() != my_gen { stop_tun(h, &stop_flag); let _ = std::fs::remove_file(&tmp); return Err("کنسل شد".into()); }
    if !ok {
        stop_tun(h, &stop_flag);
        let _ = std::fs::remove_file(&tmp);
        log("خطا: تونل بالا آمد ولی ترافیک رد نشد (سرور مرده) — «متصل» نشد");
        return Err("این سرور از تونل جواب نداد — یکی دیگر را امتحان کن".into());
    }
    let wgen = {
        let mut g = eng().lock().unwrap();
        g.gen += 1;
        g.sb = Some(h); g.stop = stop_flag; g.connected = true; g.link = Some(link.to_string()); g.cfg = Some(tmp); g.port = 0;
        g.gen
    };
    std::thread::spawn(move || watchdog(wgen)); // پایشِ افتادنِ تونل (قبلاً فقط برای xray بود)
    log(if use_boost { "✅ گیم متصل شد — بوستِ شبکه فعال، همه‌ی ترافیک از تونل" } else { "✅ TUN متصل شد — همه‌ی ترافیک (بازی/برنامه‌ها) از تونل می‌رود" });
    Ok(())
}

// توقفِ تونل/گیم: در حالتِ گیم‌بوست اول StopFlag را می‌سازیم تا رَپر تنظیماتِ سیستم را
// تمیز برگرداند و خودش ببندد؛ بعد هندل را می‌بندیم. در حالتِ TUNِ ساده مستقیم می‌بندیم.
fn stop_tun(h: isize, stop_flag: &Option<PathBuf>) {
    if let Some(flag) = stop_flag {
        let _ = std::fs::write(flag, b"stop");
        // تا ۶ ثانیه فرصت بده رَپر revert کند و خودش خارج شود
        for _ in 0..24 { if !handle_alive(h) { break; } std::thread::sleep(Duration::from_millis(250)); }
    }
    kill_handle(h);
}

// ==========================================================================
//   رله (بلک‌اوت) — تونل از مسیرِ گوگل + کلادفلر برای وقتی اینترنت تقریباً قطع است
//   relay.exe یک پراکسیِ محلی روی ۸۰۸۵ بالا می‌آورد که ترافیک را با domain-fronting
//   از IP گوگل به Google Apps Script و از آنجا به Workerِ خودِ کاربر می‌فرستد.
//   نکته: رله MITM می‌کند، پس یک گواهیِ ریشه لازم دارد. مسیرِ CA را *ثابت* می‌دهیم
//   (DFT_CA_DIR) وگرنه زیرِ PyInstaller هر اجرا یک CA جدید ساخته و نصب می‌شد.
// ==========================================================================
const RELAY_PORT: u16 = 8085;

// ── پورتابل: اگر کنارِ exe پوشهٔ «data» باشد (یا فایلِ portable.txt)، همهٔ داده‌ها
// همان‌جا می‌روند — نه AppData، نه رجیستری. حذفِ پوشه = حذفِ کامل.
// در حالتِ نصب‌شده مثل قبل %LOCALAPPDATA%\Shabgard استفاده می‌شود.
pub fn portable_root() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    // فایلِ نشانگر — حتی اگر کاربر پوشهٔ data را نداشته باشد، ساختنش پورتابل می‌کند
    if exe_dir.join("portable.txt").exists() {
        return Some(exe_dir);
    }
    None
}

pub fn app_data_dir() -> PathBuf {
    if let Some(root) = portable_root() {
        let d = root.join("data");
        let _ = std::fs::create_dir_all(&d);
        return d;
    }
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    let d = PathBuf::from(base).join("Shabgard");  // مسیرِ لاتین: امن‌تر برای ابزارهای بیرونی
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn relay_running() -> bool {
    let g = eng().lock().unwrap();
    g.connected && g.port == RELAY_PORT && g.sb.is_none()
}

// auth_key و script_idها از ویزاردِ بلک‌اوت می‌آیند (آدرسِ Worker داخلِ خودِ Code.gs است)
pub fn connect_relay(auth_key: &str, script_ids: Vec<String>) -> Result<(), String> {
    if auth_key.trim().is_empty() { return Err("کلیدِ رله خالی است (مرحله ۱)".into()); }
    let ids: Vec<String> = script_ids.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .collect();
    if ids.is_empty() { return Err("Script ID معتبر نیست (مرحله ۳)".into()); }

    disconnect();
    let my_gen = cur_gen();
    log(format!("🌙 در حال اتصالِ رله ({} اسکریپت)…", ids.len()));

    let dir = app_data_dir();
    let ca_dir = dir.join("relay-ca");
    let _ = std::fs::create_dir_all(&ca_dir);
    let cfg_path = dir.join("relay-config.json");
    let cfg = json!({
        "mode": "apps_script",
        "front_domain": "www.google.com",
        "auth_key": auth_key.trim(),
        "script_ids": ids,
        "listen_host": "127.0.0.1",
        "listen_port": RELAY_PORT,
        "socks5_enabled": false,
        "log_level": "WARNING",
        "lan_sharing": false,
        "parallel_relay": 2,
        "relay_timeout": 25,
        "tls_connect_timeout": 15,
        "tcp_connect_timeout": 10
    });
    std::fs::write(&cfg_path, serde_json::to_vec_pretty(&cfg).map_err(|e| e.to_string())?)
        .map_err(|e| format!("نوشتنِ کانفیگِ رله نشد: {e}"))?;

    // اگر پورت از اجرای قبلی اشغال است، همان‌جا رد شو (پروسه‌ی دیگری را نمی‌کُشیم)
    if TcpStream::connect_timeout(&format!("127.0.0.1:{RELAY_PORT}").parse().unwrap(),
                                  Duration::from_millis(300)).is_ok() {
        log("پورتِ ۸۰۸۵ اشغال است — احتمالاً رله از قبل بالاست");
    }

    let child = Command::new(bin("relay.exe"))
        .arg("-c").arg(&cfg_path)
        .env("DFT_CA_DIR", &ca_dir)   // CAِ ثابت: فقط یک گواهی، برای همیشه
        .creation_flags(NOWIN)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().map_err(|e| { log("خطا: اجرای رله نشد"); format!("اجرای رله نشد: {e}") })?;
    adopt(child.id());

    // رله کندتر از xray بالا می‌آید (ساختِ CA + اسکنِ IP گوگل)
    if !port_up(RELAY_PORT, 25000) || cur_gen() != my_gen {
        let mut c = child; let _ = c.kill();
        if cur_gen() != my_gen { return Err("کنسل شد".into()); }
        log("خطا: رله بالا نیامد — کلید/Script ID یا Worker را چک کن");
        return Err("رله بالا نیامد — کلید، Script ID و Worker را چک کن".into());
    }
    // تأییدِ واقعی: قبل از دست‌زدن به پراکسیِ سیستم مطمئن شو ترافیک رد می‌شود
    let ok = probe_proxy(RELAY_PORT);
    if cur_gen() != my_gen { let mut c = child; let _ = c.kill(); return Err("کنسل شد".into()); }
    if !ok {
        let mut c = child; let _ = c.kill();
        log("خطا: رله جواب نداد (GAS/Worker مشکل دارد)");
        return Err("رله جواب نداد — مراحلِ ویزارد (Deploy و دسترسیِ Anyone) را چک کن".into());
    }
    set_proxy(&format!("127.0.0.1:{RELAY_PORT}"));
    let gen = {
        let mut g = eng().lock().unwrap();
        g.gen += 1;
        g.child = Some(child); g.exe = Some(bin("relay.exe"));
        g.connected = true; g.link = Some("__relay__".into());
        g.cfg = Some(cfg_path);       // تا Kill Switch بتواند رله را دوباره بالا بیاورد
        g.port = RELAY_PORT;
        g.gen
    };
    std::thread::spawn(move || watchdog(gen));
    log("✅ رله وصل شد (پراکسیِ سیستم روی ۸۰۸۵)");
    Ok(())
}

// ---------- پراکسیِ سیستم ----------
#[link(name = "wininet")]
extern "system" {
    fn InternetSetOptionW(h: *mut core::ffi::c_void, opt: u32, buf: *mut core::ffi::c_void, len: u32) -> i32;
}
fn refresh_inet() { unsafe { InternetSetOptionW(std::ptr::null_mut(), 39, std::ptr::null_mut(), 0); InternetSetOptionW(std::ptr::null_mut(), 37, std::ptr::null_mut(), 0); } }
fn reg(args: &[&str]) { let _ = Command::new("reg").args(args).creation_flags(NOWIN).output(); }
const IKEY: &str = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

fn reg_query(val: &str) -> String {
    if let Ok(out) = Command::new("reg").args(["query", IKEY, "/v", val]).creation_flags(NOWIN).output() {
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            let t = line.trim_start();
            if t.starts_with(val) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 { return parts[2..].join(" "); }
            }
        }
    }
    String::new()
}
// پراکسیِ قبلی (مثلِ v2rayN/nekobox) را نگه می‌داریم تا موقعِ قطع برگردانیم — هرگز آن را خراب نکن.
fn prev() -> &'static Mutex<Option<(String, String)>> {
    static P: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(None))
}
// پراکسیِ قبلی را روی دیسک هم نگه می‌داریم. اگر اپ کرش کند یا با Task Manager بسته
// شود، حافظه از بین می‌رود و پراکسیِ سیستم روی پورتِ مرده گیر می‌کند → کاربر کلاً
// بی‌اینترنت می‌شود. با این فایل، اجرای بعدی خودش وضعیت را برمی‌گرداند.
fn prev_file() -> PathBuf { app_data_dir().join("prev-proxy.json") }

fn set_proxy(hp: &str) {
    let en = reg_query("ProxyEnable");
    let sv = reg_query("ProxyServer");
    let _ = std::fs::write(&prev_file(),
        serde_json::to_vec(&json!({ "enable": en, "server": sv })).unwrap_or_default());
    *prev().lock().unwrap() = Some((en, sv));
    reg(&["add", IKEY, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "1", "/f"]);
    reg(&["add", IKEY, "/v", "ProxyServer", "/d", hp, "/f"]);
    reg(&["add", IKEY, "/v", "ProxyOverride", "/d", "localhost;127.*;<local>", "/f"]);
    refresh_inet();
}
fn unset_proxy() {
    let restored = prev().lock().unwrap().take();
    match restored {
        Some((en, sv)) if !sv.is_empty() && en.contains("0x1") => {
            // پراکسیِ قبلی را برگردان (v2rayN و… خراب نشود)
            reg(&["add", IKEY, "/v", "ProxyServer", "/d", &sv, "/f"]);
            reg(&["add", IKEY, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "1", "/f"]);
        }
        _ => {
            reg(&["add", IKEY, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f"]);
        }
    }
    let _ = std::fs::remove_file(prev_file());   // تمیز بستیم، دیگر بازیابی لازم نیست
    refresh_inet();
}

// اگر اجرای قبلی کرش کرد و پراکسی را روی پورتِ مرده رها کرد، همین‌جا برش گردان.
// (نشانه: فایلِ prev-proxy.json مانده و پراکسیِ فعلی به یک پورتِ محلیِ بسته اشاره دارد.)
fn restore_proxy_if_stale() {
    let f = prev_file();
    let Ok(txt) = std::fs::read_to_string(&f) else { return };
    let Ok(j) = serde_json::from_str::<Value>(&txt) else { let _ = std::fs::remove_file(&f); return };
    let cur = reg_query("ProxyServer");
    let cur_on = reg_query("ProxyEnable").contains("0x1");
    // فقط اگر پراکسیِ فعلی محلی و *مرده* است دست بزن — پراکسیِ زنده‌ی کاربر (v2rayN) را خراب نکن
    let dead_local = cur_on && cur.starts_with("127.0.0.1:")
        && cur.rsplit(':').next().and_then(|p| p.parse::<u16>().ok())
            .map(|p| TcpStream::connect_timeout(&format!("127.0.0.1:{p}").parse().unwrap(),
                                                Duration::from_millis(400)).is_err())
            .unwrap_or(false);
    if dead_local {
        let en = j["enable"].as_str().unwrap_or("");
        let sv = j["server"].as_str().unwrap_or("");
        if !sv.is_empty() && en.contains("0x1") {
            reg(&["add", IKEY, "/v", "ProxyServer", "/d", sv, "/f"]);
            reg(&["add", IKEY, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "1", "/f"]);
            log("پراکسیِ قبلی بعد از بسته‌شدنِ ناگهانی برگردانده شد");
        } else {
            reg(&["add", IKEY, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f"]);
            log("پراکسیِ مرده‌ی بازمانده از اجرای قبلی خاموش شد (اینترنت برگشت)");
        }
        refresh_inet();
    }
    let _ = std::fs::remove_file(&f);
}

// ---------- اتصال / قطع ----------
pub fn connect(link: &str, fragment: bool, bypass_iran: bool) -> Result<(), String> {
    connect_via(link, fragment, bypass_iran, None)
}

// carrier: برای wg:// روی نت‌هایی که UDP بسته است (تونل داخلِ تونل)
pub fn connect_via(link: &str, fragment: bool, bypass_iran: bool, carrier: Option<String>) -> Result<(), String> {
    disconnect();
    let my_gen = cur_gen();   // اگر کاربر کنسل بزند، این کهنه می‌شود و ما برمی‌گردیم
    log(format!("در حال اتصال به: {}{}{}", remark_of(link),
        if fragment { " · Fragment" } else { "" }, if bypass_iran { " · بایپسِ ایران" } else { "" }));
    let (port, mport) = free_ports();   // پورتِ پراکسی + پورتِ آمارِ مصرفِ داده
    let _ = std::fs::remove_file(access_log_path());   // هر اتصال، فعالیتِ تازه
    // hy2/tuic (QUIC/UDP) را xray نمی‌فهمد → با sing-box به‌عنوانِ پراکسیِ محلی (بدونِ ادمین) بالا می‌آوریم
    let is_wg = is_wireguard(link);
    let singbox_only = is_wg || is_singbox_only(link);
    let exe = if singbox_only { bin("sing-box.exe") } else { bin("xray.exe") };
    let cfg = if is_wg {
        build_wg_config_via(link, port, carrier.as_deref())
    } else if singbox_only {
        build_singbox_proxy_config(link, port)
    } else {
        build_config(link, port, fragment, bypass_iran, mport)
    }.ok_or_else(|| { log("خطا: کانفیگ پشتیبانی نمی‌شود"); "این کانفیگ پشتیبانی نمی‌شود".to_string() })?;
    let tmp = std::env::temp_dir().join(format!("shabgard_{}.json", std::process::id()));
    std::fs::write(&tmp, serde_json::to_vec(&cfg).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let child = Command::new(&exe).arg("run").arg("-c").arg(&tmp)
        .env("XRAY_LOCATION_ASSET", asset_dir())
        .creation_flags(NOWIN).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().map_err(|e| { log("خطا: اجرای هسته نشد"); format!("اجرای هسته نشد: {e}") })?;
    adopt(child.id());
    if !port_up(port, 6000) || cur_gen() != my_gen {
        let mut c = child; let _ = c.kill();
        if cur_gen() != my_gen { return Err("کنسل شد".into()); }
        log("خطا: اتصال برقرار نشد (کانفیگ/نت مرده)");
        return Err("اتصال برقرار نشد (کانفیگ/نت)".into());
    }
    // تأییدِ واقعی: قبل از دست‌زدن به پراکسیِ سیستم، مطمئن شو ترافیک واقعاً رد می‌شود.
    // اگر سرور مرده باشد، اینجا رد می‌شویم و «متصل» نمی‌گوییم (و پراکسیِ علی هم دست‌نخورده می‌ماند).
    let ok = probe_proxy(port);
    if cur_gen() != my_gen {
        let mut c = child; let _ = c.kill();
        return Err("کنسل شد".into());
    }
    if !ok {
        let mut c = child; let _ = c.kill();
        log("خطا: سرور جواب نداد (کانفیگ مرده) — «متصل» نشد");
        return Err("این سرور جواب نداد — یکی دیگر را امتحان کن".into());
    }
    if exits_iran(port) { log("⚠️ این کانفیگ از داخلِ ایران خارج می‌شود — فیلترشکنی نمی‌کند"); }
    set_proxy(&format!("127.0.0.1:{port}"));
    log(format!("✅ متصل شد (پراکسیِ سیستم روی {})", port));
    let gen = {
        let mut gme = eng().lock().unwrap();
        gme.gen += 1;
        gme.child = Some(child); gme.exe = Some(exe); gme.connected = true; gme.link = Some(link.to_string()); gme.cfg = Some(tmp); gme.port = port;
        gme.mport = if singbox_only { 0 } else { mport };
        gme.gen
    };
    std::thread::spawn(move || watchdog(gen)); // Kill Switch / اتصالِ مجدد
    Ok(())
}

// اطلاعاتِ خروجیِ واقعی (از داخلِ تونل) — کشور/شهرِ درست، نه برچسبِ اشتباهِ کانفیگ.
pub fn exit_info() -> NetInfo {
    let empty = || NetInfo { isp: String::new(), ip: String::new(), cc: String::new(), country: String::new() };
    let (port, is_tun, connected) = { let g = eng().lock().unwrap(); (g.port, g.sb.is_some(), g.connected) };
    if !connected { return empty(); }
    // حالتِ TUN: کلِ سیستم از تونل می‌رود، پس درخواستِ مستقیم (بدونِ پراکسی) هم از تونل خارج می‌شود
    let agent = if is_tun {
        ureq::AgentBuilder::new().timeout(Duration::from_secs(9)).build()
    } else {
        if port == 0 { return empty(); }
        let Ok(px) = ureq::Proxy::new(&format!("http://127.0.0.1:{port}")) else { return empty(); };
        ureq::AgentBuilder::new().proxy(px).timeout(Duration::from_secs(9)).build()
    };
    for url in ["https://api.ip.sb/geoip", "http://ip-api.com/json/?fields=query,isp,org,countryCode,country,city"] {
        let Ok(resp) = agent.get(url).set("User-Agent", "Mozilla/5.0").call() else { continue };
        let Ok(txt) = resp.into_string() else { continue };
        let Ok(j) = serde_json::from_str::<Value>(&txt) else { continue };
        let ip = j["ip"].as_str().or_else(|| j["query"].as_str()).unwrap_or("").to_string();
        if ip.is_empty() { continue; }
        let country = j["country"].as_str().or_else(|| j["country_name"].as_str()).unwrap_or("").to_string();
        let cc = j["country_code"].as_str().or_else(|| j["countryCode"].as_str()).unwrap_or("").to_string();
        let city = j["city"].as_str().unwrap_or("");
        let isp = j["isp"].as_str().or_else(|| j["organization"].as_str()).or_else(|| j["org"].as_str()).unwrap_or("").to_string();
        log(format!("خروجیِ واقعی: {} {} ({})", country, city, ip));
        return NetInfo { isp, ip, cc: if cc.len() == 2 { cc } else { String::new() },
                         country: if city.is_empty() { country } else { format!("{country} · {city}") } };
    }
    NetInfo { isp: String::new(), ip: String::new(), cc: String::new(), country: String::new() }
}

pub fn disconnect() {
    let mut gme = eng().lock().unwrap();
    let was = gme.connected;
    gme.gen += 1; // watchdog را متوقف کن (تا بعد از قطع، دوباره وصل نکند)
    let sb = gme.sb.take();
    let stop = gme.stop.take();
    if let Some(mut c) = gme.child.take() { let _ = c.kill(); }
    if let Some(p) = gme.cfg.take() { let _ = std::fs::remove_file(p); }
    gme.connected = false; gme.link = None; gme.port = 0; gme.mport = 0;
    drop(gme);
    let _ = std::fs::remove_file(access_log_path());  // لاگِ فعالیت روی دیسک نماند
    match sb {
        // حالتِ TUN/گیم: تونل را ببند (و در گیم‌بوست تنظیماتِ سیستم را برگردان) — رجیستریِ
        // پراکسی را دست نزن (وگرنه v2rayNِ علی خراب می‌شود)
        Some(h) => { stop_tun(h, &stop); if was { log(if stop.is_some() { "گیم قطع شد — بوستِ شبکه برگشت، مسیریابی آزاد شد" } else { "TUN قطع شد — مسیریابیِ سیستم آزاد شد" }); } }
        // حالتِ پراکسی: پراکسیِ قبلی (v2rayN/nekobox) را برگردان
        None => { unset_proxy(); if was { log("قطع شد — پراکسیِ قبلی برگردانده شد"); } }
    }
}

pub fn is_connected() -> bool { eng().lock().unwrap().connected }

// ==========================================================================
//   Cloudflare WARP — سرورِ مجانی، نامحدود و رسمیِ کلادفلر
//   خودمان یک کلیدِ X25519 می‌سازیم و در API عمومیِ WARP ثبت‌نام می‌کنیم
//   (همان کاری که خودِ اپِ 1.1.1.1 می‌کند). خروجی یک endpointِ WireGuard است
//   که sing-box اجرا می‌کند — بدونِ نیاز به اکانت، بدونِ نیاز به کانفیگِ کسی.
// ==========================================================================
const WARP_PEER_PUB: &str = "bmXOC+F1FxEMF9dyiK2H5/1SUtzH0JuVo51h2wPfgyo=";

fn warp_file() -> PathBuf { app_data_dir().join("warp.json") }

// ثبت‌نامِ تازه در WARP و ذخیره‌ی نتیجه (یک‌بار کافی است)
pub fn warp_register() -> Result<Value, String> {
    use base64::Engine as _;
    use rand_core::OsRng;
    let secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
    let public = x25519_dalek::PublicKey::from(&secret);
    let b64 = base64::engine::general_purpose::STANDARD;
    let priv_b64 = b64.encode(secret.to_bytes());
    let pub_b64 = b64.encode(public.as_bytes());

    let body = json!({
        "key": pub_b64,
        "install_id": "",
        "fcm_token": "",
        "tos": chrono_now(),
        "model": "PC",
        "serial_number": "",
        "locale": "en_US"
    });
    // مهم: API کلادفلر از داخلِ ایران مستقیم فیلتر است (تست شد: تایم‌اوت).
    // پس اگر همین حالا به یک سرور وصلیم، ثبت‌نام را از داخلِ همان تونل می‌فرستیم.
    let (port, is_tun, connected) = { let g = eng().lock().unwrap(); (g.port, g.sb.is_some(), g.connected) };
    let agent = if connected && !is_tun && port > 0 {
        match ureq::Proxy::new(&format!("http://127.0.0.1:{port}")) {
            Ok(px) => ureq::AgentBuilder::new().proxy(px).timeout(Duration::from_secs(30)).build(),
            Err(_) => ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).build(),
        }
    } else {
        ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).build()
    };
    let resp = agent.post("https://api.cloudflareclient.com/v0a4005/reg")
        .set("User-Agent", "okhttp/3.12.1")
        .set("CF-Client-Version", "a-6.30-3596")
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("ثبت‌نامِ WARP نشد (API کلادفلر از ایران فیلتر است — اول به یک سرور وصل شو، بعد WARP را بزن): {e}"))?;
    let j: Value = resp.into_json().map_err(|e| e.to_string())?;

    let v4 = j["config"]["interface"]["addresses"]["v4"].as_str().unwrap_or("172.16.0.2").to_string();
    let v6 = j["config"]["interface"]["addresses"]["v6"].as_str().unwrap_or("").to_string();
    let peer_pub = j["config"]["peers"][0]["public_key"].as_str().unwrap_or(WARP_PEER_PUB).to_string();
    let endpoint = j["config"]["peers"][0]["endpoint"]["host"].as_str().unwrap_or("162.159.192.1:2408").to_string();
    // ⚠️ حیاتی: کلادفلر ۳ بایتِ «reserved» در هدرِ WireGuard را با client_id چک می‌کند؛
    // اگر صفر بفرستیم پکت‌ها بی‌صدا دور ریخته می‌شوند («وصل است ولی هیچی لود نمی‌شود»).
    let cid = j["config"]["client_id"].as_str().unwrap_or("");
    let reserved = b64_safe(cid).unwrap_or_else(|| public.as_bytes()[..3].to_vec());
    let saved = json!({ "private_key": priv_b64, "v4": v4, "v6": v6,
                        "peer_public": peer_pub, "endpoint": endpoint, "reserved": reserved });
    let _ = std::fs::write(warp_file(), serde_json::to_vec_pretty(&saved).unwrap_or_default());
    log("✅ WARP ثبت‌نام شد (مجانی و نامحدود)");
    Ok(saved)
}

fn chrono_now() -> String {
    // زمانِ ISO تقریبی — API فقط قالب را می‌خواهد
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z", y, m, d,
            (secs / 3600) % 24, (secs / 60) % 60, secs % 60)
}
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// آماده بودنِ WARP فقط «وجودِ فایل» نیست — فایلِ قدیمیِ بدونِ reserved عملاً بی‌فایده است
// (کلادفلر پکت‌هایش را دور می‌ریزد)، پس false برمی‌گردانیم تا خودکار دوباره ثبت‌نام شود.
pub fn warp_ready() -> bool {
    let Ok(txt) = std::fs::read_to_string(warp_file()) else { return false };
    let Ok(w) = serde_json::from_str::<Value>(&txt) else { return false };
    let ok = w["private_key"].as_str().map(|s| !s.is_empty()).unwrap_or(false)
        && w["reserved"].as_array().map(|a| a.len() == 3).unwrap_or(false);
    if !ok { let _ = std::fs::remove_file(warp_file()); }
    ok
}

// ==========================================================================
//   اسکنرِ endpointِ WARP — همان ترفندی که Aether را «سریع» می‌کند
//   کلادفلر ده‌ها IP و پورت برای WARP دارد. در ایران بعضی‌ها فیلتر/کندند و
//   بعضی‌ها باز. اینجا واقعاً هرکدام را با sing-box بالا می‌آوریم و از داخلش
//   یک درخواستِ واقعی می‌زنیم؛ سریع‌ترینِ کارآمد را نگه می‌داریم.
//   (تستِ نتِ علی نشان داد UDP باز است و فقط endpointهای معروف بسته‌اند.)
// ==========================================================================
const WARP_PORTS: &[u16] = &[2408, 500, 1701, 4500, 8886, 880, 891, 903, 943, 955, 1018, 1387, 2371, 3138, 7156, 8319];
const WARP_NETS: &[&str] = &[
    "162.159.192", "162.159.193", "162.159.195",
    "188.114.96", "188.114.97", "188.114.98", "188.114.99",
];

fn warp_candidates(max: usize) -> Vec<String> {
    // ترکیبِ شبه‌تصادفیِ پایدار (بدونِ نیاز به کتابخانه‌ی rand)
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs()).unwrap_or(7);
    let mut out = Vec::new();
    let mut i = seed as usize;
    while out.len() < max {
        i = i.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let net = WARP_NETS[(i >> 13) % WARP_NETS.len()];
        let host = 1 + ((i >> 21) % 254);
        let port = WARP_PORTS[(i >> 31) % WARP_PORTS.len()];
        let ep = format!("{net}.{host}:{port}");
        if !out.contains(&ep) { out.push(ep); }
    }
    out
}

// یک endpoint را واقعاً امتحان می‌کند؛ خروجی: تأخیر (ms) یا None
fn try_warp_endpoint(ep: &str) -> Option<i32> {
    let port = free_port();
    let mut cfg = build_warp_config(port, None)?;
    // endpointِ کاندید را جایگزین کن
    let (host, p) = ep.rsplit_once(':')?;
    cfg["endpoints"][0]["peers"][0]["address"] = json!(host);
    cfg["endpoints"][0]["peers"][0]["port"] = json!(p.parse::<u16>().ok()?);
    let tmp = std::env::temp_dir().join(format!("wscan_{}_{}.json", std::process::id(), port));
    std::fs::write(&tmp, serde_json::to_vec(&cfg).ok()?).ok()?;
    let mut child = Command::new(bin("sing-box.exe")).arg("run").arg("-c").arg(&tmp)
        .creation_flags(NOWIN).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().ok()?;
    adopt(child.id());
    let res = (|| {
        if !port_up(port, 3000) { return None; }
        let px = ureq::Proxy::new(&format!("http://127.0.0.1:{port}")).ok()?;
        let agent = ureq::AgentBuilder::new().proxy(px).timeout(Duration::from_secs(5)).build();
        let t0 = Instant::now();
        let ms = match agent.get("http://cp.cloudflare.com/generate_204").call() {
            Ok(r) if r.status() == 204 || r.status() == 200 => t0.elapsed().as_millis() as i32,
            _ => return None,
        };
        // endpointی که خروجی‌اش داخلِ ایران است هیچ فایده‌ای ندارد
        if exit_cc_via(port) == "IR" { return None; }
        // سرعت هم مهم است: بعضی endpointها پینگِ خوب ولی پهنای باندِ افتضاح دارند
        // (علی گزارش داد WARP وصل می‌شود ولی چیزی لود نمی‌شود). ۲۵۶ کیلوبایت تست.
        let t1 = Instant::now();
        let kbps = match agent.get("http://speed.cloudflare.com/__down?bytes=262144").call() {
            Ok(r) => {
                let mut buf = Vec::new();
                if r.into_reader().read_to_end(&mut buf).is_err() { return None; }
                (buf.len() as f64 / 1024.0) / t1.elapsed().as_secs_f64().max(0.001)
            }
            Err(_) => return None,
        };
        if kbps < 120.0 { return None; }          // زیرِ ~۱ مگابیت = عملاً بی‌فایده
        // امتیاز: پینگ جریمه می‌شود، سرعت پاداش (کمترین بهتر)
        Some(ms - (kbps.min(4000.0) / 40.0) as i32)
    })();
    let _ = child.kill();
    let _ = std::fs::remove_file(&tmp);
    res
}

// اسکن: بهترین endpoint را پیدا و ذخیره می‌کند
pub fn warp_scan(count: usize) -> Result<String, String> {
    if !warp_ready() { warp_register()?; }
    let cands = warp_candidates(count.clamp(8, 60));
    log(format!("اسکنِ {} endpointِ کلادفلر برای WARP…", cands.len()));
    let mut best: Option<(i32, String)> = None;
    for ep in cands {
        if let Some(ms) = try_warp_endpoint(&ep) {
            log(format!("  ✅ {ep} → {ms}ms"));
            if best.as_ref().map(|(b, _)| ms < *b).unwrap_or(true) { best = Some((ms, ep)); }
            // اگر خیلی خوب بود، ادامه نده
            if best.as_ref().map(|(b, _)| *b < 200).unwrap_or(false) { break; }
        }
    }
    match best {
        Some((ms, ep)) => {
            // در فایلِ WARP ذخیره کن تا اتصالِ بعدی از همین برود
            if let Ok(txt) = std::fs::read_to_string(warp_file()) {
                if let Ok(mut w) = serde_json::from_str::<Value>(&txt) {
                    w["endpoint"] = json!(ep);
                    let _ = std::fs::write(warp_file(), serde_json::to_vec_pretty(&w).unwrap_or_default());
                }
            }
            log(format!("✅ بهترین endpointِ WARP: {ep} ({ms}ms)"));
            Ok(format!("{ep}|{ms}"))
        }
        None => Err("هیچ endpointِ کلادفلری روی نتِ تو باز نبود — دوباره اسکن کن یا از حاملِ سرور استفاده کن".into()),
    }
}

// ==========================================================================
//   WireGuard — کانفیگِ استانداردِ .conf (Windscribe / Proton / هر سرویسی)
//   کاربر با اکانتِ *خودش* از سایتِ آن سرویس کانفیگ می‌گیرد و اینجا پیست می‌کند.
//   این کاملاً مجاز است: از تیرِ مجانیِ خودشان با اکانتِ خودت استفاده می‌کنی.
//   لینکِ داخلیِ ما: wg://<base64 of the .conf>
// ==========================================================================
pub fn is_wireguard(link: &str) -> bool { link.trim().to_lowercase().starts_with("wg://") }

fn parse_wg_conf(conf: &str) -> Option<Value> {
    let (mut priv_key, mut addrs, mut mtu) = (String::new(), Vec::new(), 0u32);
    let (mut peer_pub, mut psk, mut endpoint) = (String::new(), String::new(), String::new());
    let mut section = "";
    for raw in conf.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() { continue; }
        if line.starts_with('[') { section = if line.eq_ignore_ascii_case("[Peer]") { "peer" } else { "iface" }; continue; }
        let Some((k, v)) = line.split_once('=') else { continue };
        let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
        match (section, k.as_str()) {
            ("iface", "privatekey") => priv_key = v,
            ("iface", "address") => addrs = v.split(',').map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect(),
            ("iface", "mtu") => mtu = v.parse().unwrap_or(0),
            ("peer", "publickey") => peer_pub = v,
            ("peer", "presharedkey") => psk = v,
            ("peer", "endpoint") => endpoint = v,
            _ => {}
        }
    }
    if priv_key.is_empty() || peer_pub.is_empty() || endpoint.is_empty() || addrs.is_empty() { return None; }
    // endpoint می‌تواند host:port یا [v6]:port باشد
    let (host, port) = endpoint.rsplit_once(':')?;
    let host = host.trim_matches(|c| c == '[' || c == ']').to_string();
    let mut peer = json!({
        "address": host, "port": port.parse::<u16>().ok()?,
        "public_key": peer_pub, "allowed_ips": ["0.0.0.0/0", "::/0"]
    });
    if !psk.is_empty() { peer["pre_shared_key"] = json!(psk); }
    Some(json!({
        "type": "wireguard", "tag": "wg",
        "address": addrs,
        "private_key": priv_key,
        "mtu": if mtu > 0 { mtu } else { 1420 },
        "peers": [peer]
    }))
}

// اسمِ خوانا برای نمایش در لیست
pub fn wg_name(link: &str) -> Option<String> {
    let conf = String::from_utf8(b64_safe(link.trim().trim_start_matches("wg://").trim_start_matches("WG://"))?).ok()?;
    let ep = conf.lines().find(|l| l.trim().to_lowercase().starts_with("endpoint"))?;
    let host = ep.split('=').nth(1)?.trim().rsplit_once(':').map(|(h, _)| h.to_string())?;
    Some(format!("WireGuard · {host}"))
}

// carrier: اگر داده شود، ترافیکِ WireGuard از داخلِ همان سرور (TCP) رد می‌شود.
// این تنها راهِ کارکردنِ WireGuard/WARP روی نت‌هایی است که UDP را بسته‌اند (سامانتل).
fn wg_config_with_carrier(ep: Value, port: u16, carrier: Option<&str>) -> Option<Value> {
    let mut ep = ep;
    let mut outs = vec![json!({ "type": "direct", "tag": "direct" })];
    if let Some(link) = carrier {
        let mut ob = singbox_outbound(link)?;
        ob["tag"] = json!("carrier");
        // MTU کمتر چون یک لایه‌ی تونل اضافه شده
        ep["mtu"] = json!(1280);
        ep["detour"] = json!("carrier");
        outs.insert(0, ob);
    }
    Some(json!({
        "log": { "level": "warn" },
        "dns": { "servers": [ { "tag": "l", "type": "local" } ], "final": "l" },
        "inbounds": [{ "type": "mixed", "tag": "in", "listen": "127.0.0.1", "listen_port": port }],
        "endpoints": [ ep ],
        "outbounds": outs,
        "route": { "final": "wg", "default_domain_resolver": { "server": "l" } }
    }))
}

fn build_wg_config(link: &str, port: u16) -> Option<Value> {
    build_wg_config_via(link, port, None)
}

fn build_wg_config_via(link: &str, port: u16, carrier: Option<&str>) -> Option<Value> {
    let conf = String::from_utf8(b64_safe(link.trim().trim_start_matches("wg://").trim_start_matches("WG://"))?).ok()?;
    let ep = parse_wg_conf(&conf)?;
    wg_config_with_carrier(ep, port, carrier)
}

// ==========================================================================
//   بازکردنِ اپ‌های ویندوز (UWP/Store) روی پراکسیِ محلی
//   اپ‌های Store داخلِ AppContainer اجرا می‌شوند و ویندوز اتصالشان به
//   127.0.0.1 را می‌بندد. نتیجه: حتی وقتی پراکسیِ سیستم ست است، Microsoft Store
//   و اپ‌های UWP به پراکسیِ ما نمی‌رسند و باز نمی‌شوند.
//   دستورِ رسمیِ ویندوز برای معافیت: CheckNetIsolation LoopbackExempt
//   (یک‌بار لازم است و برگشت‌پذیر است — با remove_uwp_exempt پاک می‌شود.)
// ==========================================================================
const UWP_PS: &str = r#"$ErrorActionPreference='SilentlyContinue'
$n=0
foreach($p in Get-AppxPackage){
  if($p.PackageFamilyName){
    CheckNetIsolation LoopbackExempt -a -n="$($p.PackageFamilyName)" | Out-Null
    $n++
  }
}
Write-Output "exempted:$n"
"#;

pub fn uwp_exempt() -> Result<String, String> {
    let ps = std::env::temp_dir().join(format!("shabgard_uwp_{}.ps1", std::process::id()));
    std::fs::write(&ps, UWP_PS).map_err(|e| e.to_string())?;
    let params = format!("-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File \"{}\"",
                         ps.to_string_lossy());
    let h = spawn_elevated(std::path::Path::new("powershell.exe"), &params)
        .ok_or_else(|| "برای این کار باید «Yes» را در پنجره‌ی ادمین بزنی".to_string())?;
    // منتظرِ پایان (حداکثر ۹۰ ثانیه — تعدادِ اپ‌ها زیاد است)
    for _ in 0..360 {
        if !handle_alive(h) { break; }
        std::thread::sleep(Duration::from_millis(250));
    }
    kill_handle(h);
    let _ = std::fs::remove_file(&ps);
    log("✅ اپ‌های ویندوز (Store/UWP) اجازه‌ی استفاده از پراکسیِ محلی گرفتند");
    Ok("ok".into())
}

// ==========================================================================
//   پس‌دادنِ حافظه — وقتی اپ می‌رود تو سینی، حافظه‌ی کاری را به ویندوز برگردان
//   (WebView2 چند پروسه دارد و هر کدام صفحاتِ بلااستفاده نگه می‌دارد. این کار
//   حافظه را آزاد می‌کند؛ به‌محضِ باز شدنِ پنجره خودش برمی‌گردد.)
// ==========================================================================
#[repr(C)]
struct ProcEntry32 {
    size: u32, usage: u32, pid: u32, default_heap: usize, module_id: u32,
    threads: u32, parent_pid: u32, pri_class_base: i32, flags: u32, exe: [u16; 260],
}
#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> *mut core::ffi::c_void;
    fn Process32FirstW(snap: *mut core::ffi::c_void, e: *mut ProcEntry32) -> i32;
    fn Process32NextW(snap: *mut core::ffi::c_void, e: *mut ProcEntry32) -> i32;
    fn SetProcessWorkingSetSize(h: *mut core::ffi::c_void, min: usize, max: usize) -> i32;
}

pub fn trim_memory() {
    let me = std::process::id();
    // ۱) خودمان
    trim_pid(me);
    // ۲) بچه‌های WebView2 (چند سطح)
    let mut targets = vec![me];
    unsafe {
        let snap = CreateToolhelp32Snapshot(0x0000_0002 /*TH32CS_SNAPPROCESS*/, 0);
        if snap.is_null() || snap as isize == -1 { return; }
        // سه دور برای گرفتنِ نوه‌ها
        for _ in 0..3 {
            let mut e: ProcEntry32 = std::mem::zeroed();
            e.size = std::mem::size_of::<ProcEntry32>() as u32;
            let mut ok = Process32FirstW(snap, &mut e);
            while ok != 0 {
                if targets.contains(&e.parent_pid) && !targets.contains(&e.pid) {
                    targets.push(e.pid);
                }
                ok = Process32NextW(snap, &mut e);
            }
        }
        CloseHandle(snap);
    }
    for pid in targets.into_iter().filter(|p| *p != me) { trim_pid(pid); }
}

fn trim_pid(pid: u32) {
    unsafe {
        // PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION
        let h = OpenProcess(0x0100 | 0x0400, 0, pid);
        if !h.is_null() {
            // (usize::MAX, usize::MAX) = «همه‌ی صفحاتِ بلااستفاده را پس بده»
            SetProcessWorkingSetSize(h, usize::MAX, usize::MAX);
            CloseHandle(h);
        }
    }
}

// ── اجرا هنگامِ روشن شدنِ ویندوز ──────────────────────────────────────────
// از کلیدِ Run کاربر استفاده می‌کنیم (بدونِ نیاز به ادمین). با `--tray` بالا
// می‌آید تا مزاحمِ کاربر نشود.
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

pub fn autostart_get() -> bool {
    if let Ok(out) = Command::new("reg").args(["query", RUN_KEY, "/v", "Shabgard"])
        .creation_flags(NOWIN).output() { out.status.success() } else { false }
}

pub fn autostart_set(on: bool) -> Result<(), String> {
    // پورتابل = صفر ردِ سیستم. استارتاپ ویندوز در این حالت غیرفعال است.
    if portable_root().is_some() {
        log("در حالتِ پورتابل، استارتاپِ خودکار غیرفعال است (بدونِ ردِ سیستم)");
        return Err("پورتابل".into());
    }
    if on {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let val = format!("\"{}\" --tray", exe.to_string_lossy());
        let out = Command::new("reg")
            .args(["add", RUN_KEY, "/v", "Shabgard", "/t", "REG_SZ", "/d", &val, "/f"])
            .creation_flags(NOWIN).output().map_err(|e| e.to_string())?;
        if !out.status.success() { return Err("ثبت در استارتاپ نشد".into()); }
        log("اجرا هنگامِ روشن شدنِ ویندوز: روشن");
    } else {
        let _ = Command::new("reg").args(["delete", RUN_KEY, "/v", "Shabgard", "/f"])
            .creation_flags(NOWIN).output();
        log("اجرا هنگامِ روشن شدنِ ویندوز: خاموش");
    }
    Ok(())
}

// کانفیگِ sing-box برای WARP (پراکسیِ محلی — بدونِ نیاز به ادمین)
fn build_warp_config(port: u16, carrier: Option<&str>) -> Option<Value> {
    let txt = std::fs::read_to_string(warp_file()).ok()?;
    let w: Value = serde_json::from_str(&txt).ok()?;
    let v4 = w["v4"].as_str()?;
    let v6 = w["v6"].as_str().unwrap_or("");
    let ep = w["endpoint"].as_str().unwrap_or("162.159.192.1:2408");
    let (host, p) = ep.rsplit_once(':')?;
    let mut addrs = vec![format!("{v4}/32")];
    if !v6.is_empty() { addrs.push(format!("{v6}/128")); }
    let reserved: Vec<u32> = w["reserved"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|n| n as u32)).collect())
        .unwrap_or_else(|| vec![0, 0, 0]);
    let ep = json!({
        "type": "wireguard", "tag": "wg",
        "address": addrs,
        "private_key": w["private_key"],
        "mtu": 1280,
        "peers": [{
            "address": host.trim_matches(|c| c == '[' || c == ']'),
            "port": p.parse::<u16>().unwrap_or(2408),
            "public_key": w["peer_public"],
            "allowed_ips": ["0.0.0.0/0", "::/0"],
            "reserved": reserved
        }]
    });
    wg_config_with_carrier(ep, port, carrier)
}

// ==========================================================================
//   gool — WARP داخلِ WARP (دو زنجیره‌ی WireGuard پشتِ سر هم)
//   چرا مفید است: (۱) محلِ مجازیِ خروج عوض می‌شود و گاهی از throttle/بلاکِ
//   ISP رد می‌شود، (۲) لایه‌ی دوم الگوی ترافیک را عوض می‌کند.
//   هزینه‌اش کمی پینگِ بیشتر است، پس فقط وقتی استفاده می‌شود که کاربر بخواهد
//   یا حالتِ ساده جواب ندهد. (همان ترفندِ warp-plus/Aether)
// ==========================================================================
fn build_gool_config(port: u16, outer: &Value, inner: &Value) -> Option<Value> {
    // لایهٔ دوم «هویتِ مستقل» دارد — کلادفلر دو session موازی با یک client_id را
    // قبول نمی‌کند و handshakeها همدیگر را می‌اندازند.
    let mk_peer = |reg: &Value| {
        let ep = reg["endpoint"].as_str().unwrap_or("162.159.192.1:2408");
        let (h, p) = ep.rsplit_once(':')?;
        Some(json!({
            "address": h.trim_matches(|c| c == '[' || c == ']'),
            "port": p.parse::<u16>().unwrap_or(2408),
            "public_key": reg["peer_public"],
            "allowed_ips": ["0.0.0.0/0", "::/0"],
            "reserved": reg["reserved"]
        }))
    };
    let mk_addr = |reg: &Value| {
        let v4 = reg["v4"].as_str()?;
        let mut a = vec![format!("{v4}/32")];
        if let Some(v6) = reg["v6"].as_str() { if !v6.is_empty() { a.push(format!("{v6}/128")); } }
        Some(a)
    };
    let outer_peer = mk_peer(outer)?;
    let inner_peer = mk_peer(inner)?;
    let outer_addr = mk_addr(outer)?;
    let inner_addr = mk_addr(inner)?;
    Some(json!({
        "log": { "level": "warn" },
        "dns": { "servers": [ { "tag": "l", "type": "local" } ], "final": "l" },
        "inbounds": [{ "type": "mixed", "tag": "in", "listen": "127.0.0.1", "listen_port": port }],
        "endpoints": [
            { "type": "wireguard", "tag": "warp1", "address": outer_addr,
              "private_key": outer["private_key"].clone(), "mtu": 1280, "peers": [ outer_peer ] },
            // لایهٔ دوم از داخلِ لایهٔ اول می‌رود؛ هویتِ خودش را دارد (MTU کمتر برای سربار)
            { "type": "wireguard", "tag": "warp2", "detour": "warp1", "address": inner_addr,
              "private_key": inner["private_key"].clone(), "mtu": 1180, "peers": [ inner_peer ] }
        ],
        "outbounds": [{ "type": "direct", "tag": "direct" }],
        "route": { "final": "warp2", "default_domain_resolver": { "server": "l" } }
    }))
}

// اتصالِ gool — WARP دولایه
pub fn connect_gool() -> Result<(), String> {
    // ثبت‌نام‌ها «وقتی هنوز وصلیم» انجام شود — API کلادفلر از ایران مستقیم فیلتر است.
    // (ترتیب مهم است: اول بیرونی، بعد داخلی با هویتِ مستقل.)
    if !warp_ready() { warp_register()?; }
    let outer: Value = serde_json::from_str(&std::fs::read_to_string(warp_file()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    log("ثبت‌نامِ هویتِ دوم برای لایهٔ داخلی…");
    let inner = warp_register()?;
    disconnect();
    let my_gen = cur_gen();
    log("در حال اتصالِ gool (WARP داخلِ WARP)…");
    let port = free_port();
    let cfg = build_gool_config(port, &outer, &inner).ok_or_else(|| "کانفیگِ gool ساخته نشد".to_string())?;
    let tmp = std::env::temp_dir().join(format!("shabgard_gool_{}.json", std::process::id()));
    std::fs::write(&tmp, serde_json::to_vec(&cfg).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let child = Command::new(bin("sing-box.exe")).arg("run").arg("-c").arg(&tmp)
        .creation_flags(NOWIN).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().map_err(|e| format!("اجرای gool نشد: {e}"))?;
    adopt(child.id());
    if !port_up(port, 12000) || cur_gen() != my_gen {
        let mut c = child; let _ = c.kill();
        if cur_gen() != my_gen { return Err("کنسل شد".into()); }
        return Err("gool بالا نیامد".into());
    }
    let ok = probe_proxy(port);
    if cur_gen() != my_gen { let mut c = child; let _ = c.kill(); return Err("کنسل شد".into()); }
    if ok && exits_iran(port) {
        let mut c = child; let _ = c.kill();
        return Err("gool هم از ایران خارج شد (بی‌فایده)".into());
    }
    if !ok {
        let mut c = child; let _ = c.kill();
        return Err("gool جواب نداد — اول endpoint را اسکن کن".into());
    }
    set_proxy(&format!("127.0.0.1:{port}"));
    let gen = {
        let mut g = eng().lock().unwrap();
        g.gen += 1;
        g.child = Some(child); g.exe = Some(bin("sing-box.exe"));
        g.connected = true; g.link = Some("__gool__".into()); g.cfg = Some(tmp);
        g.port = port; g.mport = 0;
        g.gen
    };
    std::thread::spawn(move || watchdog(gen));
    log("✅ gool متصل شد (WARP دولایه)");
    Ok(())
}

// اتصال به WARP (اگر ثبت‌نام نشده، اول ثبت‌نام می‌کند)
pub fn connect_warp(carrier: Option<String>) -> Result<(), String> {
    if !warp_ready() { warp_register()?; }
    disconnect();
    let my_gen = cur_gen();
    log("در حال اتصال به Cloudflare WARP…");
    let port = free_port();
    let cfg = build_warp_config(port, carrier.as_deref()).ok_or_else(|| "کانفیگِ WARP ساخته نشد".to_string())?;
    let tmp = std::env::temp_dir().join(format!("shabgard_warp_{}.json", std::process::id()));
    std::fs::write(&tmp, serde_json::to_vec(&cfg).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let child = Command::new(bin("sing-box.exe")).arg("run").arg("-c").arg(&tmp)
        .creation_flags(NOWIN).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().map_err(|e| format!("اجرای WARP نشد: {e}"))?;
    adopt(child.id());
    if !port_up(port, 9000) || cur_gen() != my_gen {
        let mut c = child; let _ = c.kill();
        if cur_gen() != my_gen { return Err("کنسل شد".into()); }
        return Err("WARP بالا نیامد".into());
    }
    let ok = probe_proxy(port);
    if cur_gen() != my_gen { let mut c = child; let _ = c.kill(); return Err("کنسل شد".into()); }
    if ok && exits_iran(port) {
        let mut c = child; let _ = c.kill();
        return Err("WARP از داخلِ ایران خارج شد (بی‌فایده) — راهِ بعدی امتحان می‌شود".into());
    }
    if !ok {
        let mut c = child; let _ = c.kill();
        log("خطا: WARP جواب نداد (شاید نتت UDP را بسته یا WARP فیلتر است)");
        return Err(if carrier.is_some() { "WARP از داخلِ این سرور هم جواب نداد — سرورِ دیگری را امتحان کن".into() }
                   else { "WARP جواب نداد (UDP روی نتت بسته است) — یک سرور انتخاب کن تا WARP از داخلِ آن رد شود".to_string() });
    }
    set_proxy(&format!("127.0.0.1:{port}"));
    let gen = {
        let mut g = eng().lock().unwrap();
        g.gen += 1;
        g.child = Some(child); g.exe = Some(bin("sing-box.exe"));
        g.connected = true; g.link = Some("__warp__".into()); g.cfg = Some(tmp);
        g.port = port; g.mport = 0;
        g.gen
    };
    std::thread::spawn(move || watchdog(gen));
    log("✅ WARP متصل شد");
    Ok(())
}

// ==========================================================================
//   مقایسه‌ی مسیر برای بازی — «تونل کمکت می‌کند یا خرابش می‌کند؟»
//   واقعیتِ فنی: عبور از VPN همیشه چند هاپ اضافه می‌کند. فقط وقتی پینگت را
//   *کم* می‌کند که ISP مسیرِ مستقیم را throttle/شکل‌دهی کرده باشد. پس به‌جای
//   ادعا، هر دو را اندازه می‌گیریم و عددِ واقعی را به کاربر می‌دهیم.
// ==========================================================================
#[derive(Serialize, Clone, Default)]
pub struct RouteAdvice { pub direct_ms: i32, pub tunnel_ms: i32, pub better: String }

fn tcp_ms(host: &str, port: u16, tries: u8) -> i32 {
    let Ok(mut it) = std::net::ToSocketAddrs::to_socket_addrs(&format!("{host}:{port}")) else { return -1 };
    let Some(sa) = it.next() else { return -1 };
    let mut best = i32::MAX;
    for _ in 0..tries.max(1) {
        let t0 = Instant::now();
        if TcpStream::connect_timeout(&sa, Duration::from_millis(2500)).is_ok() {
            let ms = t0.elapsed().as_millis() as i32;
            if ms < best { best = ms; }
        }
    }
    if best == i32::MAX { -1 } else { best }
}

// مقصدِ نمونه: سرورهای بازی معمولاً در اروپا هستند
pub fn route_advice() -> RouteAdvice {
    let target = ("speed.cloudflare.com", 443u16);
    let direct = tcp_ms(target.0, target.1, 3);
    let tunnel = {
        let (port, connected) = { let g = eng().lock().unwrap(); (g.port, g.connected) };
        if !connected || port == 0 { -1 } else { live_ping() }
    };
    let better = if direct > 0 && tunnel > 0 {
        if direct + 15 < tunnel { "direct".into() } else if tunnel + 15 < direct { "tunnel".into() } else { "same".into() }
    } else { String::new() };
    log(format!("مقایسه‌ی مسیر — مستقیم: {direct}ms · از تونل: {tunnel}ms → بهتر: {better}"));
    RouteAdvice { direct_ms: direct, tunnel_ms: tunnel, better }
}

// پینگِ زنده‌ی اتصالِ فعلی — از داخلِ همان تونلی که وصلی، نه عددِ کهنه‌ی تست.
pub fn live_ping() -> i32 {
    let (port, is_tun, connected) = { let g = eng().lock().unwrap(); (g.port, g.sb.is_some(), g.connected) };
    if !connected { return -1; }
    let agent = if is_tun || port == 0 {
        ureq::AgentBuilder::new().timeout(Duration::from_secs(4)).build()
    } else {
        let Ok(px) = ureq::Proxy::new(&format!("http://127.0.0.1:{port}")) else { return -1 };
        ureq::AgentBuilder::new().proxy(px).timeout(Duration::from_secs(4)).build()
    };
    let t0 = Instant::now();
    match agent.get("http://cp.cloudflare.com/generate_204").call() {
        Ok(r) if r.status() == 204 || r.status() == 200 => t0.elapsed().as_millis() as i32,
        _ => -1,
    }
}

// ظرفیتِ واقعیِ پهنای باند (مگابیت بر ثانیه) — یک دانلودِ کوتاه از داخلِ تونل.
// نتیجه به‌عنوانِ «سقف» استفاده می‌شود تا مصرفِ لحظه‌ای در برابرش نشان داده شود.
pub fn bandwidth_test() -> (f64, f64) {
    let (port, is_tun, connected) = { let g = eng().lock().unwrap(); (g.port, g.sb.is_some(), g.connected) };
    if !connected { return (0.0, 0.0); }
    let agent = if is_tun || port == 0 {
        ureq::AgentBuilder::new().timeout(Duration::from_secs(25)).build()
    } else {
        let Ok(px) = ureq::Proxy::new(&format!("http://127.0.0.1:{port}")) else { return (0.0, 0.0) };
        ureq::AgentBuilder::new().proxy(px).timeout(Duration::from_secs(25)).build()
    };
    // ۸ مگابایت از کلادفلر — به‌اندازه‌ی کافی بزرگ که سرعت واقعی دربیاید،
    // به‌اندازه‌ی کافی کوچک که دیتای کاربر را نخورد.
    // دانلود
    let t0 = Instant::now();
    let mut down = 0.0;
    if let Ok(resp) = agent.get("http://speed.cloudflare.com/__down?bytes=8388608").call() {
        let mut buf = Vec::new();
        if resp.into_reader().read_to_end(&mut buf).is_ok() {
            let secs = t0.elapsed().as_secs_f64().max(0.001);
            down = (buf.len() as f64 * 8.0) / secs / 1_000_000.0;
        }
    }
    // آپلود (۲ مگابایت — کافی برای تخمین، بدونِ هدر رفتنِ دیتا)
    let mut up = 0.0;
    let payload = vec![b'0'; 2 * 1024 * 1024];
    let t1 = Instant::now();
    if agent.post("http://speed.cloudflare.com/__up").send_bytes(&payload).is_ok() {
        let secs = t1.elapsed().as_secs_f64().max(0.001);
        up = (payload.len() as f64 * 8.0) / secs / 1_000_000.0;
    }
    log(format!("تستِ پهنای باند: ↓{:.1} ↑{:.1} Mbps", down, up));
    ((down * 10.0).round() / 10.0, (up * 10.0).round() / 10.0)
}

// ==========================================================================
//   فعالیتِ اینترنت — «چه سایتی الان باز می‌شود»
//   xray یک access-log می‌نویسد؛ هر خط شبیهِ این است:
//     2026/07/27 02:11:03 127.0.0.1:51233 accepted tcp:youtube.com:443 [proxy]
//   ما فقط مقصد را برمی‌داریم، دامنه‌ی ریشه را جدا می‌کنیم و آخرین‌ها را
//   یکتا و شمارش‌شده به رابط می‌دهیم. فایل محلی است و موقعِ قطع پاک می‌شود.
// ==========================================================================
pub fn access_log_path() -> PathBuf {
    app_data_dir().join("activity.log")
}

#[derive(Serialize, Clone)]
pub struct Activity { pub host: String, pub hits: u32, pub last: String }

// rotate کردنِ لاگِ فعالیت — اگر از ۵ مگ بیشتر شد فقط ۱ مگِ آخر نگه داشته می‌شود
// حتماً با seek کار می‌کنیم؛ کلِ فایل (می‌تواند چند گیگ باشد) هیچ‌وقت واردِ RAM نمی‌شود
fn rotate_log_if_big() {
    let p = access_log_path();
    let Ok(meta) = std::fs::metadata(&p) else { return };
    const MAX: u64 = 5 * 1024 * 1024;   // 5 MB
    if meta.len() <= MAX { return; }
    const KEEP: usize = 1024 * 1024;    // 1 MB
    let start = meta.len().saturating_sub(KEEP as u64);
    use std::io::{Read, Seek, SeekFrom, Write};
    let Ok(mut f) = std::fs::File::open(&p) else { return };
    if f.seek(SeekFrom::Start(start)).is_err() { return; }
    let mut tail = Vec::with_capacity(KEEP + 16);
    if f.read_to_end(&mut tail).is_err() { return; }
    drop(f);
    // وسطِ خط بریدیم — خطِ نصفهٔ اولِ تیل را بنداز
    if let Some(pos) = tail.iter().position(|&b| b == b'\n') { tail.drain(..=pos); }
    // درجا کوتاه کن و دم را بنویس (xray با append می‌نویسد، پس بعد از truncate سالم ادامه می‌دهد)
    let Ok(mut w) = std::fs::OpenOptions::new().write(true).open(&p) else { return };
    let _ = w.set_len(0);
    let _ = w.seek(SeekFrom::Start(0));
    let _ = w.write_all(&tail);
    let _ = w.flush();
    log(format!("لاگِ فعالیت rotate شد ({:.1} MB → {:.0} KB)", meta.len() as f64 / 1048576.0, tail.len() as f64 / 1024.0));
}

// از هر هزار بار فراخوانی یک‌بار چک می‌شود (overhead صفر)
pub fn maybe_rotate_log() {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    if COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 1000 != 0 { return; }
    rotate_log_if_big();
}

// چرخهٔ پس‌زمینه — وقتی اپ ساعت‌ها تو سینی است هم لاگ بی‌نهایت بزرگ نمی‌شود
// همان لحظهٔ استارت هم اجرا می‌شود تا لاگِ انباشتهٔ قبلی بلافاصله خالی شود
pub fn start_log_rotator() {
    std::thread::spawn(|| loop {
        rotate_log_if_big();
        std::thread::sleep(std::time::Duration::from_secs(1800));   // هر ۳۰ دقیقه
    });
}

pub fn activity(limit: usize) -> Vec<Activity> {
    maybe_rotate_log();
    let p = access_log_path();
    let Ok(data) = std::fs::read(&p) else { return vec![] };
    // فقط انتهای فایل را می‌خوانیم (لاگ می‌تواند بزرگ شود)
    let tail = if data.len() > 200_000 { &data[data.len() - 200_000..] } else { &data[..] };
    let txt = String::from_utf8_lossy(tail);
    let mut order: Vec<String> = Vec::new();
    let mut map: HashMap<String, (u32, String)> = HashMap::new();
    for line in txt.lines() {
        // مقصد بعد از "accepted " می‌آید: tcp:host:port یا udp:host:port
        let Some(rest) = line.split(" accepted ").nth(1) else { continue };
        let dest = rest.split_whitespace().next().unwrap_or("");
        let dest = dest.trim_start_matches("tcp:").trim_start_matches("udp:");
        let host = dest.rsplit_once(':').map(|(h, _)| h).unwrap_or(dest);
        if host.is_empty() || host.parse::<std::net::IpAddr>().is_ok() { continue; } // IP خام را نشان نده
        // دامنه‌ی ریشه (www.youtube.com → youtube.com)
        let parts: Vec<&str> = host.split('.').collect();
        let root = if parts.len() >= 3 && parts[0].eq_ignore_ascii_case("www") {
            parts[1..].join(".")
        } else { host.to_string() };
        let time = line.split_whitespace().nth(1).unwrap_or("").to_string();
        let e = map.entry(root.clone()).or_insert_with(|| { order.push(root.clone()); (0, String::new()) });
        e.0 += 1; e.1 = time;
    }
    // تازه‌ترین‌ها اول
    order.reverse();
    order.into_iter().take(limit).filter_map(|h| {
        map.get(&h).map(|(n, t)| Activity { host: h.clone(), hits: *n, last: t.clone() })
    }).collect()
}

// مصرفِ داده‌ی واقعیِ این اتصال (بایت). از endpointِ سبکِ آمارِ xray خوانده می‌شود.
// خروجی: (بالا‌رفته، پایین‌آمده). اگر در دسترس نبود (TUN/رله/hy2) صفر برمی‌گردد.
pub fn usage() -> (u64, u64) {
    let mport = { let g = eng().lock().unwrap(); if !g.connected { 0 } else { g.mport } };
    if mport == 0 { return (0, 0); }
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(2)).build();
    let Ok(resp) = agent.get(&format!("http://127.0.0.1:{mport}/debug/vars")).call() else { return (0, 0) };
    let Ok(txt) = resp.into_string() else { return (0, 0) };
    let Ok(j) = serde_json::from_str::<Value>(&txt) else { return (0, 0) };
    let (mut up, mut down) = (0u64, 0u64);
    if let Some(obj) = j["stats"]["outbound"].as_object() {
        for (tag, v) in obj {
            if tag == "metrics_in" || tag == "block" { continue; }
            up += v["uplink"].as_u64().unwrap_or(0);
            down += v["downlink"].as_u64().unwrap_or(0);
        }
    }
    (up, down)
}

// پاک‌سازیِ بازمانده‌های اجرای قبلی (اگر اپ کرش کرد یا با Task Manager بسته شد):
// برای هر جلسه‌ی قبلی پرچمِ توقف می‌گذاریم تا اگر رَپرِ گیم‌بوستش هنوز زنده است،
// تنظیماتِ سیستم را برگرداند و خودش ببندد؛ و فایل‌های کانفیگِ یتیم را پاک می‌کنیم.
pub fn cleanup_stale() {
    restore_proxy_if_stale();   // اگر اجرای قبلی کرش کرد، اینترنت را برگردان
    let me = std::process::id();
    let tmp = std::env::temp_dir();
    let Ok(rd) = std::fs::read_dir(&tmp) else { return };
    let mut n = 0;
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        // پرچمِ توقفِ جلسه‌های قبلی → رَپرِ زنده‌ی احتمالی revert می‌کند و می‌بندد
        if let Some(rest) = name.strip_prefix("shabgard_boost_") {
            if let Some(pid) = rest.split('.').next().and_then(|s| s.parse::<u32>().ok()) {
                if pid != me {
                    let _ = std::fs::write(tmp.join(format!("shabgard_stop_{pid}.flag")), b"stop");
                    n += 1;
                }
            }
        }
        // کانفیگ‌های موقتِ یتیمِ جلسه‌های قبلی (sbmulti_* = کانفیگِ موتورِ تستِ مشترک)
        if (name.starts_with("shabgard_") || name.starts_with("sbtest_") || name.starts_with("sbmulti_")) && (name.ends_with(".json") || name.ends_with(".ps1")) {
            let keep = name.contains(&me.to_string());
            if !keep { let _ = std::fs::remove_file(e.path()); }
        }
    }
    // پرچم‌های stop جلسه‌های قبلی هم پاک شوند
    for e in std::fs::read_dir(&tmp).into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("shabgard_stop_") && name.ends_with(".flag") && !name.contains(&me.to_string()) {
            let _ = std::fs::remove_file(e.path());
        }
    }
    if n > 0 { log(format!("پاک‌سازیِ {} جلسه‌ی ناتمامِ قبلی (بوستِ سیستم برگردانده شد)", n)); }
}

// ── پاک‌سازیِ کاملِ داده‌های محلی (دکمهٔ «حریم خصوصی» در تنظیمات) ─────────────
// همهٔ رَدهای روی دیسک را حذف می‌کند تا کاربر بتواند با یک کلیک ثابت کند که
// هیچ سابقه‌ای از فعالیتش باقی نمی‌ماند. اتصالِ فعال دست نمی‌خورد.
// خروجی: (تعدادِ فایلِ پاک‌شده، حجمِ آزادشده به بایت)
pub fn wipe_privacy_data() -> (u32, u64) {
    let mut files = 0u32;
    let mut bytes = 0u64;
    let rm = |p: &PathBuf, files: &mut u32, bytes: &mut u64| {
        if let Ok(m) = std::fs::metadata(p) {
            if m.is_file() {
                let sz = m.len();
                if std::fs::remove_file(p).is_ok() { *files += 1; *bytes += sz; }
            }
        }
    };
    // لاگِ فعالیت (سابقهٔ سایت‌هایی که باز شده — مهم‌ترین داده)
    rm(&access_log_path(), &mut files, &mut bytes);
    // وضعیتِ پراکسیِ قبلی — فقط وقتی «وصل نیستیم» پاک شود؛ این فایل حافظهٔ
    // پراکسیِ اصلی کاربر است و پاک‌کردنش وسطِ جلسه یعنی بعد از قطع، پراکسیِ
    // قبلیِ او (مثلاً v2rayN) از بین می‌رود.
    if !eng().lock().unwrap().connected {
        rm(&prev_file(), &mut files, &mut bytes);
    }
    // فایل‌های موقتِ جلسه‌ها در %TEMP% (کانفیگ‌ها حاوی رمزِ سرورند!)
    let me = std::process::id();
    let tmp = std::env::temp_dir();
    if let Ok(rd) = std::fs::read_dir(&tmp) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let ours_tmp = (name.starts_with("shabgard_") && name.ends_with(".json"))
                || name.starts_with("sbtest_")
                || name.starts_with("sbmulti_")
                || name.starts_with("wscan_");
            // فایل‌های جلسهٔ *فعلی* را نگه دار (اتصالِ زنده ممکن است بخواند)
            if ours_tmp && !name.contains(&me.to_string()) {
                rm(&e.path(), &mut files, &mut bytes);
            }
        }
    }
    log(format!("🧹 پاک‌سازیِ حریم خصوصی: {} فایل ({:.1} KB) حذف شد", files, bytes as f64 / 1024.0));
    (files, bytes)
}
