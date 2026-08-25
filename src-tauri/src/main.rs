#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod engine;
use engine::Server;

#[tauri::command]
async fn list_servers(subs: Vec<String>, manual: Vec<String>, game: Option<bool>) -> Vec<Server> {
    tauri::async_runtime::spawn_blocking(move || engine::list_from_subs_mode(&subs, &manual, game.unwrap_or(false)))
        .await
        .unwrap_or_default()
}

// تستِ واقعی — ناهمگام + نتیجه‌ی هر کانفیگ را همان لحظه با event می‌فرستد (تستِ زنده)
#[tauri::command]
async fn test_all(app: tauri::AppHandle, links: Vec<String>) -> Vec<i32> {
    use tauri::Emitter;
    tauri::async_runtime::spawn_blocking(move || {
        let app2 = app.clone();
        engine::test_batch(&links, move |i, ping| { let _ = app2.emit("test_result", (i, ping)); })
    }).await.unwrap_or_default()
}

// همه‌ی کارهای شبکه‌ای ناهمگام‌اند تا رابط هرگز «Not Responding» نشود
#[tauri::command]
async fn connect(link: String, fragment: bool, bypass: Option<bool>, carrier: Option<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || engine::connect_via(&link, fragment, bypass.unwrap_or(false), carrier))
        .await
        .map_err(|e| e.to_string())?
}

// کنسلِ اتصالِ در حالِ انجام (دکمه‌ی زردِ گیرکرده)
#[tauri::command]
fn cancel_connect() {
    engine::cancel_connect();
}

// رله‌ی بلک‌اوت — تونل از مسیرِ گوگل + کلادفلرِ خودِ کاربر
#[tauri::command]
async fn connect_relay(auth_key: String, script_ids: Vec<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || engine::connect_relay(&auth_key, script_ids))
        .await
        .map_err(|e| e.to_string())?
}

// حالتِ TUN/گیم — sing-box با دسترسیِ ادمین (UAC). game=true بهینه‌سازیِ تأخیر،
// boost=true گیم‌بوستِ سیستمی (کاهشِ تأخیر + نگه‌داشتنِ آپدیت‌ها + اولویتِ بازی).
#[tauri::command]
async fn connect_tun(link: String, game: bool, bypass: Option<bool>, boost: Option<bool>, game_exe: Option<String>, apps: Option<Vec<String>>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        engine::connect_tun(&link, game, bypass.unwrap_or(false), boost.unwrap_or(true), game_exe, apps.unwrap_or_default())
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn disconnect() {
    let _ = tauri::async_runtime::spawn_blocking(engine::disconnect).await;
}

#[tauri::command]
fn is_connected() -> bool {
    engine::is_connected()
}

// فعالیتِ اینترنت: چه سایت‌هایی از تونل باز شده‌اند (محلی و موقتی)
#[tauri::command]
async fn activity(limit: Option<usize>) -> Vec<engine::Activity> {
    let n = limit.unwrap_or(12);
    tauri::async_runtime::spawn_blocking(move || engine::activity(n)).await.unwrap_or_default()
}

// ظرفیتِ پهنای باند (Mbps) — یک دانلودِ کوتاهِ واقعی از داخلِ تونل
#[tauri::command]
async fn bandwidth_test() -> (f64, f64) {
    tauri::async_runtime::spawn_blocking(engine::bandwidth_test).await.unwrap_or((0.0, 0.0))
}

// اتصال به Cloudflare WARP (مجانی، نامحدود، بدونِ اکانت)
#[tauri::command]
async fn connect_warp(carrier: Option<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || engine::connect_warp(carrier)).await.map_err(|e| e.to_string())?
}

// gool — WARP داخلِ WARP (محلِ خروجِ مجازی عوض می‌شود، از throttle رد می‌شود)
#[tauri::command]
async fn connect_gool() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(engine::connect_gool).await.map_err(|e| e.to_string())?
}

// اسکنِ endpointهای کلادفلر تا WARP روی نتِ کاربر سریع و باز باشد
#[tauri::command]
async fn warp_scan(count: Option<usize>) -> Result<String, String> {
    let n = count.unwrap_or(24);
    tauri::async_runtime::spawn_blocking(move || engine::warp_scan(n)).await.map_err(|e| e.to_string())?
}

// اپ‌های Store/UWP اجازه‌ی اتصال به پراکسیِ محلی بگیرند (یک‌بار، با ادمین)
#[tauri::command]
async fn uwp_exempt() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(engine::uwp_exempt).await.map_err(|e| e.to_string())?
}

// پس‌دادنِ حافظه‌ی بلااستفاده به ویندوز (وقتی پنجره مخفی می‌شود)
#[tauri::command]
async fn trim_memory() {
    let _ = tauri::async_runtime::spawn_blocking(engine::trim_memory).await;
}

// اجرا هنگامِ روشن شدنِ ویندوز
#[tauri::command]
fn autostart_get() -> bool { engine::autostart_get() }

#[tauri::command]
fn autostart_set(on: bool) -> Result<(), String> { engine::autostart_set(on) }

// مقایسه‌ی مسیرِ مستقیم و تونل — برای بازی کدام بهتر است
#[tauri::command]
async fn route_advice() -> engine::RouteAdvice {
    tauri::async_runtime::spawn_blocking(engine::route_advice).await.unwrap_or_default()
}

// پینگِ زنده‌ی اتصالِ فعلی (میلی‌ثانیه، ‎-1‎ یعنی در دسترس نیست)
#[tauri::command]
async fn live_ping() -> i32 {
    tauri::async_runtime::spawn_blocking(engine::live_ping).await.unwrap_or(-1)
}

// مصرفِ داده‌ی واقعیِ اتصالِ فعلی: [بالا، پایین] به بایت
#[tauri::command]
async fn usage() -> (u64, u64) {
    tauri::async_runtime::spawn_blocking(engine::usage).await.unwrap_or((0, 0))
}

#[tauri::command]
fn get_log() -> Vec<String> {
    engine::get_log()
}

// حریم خصوصی: همهٔ ردهای محلی (لاگِ فعالیت، کانفیگ‌های موقت، وضعیت پراکسی) پاک شود
#[tauri::command]
fn wipe_privacy() -> (u32, u64) {
    engine::wipe_privacy_data()
}

// ── آپدیتِ خودکار ──
#[tauri::command]
async fn check_update() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(engine::check_update)
        .await.map_err(|e| e.to_string())?
}

// دانلود installer جدید و اجرا (اپ با taskkill بسته می‌شود — نصاب خودش ادامه می‌دهد)
#[tauri::command]
fn install_update() -> Result<(), String> {
    let path = engine::download_update()?;
    engine::log("نصبِ نسخهٔ جدید شروع شد…");
    // نصاب NSIS currentUser خودش اپ را می‌بندد و پس از آپدیت اجرا می‌کند
    std::process::Command::new(&path)
        .spawn().map_err(|e| e.to_string())?;
    std::process::exit(0);
}

#[tauri::command]
async fn exit_info() -> engine::NetInfo {
    tauri::async_runtime::spawn_blocking(engine::exit_info)
        .await
        .unwrap_or(engine::NetInfo { isp: String::new(), ip: String::new(), cc: String::new(), country: String::new() })
}

// تحلیلِ کاملِ نت: چطور فیلتر شده و چه نوع کانفیگی رویش بهتر کار می‌کند
#[tauri::command]
async fn analyze_net() -> engine::NetProbe {
    tauri::async_runtime::spawn_blocking(engine::analyze_net)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn detect_net() -> engine::NetInfo {
    tauri::async_runtime::spawn_blocking(engine::detect_net)
        .await
        .unwrap_or(engine::NetInfo { isp: "نامشخص".into(), ip: String::new(), cc: String::new(), country: String::new() })
}

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(hwnd: *mut core::ffi::c_void, op: *const u16, file: *const u16,
                     params: *const u16, dir: *const u16, show: i32) -> isize;
}

// ── پرفراپِ حرفه‌ای: فهرستِ برنامه‌های در حالِ اجرا (پنجره‌دار) ───────────────
// EnumWindows → PID → QueryFullProcessImageNameW = مسیرِ کاملِ exe.
// برنامه‌های UWP/Store هم exe دارند (زیر WindowsApps\)؛ مسیرِ کامل را می‌دهیم
// تا قاعدهٔ per-app با process_path_regex حتی آن‌ها را هم بگیرد.
#[derive(serde::Serialize)]
pub struct RunningApp {
    pub name: String,     // نامِ نمایشی (بدون .exe)
    pub exe: String,      // chrome.exe
    pub path: String,     // مسیرِ کامل
    pub title: String,    // عنوانِ پنجرهٔ اصلی
    pub is_store: bool,   // UWP/Store
}
#[tauri::command]
fn list_running_apps() -> Vec<RunningApp> {
    use std::collections::HashMap;
    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(cb: usize, lparam: isize) -> i32;
        fn IsWindowVisible(h: *mut core::ffi::c_void) -> i32;
        fn GetWindowTextW(h: *mut core::ffi::c_void, buf: *mut u16, max: i32) -> i32;
        fn GetWindowThreadProcessId(h: *mut core::ffi::c_void, pid: *mut u32) -> u32;
        fn IsIconic(h: *mut core::ffi::c_void) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut core::ffi::c_void;
        fn CloseHandle(h: *mut core::ffi::c_void) -> i32;
        fn QueryFullProcessImageNameW(h: *mut core::ffi::c_void, flags: u32,
                                      buf: *mut u16, size: *mut u32) -> i32;
    }
    struct Ctx { apps: HashMap<String, RunningApp> }
    unsafe extern "system" fn cb(hwnd: *mut core::ffi::c_void, lparam: isize) -> i32 {
        let ctx = &mut *(lparam as *mut Ctx);
        if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 { return 1; }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 { return 1; }
        let mut title = [0u16; 256];
        let n = GetWindowTextW(hwnd, title.as_mut_ptr(), 256);
        let title = String::from_utf16_lossy(&title[..n.max(0) as usize]);
        if title.is_empty() { return 1; }   // فقط پنجره‌های واقعی
        // مسیرِ exe از PID
        let h = OpenProcess(0x1000, 0, pid);   // PROCESS_QUERY_LIMITED_INFORMATION
        if h.is_null() { return 1; }
        let mut path = [0u16; 1024];
        let mut sz = 1024u32;
        let ok = QueryFullProcessImageNameW(h, 0, path.as_mut_ptr(), &mut sz);
        CloseHandle(h);
        if ok == 0 || sz == 0 { return 1; }
        let full = String::from_utf16_lossy(&path[..sz as usize]);
        let low = full.to_lowercase();
        // خودمان و اجزای ویندوز را نشان نده
        if low.contains("shabgard.exe") || low.ends_with("\\system32\\") { return 1; }
        let exe = full.rsplit('\\').next().unwrap_or("").to_lowercase();
        if exe.is_empty() { return 1; }
        let is_store = low.contains("\\windowsapps\\") || low.contains("\\systemapps\\");
        let name = exe.trim_end_matches(".exe").to_string();
        let entry = ctx.apps.entry(exe.clone()).or_insert_with(|| RunningApp {
            name: name.clone(), exe, path: full.clone(), title: title.clone(), is_store,
        });
        // عنوانِ طولانی‌تر = نمایانگرتر
        if title.len() > entry.title.len() { entry.title = title; }
        1
    }
    let mut ctx = Ctx { apps: HashMap::new() };
    unsafe { EnumWindows(cb as *const () as usize, &mut ctx as *mut Ctx as isize); }
    let mut v: Vec<RunningApp> = ctx.apps.into_values().collect();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v.truncate(200);
    v
}

// فقط لینکِ http(s)ی سالم را در مرورگرِ سیستم باز می‌کند (بدونِ cmd → تزریقِ دستور ممکن نیست)
#[tauri::command]
fn open_url(url: String) {
    if !(url.starts_with("https://") || url.starts_with("http://")) { return; }
    // هیچ کاراکترِ کنترلی/خطرناک/فاصله مجاز نیست
    if url.len() > 2048 || url.chars().any(|c| c.is_control() || c.is_whitespace()
        || matches!(c, '"' | '\'' | '<' | '>' | '|' | '^' | '`')) { return; }
    unsafe {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = std::ffi::OsStr::new(&url).encode_wide().chain(std::iter::once(0)).collect();
        let op: Vec<u16> = std::ffi::OsStr::new("open").encode_wide().chain(std::iter::once(0)).collect();
        ShellExecuteW(std::ptr::null_mut(), op.as_ptr(), wide.as_ptr(),
                      std::ptr::null(), std::ptr::null(), 1);
    }
}

fn main() {
    // ── سبک‌سازیِ WebView2 (باید قبل از ساختِ پنجره ست شود) ──────────────
    // رابطِ ما ثابت و ساده است و به شتاب‌دهنده‌ی GPU نیازی ندارد؛ خاموش‌کردنش
    // مصرفِ GPU را عملاً صفر می‌کند و یکی-دو پروسه هم کمتر می‌شود.
    // محدود کردنِ رندرر و حافظه‌ی JS هم مصرفِ رم را پایین می‌آورد.
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--disable-gpu --disable-gpu-compositing --disable-software-rasterizer          --renderer-process-limit=1 --disable-dev-shm-usage          --disable-features=AudioServiceOutOfProcess,CalculateNativeWinOcclusion          --js-flags=--max-old-space-size=96",
    );
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
                let _ = w.unminimize();
            }
        }))
        .on_window_event(|win, event| {
            // ذخیرهٔ هندسهٔ پنجره — دفعهٔ بعد همان‌جا و هم‌اندازه بالا بیاید
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (win.outer_position().map(|p| p.x), win.outer_position().map(|p| p.y), win.inner_size().map(|s| s.width), win.inner_size().map(|s| s.height)) {
                    let _ = std::fs::write(engine::app_data_dir().join("win_geometry.txt"), format!("{x} {y} {w} {h}"));
                }
            }
        })
        .setup(|app| {
            use tauri::Manager;
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
            // بازیابیِ هندسهٔ پنجره از اجرای قبل (اگر روی صفحه هست)
            if let Some(w) = app.get_webview_window("main") {
                if let Ok(txt) = std::fs::read_to_string(engine::app_data_dir().join("win_geometry.txt")) {
                    let nums: Vec<i32> = txt.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                    if nums.len() == 4 {
                        let (x, y, wd, ht) = (nums[0], nums[1], nums[2], nums[3]);
                        // فقط اگر کاملاً داخل یک مانیتور است — وگرنه پیش‌فرض
                        let on_screen = x > -200 && y > -200 && wd > 300 && ht > 300;
                        if on_screen { let _ = w.set_position(tauri::LogicalPosition::new(x, y)); }
                    }
                }
            }
            if let Ok(res) = app.path().resource_dir() {
                engine::set_bin_dir(res.join("binaries"));
            }
            // بازمانده‌های اجرای قبلی (کرش/کیل) را تمیز کن — بوستِ سیستم برگردد
            std::thread::spawn(engine::cleanup_stale);
            // لاگِ فعالیت هرگز بزرگ‌تر از ۵ مگ نشود — حتی وقتی اپ ساعت‌ها در سینی است
            engine::start_log_rotator();
            // اگر ویندوز ما را در استارتاپ اجرا کرده (--tray)، ساکت برو به سینی
            if std::env::args().any(|a| a == "--tray") {
                if let Some(w) = app.get_webview_window("main") { let _ = w.hide(); }
            }
            // ── System Tray ──
            // منوی کامل: قطع/وصلِ سریع بدون بازکردن پنجره (مثل v2rayN/Hiddify)
            let show = MenuItem::with_id(app, "show", "Show · نمایش", true, None::<&str>)?;
            let disconnect_mi = MenuItem::with_id(app, "tray_disconnect", "Disconnect · قطع", true, None::<&str>)?;
            let best = MenuItem::with_id(app, "tray_best", "Connect to best · بهترین سرور", true, None::<&str>)?;
            let warp = MenuItem::with_id(app, "tray_warp", "WARP", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit · خروج", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &disconnect_mi, &best, &warp, &quit])?;
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Shabgard")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => { if let Some(w) = app.get_webview_window("main") { let _ = w.show(); let _ = w.unminimize(); let _ = w.set_focus(); } }
                    "quit" => { engine::disconnect(); app.exit(0); }
                    "tray_disconnect" => { engine::log("قطع از سینی"); std::thread::spawn(engine::disconnect); }
                    // وصل‌شدن به بهترین/وارپ از سینی: آخرین لینکِ موفق (یا اولین زندهٔ لیست کش) استفاده می‌شود.
                    // سنگین است → ترد جدا تا UI سینی قفل نشود؛ نتیجه در لاگ می‌آید.
                    "tray_best" => {
                        engine::log("اتصال به بهترین از سینی…");
                        std::thread::spawn(|| {
                            if let Some(link) = engine::best_cached_link() {
                                let _ = engine::connect_via(&link, false, true, None);
                            } else { engine::log("سروری برای اتصال نیست — یکبار از داخل برنامه «تست همه» بزن"); }
                        });
                    }
                    "tray_warp" => {
                        engine::log("WARP از سینی…");
                        std::thread::spawn(|| { let _ = engine::connect_warp(None); });
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") { let _ = w.show(); let _ = w.unminimize(); let _ = w.set_focus(); }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|win, event| {
            // ضربدر → برو به tray (VPN وصل می‌ماند)، نه خروج
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = win.hide();
                // رفتیم تو سینی → حافظه‌ی کاری را به ویندوز پس بده
                std::thread::spawn(engine::trim_memory);
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_servers, test_all, connect, connect_tun, connect_relay, cancel_connect, disconnect, is_connected, usage, activity, bandwidth_test, live_ping, connect_warp, connect_gool, warp_scan, route_advice, uwp_exempt, trim_memory, autostart_get, autostart_set, open_url, get_log, wipe_privacy, list_running_apps, check_update, install_update, detect_net, exit_info, analyze_net
        ])
        .run(tauri::generate_context!())
        .expect("error running shabgard");
    engine::disconnect();
}
