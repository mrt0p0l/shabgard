// شبگرد — رابط (فعلاً با داده‌ی نمونه؛ بعداً به هسته‌ی Rust وصل می‌شود)
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => [...r.querySelectorAll(s)];
// امنیت: فرار از HTML برای هر داده‌ی نامطمئن (اسمِ کانفیگ از سابِ اینترنتی و…) — جلوی XSS
const esc = (s) => String(s == null ? "" : s).replace(/[&<>"']/g,
  c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

// ---------- زبان (فارسی / English) ----------
let LANG = localStorage.getItem("lang") || "en";
const I18N = {
  fa: {
    off:"قطع", on:"متصل", brand:"شبگرد",
    tab_servers:"سرورها", tab_myconfig:"کانفیگِ من", tab_blackout:"بلک‌اوت",
    connect:"اتصال", disconnect:"قطع اتصال", connecting:"در حال اتصال…", connected_lbl:"متصل", pickfirst:"اول یک سرور انتخاب کن",
    heroName0:"سروری انتخاب نشده", heroSub0:"یک سرور از لیست انتخاب کن",
    fragment:"Fragment — ضدِ DPI (دورزدنِ فیلترینگ)",
    testAll:"تست همه", onlyWorking:"فقط کارآمد", best:"اتصالِ خودکار به بهترین", search:"جستجو…",
    mc_title:"کانفیگِ خودت", mc_add:"افزودن کانفیگ", mc_added:"کانفیگ‌های اضافه‌شده",
    subs_title:"ساب‌ها (Subscription)", sub_add:"افزودن ساب",
    bo_title:"بلک‌اوت — رله‌ی شخصی", relay_connect:"اتصالِ رله",
    exit_from:"خروج از:", servers_word:"سرور", relay_row:"رله (بلک‌اوت)", checking_exit:"در حال بررسیِ محلِ خروج…",
    route_lbl:"مسیریابی:", route_lbl2:"مسیریابی", mode_lbl2:"حالت", route_global:"همه از VPN", route_bypass:"بایپسِ ایران",
    mc_hint:"لینکِ کانفیگ‌ها را بگذار (هر خط یکی):", mc_empty:"هنوز کانفیگی اضافه نکردی", sub_empty:"ساب اضافه نشده", copy:"کپی",
    bo_desc:"وقتی اینترنت تقریباً قطع است، رله از مسیرِ گوگل + کلادفلر تونل می‌زند. یک‌بار طبقِ مراحل بساز، بعد آن را به‌عنوانِ یک VPNِ جدا روشن کن (جدا از لیستِ سرورها).",
    relay_state0:"رله هنوز تنظیم نشده", relay_hint0:"مراحلِ پایین را انجام بده", relay_ready:"رله آماده است", relay_press:"برای اتصال دکمه را بزن", relay_on:"رله متصل (VPNِ جدا)", relay_disconnect:"قطعِ رله",
    wz1_t:"یک کلیدِ امن بساز", wz1_b:"— هر رشته‌ای (این رمزِ بینِ گوگل و کلادفلرِ توست):",
    wz2_t:"Cloudflare Worker بساز", wz2_a:"— برو به", wz2_b:"Workers & Pages Create Worker. یک اسم بده؛ آدرسش می‌شود مثلِ", wz2_c:". آن آدرس را اینجا بگذار:", wz2_code:"کدِ Worker (این را داخلِ ادیتور پیست کن Deploy)",
    wz3_t:"Google Apps Script بساز", wz3_a:"— برو به", wz3_b:"New project. این کدِ آماده (با کلید و آدرسِ خودت پُر شده) را کپی و پیست کن:", wz3_code:"کدِ Code.gs (آماده)", wz3_deploy:"سپس بالا سمتِ راست Deploy New deployment کنارِ «Select type» چرخ‌دنده Web app «Who has access» = Anyone Deploy.", wz3_id:"حالا آدرسی مثلِ …/macros/s/AKfyc…/exec می‌دهد؛ قسمتِ AKfyc… را اینجا بگذار:",
    oauth_t:"گوگل یک هشدار نشان می‌دهد", oauth_b:"(چون اپِ شخصیِ خودت است، کاملاً امن است). این مراحل را دنبال کن:", oauth_1:"«Authorize access» اکانتِ جیمیلت را انتخاب کن.", oauth_2:"صفحه‌ی «Google hasn't verified this app» پایین «Advanced» را بزن.", oauth_3:"«Go to … (unsafe)» را بزن.", oauth_4:"در آخر «Continue» / «Allow» را بزن تا اجازه بدهی.",
    wz4_t:"ذخیره و آماده‌سازی:", wz4_save:"ذخیره کن", wz4_note:"بعد از ذخیره، دکمه‌ی «اتصالِ رله» بالا فعال می‌شود.",
    please_wait:"لطفاً صبر کن…", failed:"ناموفق:", run_test_first:"اول «تست همه» را بزن", sub_added:"ساب اضافه شد", copied:"کپی شد", relay_conn_ok:"رله وصل شد", relay_conn_off:"رله قطع شد", gas_fill_first:"اول «کلید» (مرحله ۱) و «آدرسِ Worker» (مرحله ۲) را پر کن…", press_test:"«تست همه» را بزن", loading:"در حال بارگذاری…",
    your_net:"نتِ تو:", frag_on:"Fragment روشن شد", frag_off:"Fragment خاموش شد", copy_fail:"کپی نشد — متن را دستی انتخاب کن", need_key_sid:"کلید (۱) و Script ID (۳) لازم است", relay_saved:"رله ذخیره شد — حالا از دکمه‌ی بالا وصل شو", log_empty:"هنوز چیزی نیست…", fetching_servers:"در حال گرفتنِ سرورها…", fetch_fail:"گرفتن نشد:",
    mode_lbl:"حالت:", mode_proxy:"پراکسیِ سیستم (فقط مرورگر)", mode_tun:"TUN — همه‌ی برنامه‌ها (پیشنهادی)", mode_game:"گیم (کم‌تأخیر)",
    mode_hint_proxy:"فقط مرورگر و برنامه‌هایی که پراکسی را می‌خوانند. دیسکورد (صدا)، بازی‌ها و اپ‌های Store از این راه کار نمی‌کنند — برای آن‌ها TUN را بزن.", mode_hint_tun:"همه‌ی برنامه‌ها (بازی، تلگرام و…) از تونل می‌روند. نیاز به دسترسیِ ادمین (پنجره‌ی UAC Yes).", mode_hint_game:"⚠ عبور از VPN همیشه چند میلی‌ثانیه اضافه می‌کند؛ فقط وقتی پینگ را کم می‌کند که ISP مسیرِ مستقیم را کند کرده باشد (اپ بعدِ اتصال هر دو را می‌سنجد و می‌گوید کدام بهتر است). گیم‌بوست: کاهشِ تأخیر، نگه‌داشتنِ موقتِ آپدیت‌های ویندوز و آزادسازیِ پهنای باند برای بازی (موقعِ قطع همه برمی‌گردد). کلِ ترافیک از تونل؛ نیاز به ادمین (UAC Yes).",
    game_exe_lbl:"بازی‌های محبوب خودکار تشخیص داده و اولویت‌شان بالا می‌رود. اگر بازیت در لیست نبود، اسمِ پروسه‌اش را اینجا بنویس (اختیاری).",
    tun_starting:"در حال بالا آوردنِ تونل… (پنجره‌ی ادمین را تأیید کن)", game_starting:"در حال آماده‌سازیِ حالتِ گیم… (پنجره‌ی ادمین را تأیید کن)", tun_connected:"TUN متصل — کلِ سیستم از تونل", game_connected:"گیم متصل — کلِ سیستم از تونل",
    cancelled:"کنسل شد", cancel_hint:"برای کنسل، دکمه را دوباره بزن",
    stop_test:"توقفِ تست", test_stopped:"تست متوقف شد",
    relay_starting:"در حال بالا آوردنِ رله… (چند ثانیه؛ اولین بار گواهی هم نصب می‌شود)", relay_on_hint:"همه‌ی ترافیکِ مرورگر از رله می‌رود",
    matching_net:"در حال تطبیق با نتِ تو…", matched_n:"{n} کانفیگِ مناسبِ نتِ تو از {all} تا (صرفه‌جویی در دیتا)",
    frag_proxy_only:"Fragment فقط در حالتِ «پراکسیِ سیستم» کار می‌کند.", conn_lost:"اتصال قطع شد — دوباره وصل شو", already_testing:"تست در حال اجراست…",
    an_title:"تحلیلِ نت", an_hint:"می‌فهمد نتِ تو چطور فیلتر شده و چه نوع کانفیگی رویش بهتر کار می‌کند.",
    an_running:"در حال تحلیلِ نت… (چند ثانیه)", an_yes:"باز است", an_no:"بسته/محدود",
    an_udp:"عبورِ UDP", an_quic:"QUIC (پورت ۴۴۳ روی UDP)", an_sni:"فیلترینگ روی SNI",
    an_clean:"تمیز", an_dpi:"DPI فعال", an_dns:"سلامتِ DNS", an_poisoned:"مسموم",
    an_cdn:"دسترسی به کلادفلر", an_ports:"پورت‌های باز", an_advice:"نتیجه برای تو:",
    adv_udp_ok:"UDP باز است hysteria2/tuic بهترین گزینه‌اند (کم‌ترین پینگ و پکت‌لاس؛ عالی برای بازی).",
    adv_udp_limited:"UDP محدود است (QUIC رد نشد) hysteria2/tuic ممکن است کار نکند؛ روی reality/TLS بمان.",
    adv_udp_blocked:"UDP بسته است hysteria2/tuic کار نمی‌کند؛ فقط کانفیگ‌های TCP (reality یا ws-tls).",
    adv_sni_dpi:"فیلترینگ روی SNI فعال است کانفیگِ REALITY استفاده کن یا Fragment را روشن کن.",
    adv_dns_poison:"DNSِ نتت مسموم است DNSِ رمزی لازم است؛ اپ خودش این کار را می‌کند.",
    adv_no_443:"پورت ۴۴۳ مستقیم بسته است کانفیگ روی پورت‌های دیگر (۸۰/۸۰۸۰/۲۰۵۳) را امتحان کن.",
    adv_cdn_ok:"IPهای کلادفلر جواب می‌دهند کانفیگ‌های CDN/Worker روی نتت خوب کار می‌کنند.",
    bw_now:"مصرفِ لحظه‌ای", bw_cap:"ظرفیتِ خطِ تو: ↓{d} / ↑{u} مگابیت", bw_cap_none:"برای اندازه‌گیریِ ظرفیت، دکمه‌ی صاعقه را بزن",
    bw_testing:"در حال اندازه‌گیریِ سرعت…", bw_done:"سرعتِ خطِ تو: ↓{d} ↑{u} مگابیت/ثانیه", bw_need_conn:"اول وصل شو",
    wg_via_carrier:"UDP بسته است — WireGuard از داخلِ بهترین سرور رد می‌شود…", warp_via_carrier:"UDP بسته است — WARP از داخلِ بهترین سرور رد می‌شود…",
    warp_scanning:"در حال اسکنِ endpointهای کلادفلر… (چند ثانیه)", warp_found:"بهترین مسیر: {ep} ({ms}ms)", warp_need_server:"اول یک سرور را تست و انتخاب کن",
    mc_added_n:"کانفیگ اضافه شد", mc_none:"چیزی برای افزودن نبود", wg_bad:"کانفیگِ WireGuard خوانده نشد",
    adv_direct:"مستقیم {d}ms · از تونل {t}ms → برای این بازی مستقیم بهتر است (گیم‌مود را خاموش کن یا از «بایپسِ ایران» استفاده کن)",
    adv_tunnel:"مستقیم {d}ms · از تونل {t}ms → تونل بهتر است (ISP مسیرِ مستقیم را کند کرده)",
    adv_same:"مستقیم {d}ms · از تونل {t}ms → تقریباً یکسان",
    gool_trying:"امتحانِ gool (WARP داخلِ WARP)…", gool_ok:"gool متصل شد (WARP دولایه)",
    exit_iran_warn:"⚠ خروجی داخلِ ایران است — فیلترشکنی نمی‌شود:",
    warp_btn:"WARP", warp_starting:"در حال اتصال به WARP کلادفلر…", warp_ok:"WARP متصل شد (مجانی و نامحدود)",
    t_conn:"اتصال", t_lat:"تأخیر", t_srv:"سرورها", t_act:"فعالیتِ اینترنت", ms:"ms",
    up:"آپلود", down:"دانلود", act_empty:"هنوز سایتی باز نشده", act_off:"برای دیدنِ فعالیت وصل شو",
    set_apps:"اپ‌های ویندوز", uwp_fix:"بازکردنِ اپ‌های ویندوز",
    uwp_note:"اپ‌های Microsoft Store داخلِ قفلِ ویندوزند و به پراکسیِ محلی وصل نمی‌شوند. یک‌بار این را بزن تا باز شوند (نیاز به ادمین).",
    uwp_working:"در حال باز کردنِ اپ‌های ویندوز… (پنجره‌ی ادمین را تأیید کن)", uwp_done:"✅ اپ‌های ویندوز باز شدند — Store را دوباره باز کن",
    set_start:"شروعِ خودکار", set_start_lbl:"با روشن شدنِ ویندوز اجرا شود (در سینی)",
    start_on:"شروعِ خودکار روشن شد", start_off:"شروعِ خودکار خاموش شد",
    set_title:"تنظیمات", set_theme:"ظاهر", set_lang:"زبان", set_openlog:"نمایشِ لاگ", theme_set:"ظاهر عوض شد:",
    isp_pick:"نتِ خودت را انتخاب کن", isp_pick_hint:"تشخیصِ خودکار گاهی اشتباه می‌کند (مثلاً سامانتل روی خطِ رایتل است). نتِ درستت را بزن:", log_title:"لاگِ پشت‌صحنه", ready:"آماده",
    priv_title:"حریم خصوصی",
    priv_note:"شبگرد هیچ اطلاعاتی برای هیچ سروری نمی‌فرستد — بدون حساب، بدون ثبت‌نام، بدون آنالیتیکس. تنها رَدهای روی همین کامپیوتر: سابقهٔ فعالیت (کدام سایت‌ها باز شده) و فایل‌های موقت. با یک کلیک همه پاک می‌شوند.",
    priv_wipe:"پاک‌کردنِ داده‌های من", priv_done:"🧹 پاک شد — {n} فایل ({kb} KB)",
    upd_title:"به‌روزرسانی", upd_latest:"شبگرد {v} — به‌روز است ✅",
    upd_available:"نسخهٔ جدید {v} موجود است", upd_install:"دانلود و نصب",
    upd_downloading:"در حال دانلود… (چند دقیقه)", upd_check_fail:"بررسیِ به‌روزرسانی ناموفق (اینترنت؟)",
    pa_title:"کنترلِ برنامه‌ها (حالتِ TUN)",
    pa_hint:"برای هر برنامه بگو از تونل برود، مستقیم، یا کلاً بلاک شود. فقط در حالتِ TUN/گیم اعمال می‌شود. نامِ پروسه را بنویس (مثلاً chrome.exe) و حالتش را انتخاب کن.",
    pa_proxy:"از تونل", pa_direct:"مستقیم", pa_block:"بلاک",
  },
  en: {
    off:"Off", on:"Connected", brand:"Shabgard",
    tab_servers:"Servers", tab_myconfig:"My Config", tab_blackout:"Blackout",
    connect:"Connect", disconnect:"Disconnect", connecting:"Connecting…", connected_lbl:"Connected", pickfirst:"Pick a server first",
    heroName0:"No server selected", heroSub0:"Pick a server from the list",
    fragment:"Fragment — anti-DPI (bypass filtering)",
    testAll:"Test all", onlyWorking:"Working only", best:"Auto-connect to best", search:"Search…",
    mc_title:"Your config", mc_add:"Add config", mc_added:"Added configs",
    subs_title:"Subscriptions", sub_add:"Add sub",
    bo_title:"Blackout — personal relay", relay_connect:"Connect relay",
    exit_from:"Exit from:", servers_word:"servers", relay_row:"Relay (blackout)", checking_exit:"Checking exit location…",
    route_lbl:"Routing:", route_lbl2:"Routing", mode_lbl2:"Mode", route_global:"All via VPN", route_bypass:"Bypass Iran",
    mc_hint:"Paste config links (one per line):", mc_empty:"No configs added yet", sub_empty:"No subscriptions added", copy:"Copy",
    bo_desc:"When the internet is nearly down, the relay tunnels via Google + Cloudflare. Set it up once, then turn it on as a separate VPN (separate from the server list).",
    relay_state0:"Relay not set up yet", relay_hint0:"Do the steps below", relay_ready:"Relay is ready", relay_press:"Press the button to connect", relay_on:"Relay connected (separate VPN)", relay_disconnect:"Disconnect relay",
    wz1_t:"Make a secret key", wz1_b:"— any string (your secret between Google and Cloudflare):",
    wz2_t:"Create a Cloudflare Worker", wz2_a:"— go to", wz2_b:"Workers & Pages Create Worker. Give it a name; its address looks like", wz2_c:". Put that address here:", wz2_code:"Worker code (paste into the editor Deploy)",
    wz3_t:"Create a Google Apps Script", wz3_a:"— go to", wz3_b:"New project. Copy & paste this ready code (filled with your key and address):", wz3_code:"Code.gs (ready)", wz3_deploy:"Then top-right Deploy New deployment gear next to Select type Web app Who has access = Anyone Deploy.", wz3_id:"It gives an address like …/macros/s/AKfyc…/exec; put the AKfyc… part here:",
    oauth_t:"Google shows a warning", oauth_b:"(it is your own personal app, completely safe). Follow these steps:", oauth_1:"Authorize access pick your Gmail account.", oauth_2:"On the Google hasn't verified this app page click Advanced at the bottom.", oauth_3:"Click Go to … (unsafe).", oauth_4:"Finally click Continue / Allow to grant access.",
    wz4_t:"Save & prepare:", wz4_save:"Save", wz4_note:"After saving, the Connect relay button above becomes active.",
    please_wait:"Please wait…", failed:"Failed:", run_test_first:"Run Test all first", sub_added:"Subscription added", copied:"Copied", relay_conn_ok:"Relay connected", relay_conn_off:"Relay disconnected", gas_fill_first:"Fill the key (step 1) and Worker address (step 2) first…", press_test:"press Test all", loading:"Loading…",
    your_net:"Your net:", frag_on:"Fragment enabled", frag_off:"Fragment disabled", copy_fail:"Copy failed — select the text manually", need_key_sid:"Key (1) and Script ID (3) required", relay_saved:"Relay saved — now connect from the button above", log_empty:"Nothing yet…", fetching_servers:"Fetching servers…", fetch_fail:"Fetch failed:",
    mode_lbl:"Mode:", mode_proxy:"System proxy (browser only)", mode_tun:"TUN — all apps (recommended)", mode_game:"Game (low latency)",
    mode_hint_proxy:"Only the browser and proxy-aware apps. Discord voice, games and Store apps will NOT work this way — use TUN for those.", mode_hint_tun:"All apps (games, Telegram, …) go through the tunnel. Needs admin (accept the UAC prompt Yes).", mode_hint_game:"Lowest ping and packet-loss for gaming; all traffic via the tunnel. Needs admin (UAC Yes). Find the best server with Test all.",
    tun_starting:"Bringing up the tunnel… (accept the admin prompt)", game_starting:"Preparing Game mode… (accept the admin prompt)", tun_connected:"TUN connected — whole system via tunnel", game_connected:"Game connected — whole system via tunnel", game_exe_lbl:"Popular games are auto-detected and prioritized. If yours isn't listed, add its process name here (optional).",
    cancelled:"Cancelled", cancel_hint:"Press the button again to cancel",
    stop_test:"Stop test", test_stopped:"Test stopped",
    relay_starting:"Bringing up the relay… (a few seconds; first run also installs the certificate)", relay_on_hint:"All browser traffic goes through the relay",
    matching_net:"Matching to your network…", matched_n:"{n} configs match your network out of {all} (saves data)",
    frag_proxy_only:"Fragment only applies in System proxy mode.", conn_lost:"Connection lost — connect again", already_testing:"A test is already running…",
    an_title:"Network analysis", an_hint:"Finds out how your network is filtered and which config types work best on it.",
    an_running:"Analyzing your network… (a few seconds)", an_yes:"Open", an_no:"Blocked/limited",
    an_udp:"UDP passes", an_quic:"QUIC (UDP port 443)", an_sni:"SNI filtering",
    an_clean:"Clean", an_dpi:"DPI active", an_dns:"DNS integrity", an_poisoned:"Poisoned",
    an_cdn:"Cloudflare reachable", an_ports:"Open ports", an_advice:"What this means for you:",
    adv_udp_ok:"UDP is open hysteria2/tuic are your best choice (lowest ping and packet loss; great for gaming).",
    adv_udp_limited:"UDP is limited (QUIC didn't pass) hysteria2/tuic may not work; stick to reality/TLS.",
    adv_udp_blocked:"UDP is blocked hysteria2/tuic won't work; use TCP configs only (reality or ws-tls).",
    adv_sni_dpi:"SNI-based filtering is active use a REALITY config or turn Fragment on.",
    adv_dns_poison:"Your DNS is poisoned encrypted DNS is required; the app already does this.",
    adv_no_443:"Port 443 is blocked directly try configs on other ports (80/8080/2053).",
    adv_cdn_ok:"Cloudflare IPs respond CDN/Worker configs work well on your network.",
    bw_now:"Live usage", bw_cap:"Your line: ↓{d} / ↑{u} Mb/s", bw_cap_none:"Press the bolt to measure your line speed",
    bw_testing:"Measuring speed…", bw_done:"Your line: ↓{d} ↑{u} Mb/s", bw_need_conn:"Connect first",
    wg_via_carrier:"UDP is blocked — routing WireGuard through your best server…", warp_via_carrier:"UDP is blocked — routing WARP through your best server…",
    warp_scanning:"Scanning Cloudflare endpoints… (a few seconds)", warp_found:"Best route: {ep} ({ms}ms)", warp_need_server:"Test and pick a server first",
    mc_added_n:"config(s) added", mc_none:"Nothing to add", wg_bad:"Could not read the WireGuard config",
    adv_direct:"Direct {d}ms · via VPN {t}ms → direct is better for this game (turn Game mode off, or use Bypass Iran)",
    adv_tunnel:"Direct {d}ms · via VPN {t}ms → the tunnel is better (your ISP throttles the direct path)",
    adv_same:"Direct {d}ms · via VPN {t}ms → about the same",
    gool_trying:"Trying gool (WARP-in-WARP)…", gool_ok:"gool connected (double WARP)",
    exit_iran_warn:"⚠ Exit is inside Iran — not bypassing:",
    warp_btn:"WARP", warp_starting:"Connecting to Cloudflare WARP…", warp_ok:"WARP connected (free, unlimited)",
    t_conn:"Connection", t_lat:"Latency", t_srv:"Servers", t_act:"Internet activity", ms:"ms",
    up:"Upload", down:"Download", act_empty:"Nothing opened yet", act_off:"Connect to see activity",
    set_apps:"Windows apps", uwp_fix:"Unblock Windows apps",
    uwp_note:"Microsoft Store apps are sandboxed by Windows and cannot reach the local proxy. Run this once to allow them (needs admin).",
    uwp_working:"Unblocking Windows apps… (accept the admin prompt)", uwp_done:"✅ Windows apps unblocked — reopen the Store",
    set_start:"Startup", set_start_lbl:"Launch when Windows starts (to tray)",
    start_on:"Autostart enabled", start_off:"Autostart disabled",
    set_title:"Settings", set_theme:"Appearance", set_lang:"Language", set_openlog:"Show log", theme_set:"Theme:",
    isp_pick:"Pick your network", isp_pick_hint:"Auto-detection is sometimes wrong (e.g. SamanTel runs on Rightel's network). Pick your real network:", log_title:"Backstage log", ready:"Ready",
    priv_title:"Privacy",
    priv_note:"Shabgard sends nothing to any server — no account, no signup, no analytics. The only traces live on this PC: the activity history (which sites were opened) and temp files. One click wipes them all.",
    priv_wipe:"Wipe my data", priv_done:"🧹 Wiped — {n} files ({kb} KB)",
    upd_title:"Updates", upd_latest:"Shabgard {v} — up to date ✅",
    upd_available:"New version {v} available", upd_install:"Download & install",
    upd_downloading:"Downloading… (a few minutes)", upd_check_fail:"Update check failed (offline?)",
    pa_title:"Per-app control (TUN mode)",
    pa_hint:"Choose whether each app goes through the tunnel, directly, or is blocked. Applies in TUN/Game mode. Type the process name (e.g. chrome.exe) and pick a mode.",
    pa_proxy:"via tunnel", pa_direct:"direct", pa_block:"block",
  },
};
const t = (k) => (I18N[LANG] && I18N[LANG][k]) || I18N.fa[k] || k;

// ══════════════════════════════════════════════════════════════════════════
//  تم‌ها — ۷ ظاهر، از «تنظیمات» قابلِ تعویض. فقط متغیرهای CSS عوض می‌شود.
// ══════════════════════════════════════════════════════════════════════════
const THEMES = [
  { id:"linear",   fa:"لینیر (تیره)",   en:"Linear Dark",         c:["#5E6AD2","#25c26e","#0a0a0f"] },
  { id:"brutal",   fa:"بروتالیسم",       en:"Neo-Brutalism",       c:["#ffe14d","#7bf59a","#f3f0e7"] },
  { id:"bento",    fa:"بنتو",            en:"Bento Grid",          c:["#4f8cff","#2ecc71","#0e1014"] },
  { id:"oled",     fa:"سوئیسی (OLED)",   en:"OLED Swiss",          c:["#ffffff","#00e07a","#000000"] },
  { id:"clay",     fa:"کِلِی نرم",       en:"Soft Clay",           c:["#8b7bff","#71e8ad","#1c1b2e"] },
  { id:"material", fa:"متریال",          en:"Material Expressive", c:["#d0bcff","#9ff2c0","#14131a"] },
  { id:"tactile",  fa:"تکتایل (رامس)",   en:"Tactile / Rams",      c:["#e2571f","#1d6b3f","#d8d5cd"] },
];
const THEME_DEFAULT = "linear";
const curTheme = () => localStorage.getItem("theme") || THEME_DEFAULT;
function applyTheme(id) {
  if (!THEMES.some(x => x.id === id)) id = THEME_DEFAULT;
  document.documentElement.setAttribute("data-theme", id);
  localStorage.setItem("theme", id);
  renderThemeGrid();
}
function renderThemeGrid() {
  const g = document.querySelector("#themeGrid"); if (!g) return;
  const cur = curTheme();
  g.innerHTML = THEMES.map(x => `
    <button class="th ${x.id === cur ? "on" : ""}" data-theme-id="${esc(x.id)}">
      <span class="th-prev" style="background:${esc(x.c[2])}">
        <span class="th-dot" style="background:${esc(x.c[1])}"></span>
        <span class="th-bar" style="background:${esc(x.c[0])}"></span>
      </span>
      <span class="th-name">${esc(LANG === "fa" ? x.fa : x.en)}</span>
    </button>`).join("");
  g.querySelectorAll(".th").forEach(b => b.onclick = () => {
    applyTheme(b.getAttribute("data-theme-id"));
    status(t("theme_set") + " " + b.querySelector(".th-name").textContent);
  });
}
applyTheme(curTheme());

const bypassNow = () => (localStorage.getItem("route") || "global") === "bypass";
// حالتِ اتصال: proxy (پیش‌فرض) / tun / game
const connMode = () => localStorage.getItem("mode") || "proxy";

// «حامل» = یک سرورِ عادیِ TCP که WireGuard/WARP از داخلش رد می‌شود.
// روی نت‌هایی که UDP بسته است (مثلِ سامانتل) این تنها راهِ کارکردنشان است.
function pickCarrier() {
  const probe = netProbe();
  // اگر UDP باز است اصلاً حامل لازم نیست
  if (probe && probe.quic) return null;
  const cands = S.rows.filter(r =>
    r.link && !String(r.link).startsWith("wg://") && !String(r.link).startsWith("__") &&
    !/^(hysteria2|hy2|tuic):/i.test(r.link) && r.ping >= 0);
  cands.sort((a, b) => a.ping - b.ping);
  return cands.length ? cands[0].link : null;
}
function updateModeHint() {
  const h = $("#modeHint"); if (!h) return;
  const m = connMode();
  h.textContent = m === "tun" ? t("mode_hint_tun") : m === "game" ? t("mode_hint_game") : t("mode_hint_proxy");
  // فیلدِ اسمِ بازی فقط در حالتِ گیم
  const row = $("#gameExeRow");
  if (row) {
    row.style.display = m === "game" ? "flex" : "none";
    const inp = $("#gameExe"); if (inp) inp.value = localStorage.getItem("game_exe") || "";
    const lbl = $("#gameExeLbl"); if (lbl) lbl.textContent = t("game_exe_lbl");
  }
}

// نامِ کشورها (دو-زبانه) — کدِ کشور از remarkِ کانفیگ استخراج می‌شود
const CC_NAMES = {
  DE:{fa:"آلمان",en:"Germany"}, NL:{fa:"هلند",en:"Netherlands"}, FR:{fa:"فرانسه",en:"France"},
  GB:{fa:"انگلیس",en:"UK"}, UK:{fa:"انگلیس",en:"UK"}, US:{fa:"آمریکا",en:"USA"}, CA:{fa:"کانادا",en:"Canada"},
  FI:{fa:"فنلاند",en:"Finland"}, SE:{fa:"سوئد",en:"Sweden"}, NO:{fa:"نروژ",en:"Norway"}, DK:{fa:"دانمارک",en:"Denmark"},
  CH:{fa:"سوئیس",en:"Switzerland"}, AT:{fa:"اتریش",en:"Austria"}, PL:{fa:"لهستان",en:"Poland"}, CZ:{fa:"چک",en:"Czechia"},
  RU:{fa:"روسیه",en:"Russia"}, UA:{fa:"اوکراین",en:"Ukraine"}, TR:{fa:"ترکیه",en:"Turkey"}, IT:{fa:"ایتالیا",en:"Italy"},
  ES:{fa:"اسپانیا",en:"Spain"}, PT:{fa:"پرتغال",en:"Portugal"}, IE:{fa:"ایرلند",en:"Ireland"}, BE:{fa:"بلژیک",en:"Belgium"},
  RO:{fa:"رومانی",en:"Romania"}, BG:{fa:"بلغارستان",en:"Bulgaria"}, HU:{fa:"مجارستان",en:"Hungary"}, GR:{fa:"یونان",en:"Greece"},
  RS:{fa:"صربستان",en:"Serbia"}, HR:{fa:"کرواسی",en:"Croatia"}, SK:{fa:"اسلواکی",en:"Slovakia"}, SI:{fa:"اسلوونی",en:"Slovenia"},
  LT:{fa:"لیتوانی",en:"Lithuania"}, LV:{fa:"لتونی",en:"Latvia"}, EE:{fa:"استونی",en:"Estonia"}, MD:{fa:"مولداوی",en:"Moldova"},
  BR:{fa:"برزیل",en:"Brazil"}, AR:{fa:"آرژانتین",en:"Argentina"}, MX:{fa:"مکزیک",en:"Mexico"}, CL:{fa:"شیلی",en:"Chile"},
  AE:{fa:"امارات",en:"UAE"}, SA:{fa:"عربستان",en:"Saudi"}, QA:{fa:"قطر",en:"Qatar"}, KW:{fa:"کویت",en:"Kuwait"}, OM:{fa:"عمان",en:"Oman"},
  IN:{fa:"هند",en:"India"}, SG:{fa:"سنگاپور",en:"Singapore"}, JP:{fa:"ژاپن",en:"Japan"}, KR:{fa:"کره",en:"Korea"},
  HK:{fa:"هنگ‌کنگ",en:"Hong Kong"}, TW:{fa:"تایوان",en:"Taiwan"}, CN:{fa:"چین",en:"China"}, ID:{fa:"اندونزی",en:"Indonesia"},
  MY:{fa:"مالزی",en:"Malaysia"}, TH:{fa:"تایلند",en:"Thailand"}, VN:{fa:"ویتنام",en:"Vietnam"}, PH:{fa:"فیلیپین",en:"Philippines"},
  AU:{fa:"استرالیا",en:"Australia"}, NZ:{fa:"نیوزیلند",en:"New Zealand"}, ZA:{fa:"آفریقای‌جنوبی",en:"South Africa"},
  SC:{fa:"سیشل",en:"Seychelles"}, CY:{fa:"قبرس",en:"Cyprus"}, LU:{fa:"لوکزامبورگ",en:"Luxembourg"}, IS:{fa:"ایسلند",en:"Iceland"},
  AM:{fa:"ارمنستان",en:"Armenia"}, GE:{fa:"گرجستان",en:"Georgia"}, KZ:{fa:"قزاقستان",en:"Kazakhstan"}, IR:{fa:"ایران",en:"Iran"},
};
// نامِ ISPهای ایران به انگلیسی
const ISP_EN = {
  "رایتل / سامانتل":"Rightel / SamanTel", "همراه اول":"Hamrah-e Aval (MCI)", "ایرانسل":"Irancell",
  "مخابرات":"TCI", "شاتل":"Shatel", "آسیاتک":"AsiaTech", "پارس‌آنلاین":"ParsOnline", "مبین‌نت":"MobinNet",
  "های‌وب":"HiWEB", "صبانت":"Sabanet", "رسپینا":"Respina", "زیتل":"Zitel", "داتک":"Datak", "فن‌آوا":"Fanava",
  "افرانت":"Afranet", "پیشگامان":"Pishgaman", "آروان":"ArvanCloud", "کلادفلر":"Cloudflare", "نامشخص":"Unknown",
};
function localizeCountry(name) {
  const m = String(name).match(/\b([A-Z]{2})\b/);
  if (m && CC_NAMES[m[1]]) { const cc = m[1]; return `${CC_NAMES[cc][LANG] || CC_NAMES[cc].en} ${cc}`; }
  return name;
}
function localizeIsp(isp) { return (LANG === "en" && ISP_EN[isp]) ? ISP_EN[isp] : isp; }

function applyLang() {
  document.documentElement.dir = LANG === "fa" ? "rtl" : "ltr";
  document.documentElement.lang = LANG;
  $$("[data-i18n]").forEach(el => { el.textContent = t(el.getAttribute("data-i18n")); });
  const set = (sel, k) => { const e = $(sel); if (e) e.textContent = t(k); };
  set("#btnTest", "testAll"); set("#btnOnlyWorking", "onlyWorking"); set("#btnBest", "best");
  set("#tab-servers .switch .lbl", "fragment"); set("#routeLbl", "route_lbl2"); set("#modeLbl", "mode_lbl2");
  const rs = $("#routeSel");
  if (rs) {
    const cur = localStorage.getItem("route") || "global";
    rs.innerHTML = `<option value="global">${esc(t("route_global"))}</option><option value="bypass">${esc(t("route_bypass"))}</option>`;
    rs.value = cur;
  }
  const ms = $("#modeSel");
  if (ms) {
    ms.innerHTML = `<option value="proxy">${esc(t("mode_proxy"))}</option>`
      + `<option value="tun">${esc(t("mode_tun"))}</option>`
      + `<option value="game">${esc(t("mode_game"))}</option>`;
    ms.value = connMode();
    updateModeHint();
  }
  const s = $("#search"); if (s) s.placeholder = t("search");
  renderHero();
  updateConnect(); applyNetbar(); renderList();
  if (typeof renderMini === "function") renderMini();
  // عنوانِ پنجره (اسمِ برنامه) هم با زبان عوض شود
  try { if (typeof enhanceAllSelects === "function") enhanceAllSelects(); } catch (e) {}
  try { if (TAURI && window.__TAURI__.window) window.__TAURI__.window.getCurrentWindow().setTitle(t("brand")); } catch (e) {}
}

// آیا داخل Tauri هستیم؟ (وقتی هسته آماده شد invoke را صدا می‌زنیم)
const TAURI = !!(window.__TAURI__);
async function call(cmd, args) {
  if (TAURI) return window.__TAURI__.core.invoke(cmd, args);
  return mock(cmd, args); // در مرورگرِ ساده: دادهٔ نمونه
}

// ---------- دادهٔ نمونه (فقط برای پیش‌نمایشِ ظاهر) ----------
const FLAGS = { DE:"", NL:"", IT:"", FR:"", GB:"", US:"", FI:"", CA:"", CH:"", AE:"", TR:"", SE:"" };
let MOCK = [
  { link:"__relay__", cc:"", name:"رله (بلک‌اوت)", icon:"", ping:null },
  { link:"a", cc:"DE", name:"DE | Frankfurt", ping:118 },
  { link:"b", cc:"NL", name:"NL | Amsterdam", ping:142 },
  { link:"c", cc:"FI", name:"FI | Helsinki", ping:156 },
  { link:"d", cc:"FR", name:"FR | Paris", ping:171 },
  { link:"e", cc:"CH", name:"CH | Zurich", ping:188 },
  { link:"f", cc:"GB", name:"GB | London", ping:203 },
  { link:"g", cc:"SE", name:"SE | Stockholm", ping:224 },
  { link:"h", cc:"US", name:"US | New York", ping:262 },
  { link:"i", cc:"CA", name:"CA | Toronto", ping:288 },
  { link:"j", cc:"TR", name:"TR | Istanbul", ping:null },
];
function mock(cmd, args) {
  if (cmd === "list_servers") return Promise.resolve(MOCK);
  if (cmd === "test_all") return new Promise(r => setTimeout(() =>
    r((args?.links || []).map(() => 90 + Math.floor(Math.random()*400))), 700));
  if (cmd === "detect_net") return Promise.resolve({ isp:"سامانتل (نمونه)", ip:"5.1.2.3", cc:"IR", country:"Iran" });
  if (cmd === "exit_info") return Promise.resolve({ isp:"OVH", ip:"216.106.189.166", cc:"FR", country:"France · Paris" });
  if (cmd === "get_log") return Promise.resolve(["12:00:00  نمونه‌ی لاگ", "12:00:03 متصل شد"]);
  return Promise.resolve(null);
}

// ---------- وضعیت ----------
const S = { sel:null, connected:false, busy:false, onlyWorking:false, rows:[], filter:"", isp:"", detectedIsp:"", ip:"", selCc:"", selBestPing:null, exitInfo:null, testLinks:null, connectedAt:null, testing:false, usage:null, act:null, rate:null, prevU:null, livePing:null };

// مدتِ اتصال + نشانگرِ Kill Switch (پایینِ صفحه)
function fmtDur(ms) {
  const s = Math.floor(ms / 1000), h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = s % 60;
  return (h ? String(h).padStart(2, "0") + ":" : "") + String(m).padStart(2, "0") + ":" + String(sec).padStart(2, "0");
}
// مصرفِ داده‌ی واقعی (از آمارِ xray) — هر ۳ ثانیه، فقط وقتی وصلیم
function fmtBytes(n) {
  n = Number(n) || 0;
  if (n < 1024) return n + "B";
  if (n < 1048576) return (n / 1024).toFixed(0) + "KB";
  if (n < 1073741824) return (n / 1048576).toFixed(1) + "MB";
  return (n / 1073741824).toFixed(2) + "GB";
}
// ══════════════════════════════════════════════════════════════════════════
//  زمان‌بندِ کم‌مصرف — وقتی پنجره مخفی/مینیمایز است هیچ کاری نمی‌کند.
//  قبلاً هر ۳ ثانیه یک درخواستِ *واقعیِ اینترنتی* برای پینگ می‌رفت (حتی وقتی
//  اپ در سینی بود). حالا هر کار دوره‌ی خودش را دارد و همه با مخفی‌شدنِ پنجره
//  متوقف می‌شوند → مصرفِ CPU/GPU/دیتا نزدیکِ صفر در حالتِ بی‌کار.
// ══════════════════════════════════════════════════════════════════════════
const AWAKE = () => document.visibilityState === "visible";
const TICK = { usage: 0, act: 0, ping: 0, alive: 0 };
const EVERY = { usage: 5000, act: 7000, ping: 20000, alive: 6000 };

async function pollTick() {
  if (!TAURI || !AWAKE()) return;
  const now = Date.now();

  // وضعیتِ اتصال (ارزان، محلی)
  if (now - TICK.alive >= EVERY.alive && !S.busy) {
    TICK.alive = now;
    try {
      const on = await call("is_connected");
      if (on !== S.connected) {
        S.connected = on;
        if (!on) { S.connectedAt = null; S.exitInfo = null; S.usage = null; S.rate = null; status(t("conn_lost")); }
        updateConnect(); renderHero();
      }
    } catch (e) {}
  }
  if (!S.connected) { S.usage = null; S.act = null; S.rate = null; S.prevU = null; S.livePing = null; paintTiles(); return; }

  // مصرفِ داده (محلی — بدونِ ترافیکِ اینترنت)
  if (now - TICK.usage >= EVERY.usage) {
    TICK.usage = now;
    try {
      const u = await call("usage");
      if (S.prevU && now > S.prevU.t) {
        const dt = (now - S.prevU.t) / 1000;
        S.rate = [Math.max(0, u[0] - S.prevU.u[0]) * 8 / dt / 1e6,
                  Math.max(0, u[1] - S.prevU.u[1]) * 8 / dt / 1e6];
      }
      S.prevU = { u, t: now }; S.usage = u;
    } catch (e) {}
  }

  // فعالیتِ اینترنت (خواندنِ فایلِ محلی) — فقط وقتی تبِ سرورها بازست
  if (now - TICK.act >= EVERY.act && document.querySelector("#tab-servers.active")) {
    TICK.act = now;
    try { S.act = await call("activity", { limit: 14 }); } catch (e) {}
  }

  // پینگِ زنده — تنها کاری که واقعاً اینترنت مصرف می‌کند: هر ۲۰ ثانیه، نه ۳
  if (now - TICK.ping >= EVERY.ping) {
    TICK.ping = now;
    try { const lp = await call("live_ping"); if (lp >= 0) { S.livePing = lp; pushPingHist(lp); } } catch (e) {}
  }
  paintTiles();
}
setInterval(pollTick, 2500);
// با برگشتنِ پنجره فوراً یک‌بار تازه کن (تا کاربر داده‌ی کهنه نبیند)
let _trimT = null;
document.addEventListener("visibilitychange", () => {
  document.documentElement.classList.toggle("bg-idle", !AWAKE());
  if (AWAKE()) {
    if (_trimT) { clearTimeout(_trimT); _trimT = null; }
    TICK.usage = TICK.act = TICK.ping = TICK.alive = 0; pollTick();
  } else if (TAURI) {
    // ۵ ثانیه بعد از مخفی‌شدن حافظه را پس بده (اگر سریع برگشت، لغو می‌شود)
    _trimT = setTimeout(() => { call("trim_memory").catch(() => {}); _trimT = null; }, 5000);
  }
});

// کاشیِ «تأخیر» (پینگ + مصرفِ داده) و کاشیِ «فعالیتِ اینترنت»
function paintTiles() {
  // --- تأخیر ---
  const big = $("#latMs"), sub = $("#latSub"), wrap = big && big.parentElement;
  if (big && wrap) {
    let ms = null;
    if (S.connected && S.livePing != null && S.livePing >= 0) ms = S.livePing;   // زنده
    else if (S.connected && S.selBestPing != null && S.selBestPing >= 0) ms = S.selBestPing;
    else if (S.sel) { const r = S.rows.find(x => x.link === S.sel); if (r && r.ping >= 0) ms = r.ping; }
    wrap.classList.remove("y", "r", "n");
    if (ms == null) { big.textContent = "—"; wrap.classList.add("n"); }
    else {
      big.textContent = ms;
      if (ms > 300) wrap.classList.add("r"); else if (ms > 180) wrap.classList.add("y");
    }
    if (sub) {
      sub.textContent = S.connected
        ? (S.exitInfo && S.exitInfo.country ? t("exit_from") + " " + S.exitInfo.country : t("checking_exit"))
        : (S.sel ? t("press_test") : t("heroSub0"));
    }
  }
  // --- مصرفِ داده (کنارِ پینگ، نه پایینِ صفحه) ---
  const u = S.usage || [0, 0];
  const up = $("#upVal"), dn = $("#downVal");
  if (up) up.textContent = fmtBytes(u[0]);
  if (dn) dn.textContent = fmtBytes(u[1]);
  // --- پهنای باند: مصرفِ لحظه‌ای در برابرِ ظرفیتِ اندازه‌گیری‌شده ---
  const capD = Number(localStorage.getItem("bw_cap_dn") || 0);
  const capU = Number(localStorage.getItem("bw_cap_up") || 0);
  const r = S.rate || [0, 0];
  const setBar = (barId, valId, capId, mbps, cap) => {
    const b = $(barId), v = $(valId), cEl = $(capId);
    if (v) v.textContent = mbps >= 10 ? mbps.toFixed(0) : mbps.toFixed(1);
    if (cEl) cEl.textContent = cap > 0 ? " / " + (cap >= 10 ? cap.toFixed(0) : cap.toFixed(1)) : "";
    if (b) {
      const max = cap > 0 ? cap : Math.max(1, mbps * 1.4);
      const pct = Math.min(100, (mbps / max) * 100);
      b.style.width = pct.toFixed(1) + "%";
      b.style.background = pct > 85 ? "var(--red)" : pct > 60 ? "var(--amber)" : "var(--grn)";
    }
  };
  setBar("#bwDn", "#bwDnV", "#bwDnC", r[1] || 0, capD);
  setBar("#bwUp", "#bwUpV", "#bwUpC", r[0] || 0, capU);
  const capEl = $("#bwCap");
  if (capEl) capEl.textContent = (capD > 0 || capU > 0)
    ? t("bw_cap").replace("{d}", capD).replace("{u}", capU) : t("bw_cap_none");

  // --- فعالیتِ اینترنت ---
  const al = $("#actList");
  if (al) {
    const rows = S.act || [];
    al.innerHTML = rows.length
      ? rows.map(a => `<div class="act"><span class="act-dot"></span>
          <span class="act-host">${esc(a.host)}</span>
          <span class="act-n">${a.hits > 1 ? "\u00d7" + a.hits : ""}</span></div>`).join("")
      : `<div class="act-empty">${esc(S.connected ? t("act_empty") : t("act_off"))}</div>`;
  }
}

setInterval(() => {
  if (!AWAKE()) return;                    // پنجره مخفی است — کاری نکن
  const el = document.querySelector("#connStats"); if (!el) return;
  if (S.connected && S.connectedAt) {
    // صادقانه: Kill Switch (اتصالِ مجددِ خودکار) فقط در حالتِ پراکسی است؛ در TUN/گیم
    // اگر تونل بیفتد نمی‌شود بی‌UAC برش گرداند، پس اپ قطع را اعلام می‌کند.
    const m = connMode();
    const guard = m === "proxy" ? (LANG === "fa" ? "Kill Switch روشن" : "Kill Switch on")
                : m === "game"  ? (LANG === "fa" ? "گیم — کلِ سیستم" : "Game — whole system")
                                : (LANG === "fa" ? "TUN — کلِ سیستم" : "TUN — whole system");
    // مصرفِ داده به کاشیِ «تأخیر» منتقل شد؛ پایینِ صفحه فقط زمان و وضعیت
    el.textContent = fmtDur(Date.now() - S.connectedAt) + "  \u00b7  " + guard;
  } else el.textContent = "";
}, 1000);

// رندرِ زنده با debounce (موقعِ تستِ زنده، صدها به‌روزرسانی را جمع می‌کند)
let _renderTimer = null;
function scheduleRender() { if (_renderTimer) return; _renderTimer = setTimeout(() => { _renderTimer = null; renderList(); }, 350); }

// هدر (اسمِ کشور + خروج از) — با تغییرِ زبان هم درست به‌روز می‌شود
function renderHero() {
  try { paintTiles(); } catch (e) {}
  // در حالِ اتصال: به‌جای متنِ عادی، راهنمای کنسل را نشان بده
  if (S.busy) { $("#heroName").textContent = t("connecting"); $("#heroSub").textContent = t("cancel_hint"); return; }
  if (!S.sel) { $("#heroName").textContent = t("heroName0"); $("#heroSub").textContent = t("heroSub0"); return; }
  // وقتی متصلیم و محلِ واقعیِ خروج را داریم، همان کشورِ واقعی را نشان بده (دقیق‌تر از برچسبِ کانفیگ)
  const cc = (S.connected && S.exitInfo && S.exitInfo.cc) ? S.exitInfo.cc : S.selCc;
  $("#heroName").textContent = cc ? ccName(cc) : "—";
  if (S.connected && S.exitInfo && S.exitInfo.country) {
    const inIran = String(S.exitInfo.cc || "").toUpperCase() === "IR";
    $("#heroSub").textContent = (inIran ? t("exit_iran_warn") + " " : t("exit_from") + " ")
      + S.exitInfo.country + (S.exitInfo.ip ? " (" + S.exitInfo.ip + ")" : "");
    $("#heroSub").classList.toggle("warn", inIran);
  }
  else if (S.connected) $("#heroSub").textContent = t("checking_exit");
  else $("#heroSub").textContent = (S.selBestPing != null && S.selBestPing >= 0)
    ? ((LANG === 'fa' ? 'بهترین پینگِ این کشور: ' : 'Best ping: ') + S.selBestPing + 'ms') : t("heroSub0");
}

// ---------- یادگیریِ کانفیگ‌های کارآمد برای هر ISP ----------
function ispKey() { return "ok_" + (S.isp || "?"); }
function rememberWorking(links) { try { localStorage.setItem(ispKey(), JSON.stringify(links)); } catch (e) {} }
function rememberedSet() { try { return new Set(JSON.parse(localStorage.getItem(ispKey()) || "[]")); } catch (e) { return new Set(); } }

// ══════════════════════════════════════════════════════════════════════════
//  کشِ نتیجهٔ تست — «کانفیگ‌های مرده را فراموش کن، زنده‌ها را اول بیاور»
//  کانفیگ‌ها از تلگرام/گیت‌هاب می‌آیند و بیشترشان بعد چند روز می‌میرند. هر بار
//  تستِ کامل از صفر = هدر دادنِ دیتا روی جسدها. اینجا نتیجهٔ هر لینک با زمان
//  ذخیره می‌شود: مردهٔ قدیمی (>۲ روز) از لیست حذف می‌شود، زندهٔ قدیمی اول لیست
//  می‌آید و پینگِ کهنه‌اش نشان داده می‌شود تا کاربر بداند تقریبی است.
// ══════════════════════════════════════════════════════════════════════════
const PING_CACHE_KEY = "ping_cache";
const DEAD_FORGET_MS = 2 * 24 * 3600 * 1000;   // مردهٔ ۲ روزه → حذف
function loadPingCache() {
  try {
    const c = JSON.parse(localStorage.getItem(PING_CACHE_KEY) || "{}");
    // پاکسازی دوره‌ای خودِ کش (جلوی رشدِ بی‌نهایت)
    const now = Date.now();
    for (const k of Object.keys(c)) if (now - c[k].t > 14 * 24 * 3600 * 1000) delete c[k];
    return c;
  } catch (e) { return {}; }
}
function savePingCache(c) { try { localStorage.setItem(PING_CACHE_KEY, JSON.stringify(c)); } catch (e) {} }
// بعد از هر تستِ کامل صدا زده می‌شود — فقط لینک‌هایی که الان واقعاً تست شدند
function updatePingCache(links, pings) {
  const c = loadPingCache(); const now = Date.now();
  links.forEach((l, i) => {
    const p = pings[i] ?? -1;
    if (p >= 0 || c[l]) c[l] = { p, t: now };   // مردهٔ جدید هم ثبت شود تا «کهنه» شناخته شود
    else delete c[l];                            // قبلاً نبود و حالا مرده است → ثبت
  });
  savePingCache(c);
}
// اعمالِ کش روی ردیف‌ها: مردهٔ کهنه حذف، زندهٔ کهنه اول + پینگِ کهنه با نشانگر
async function applyPingCache(rows) {
  const c = loadPingCache();
  const now = Date.now();
  let dropped = 0;
  let out = rows.filter(r => {
    if (r.link === "__relay__") return true;
    const e = c[r.link];
    if (!e) return true;                          // بیسابقه → بماند
    if (e.p < 0 && now - e.t > DEAD_FORGET_MS) { dropped++; return false; }   // مردهٔ کهنه → حذف
    return true;
  });
  out.forEach(r => {
    const e = c[r.link];
    if (e && e.p >= 0 && r.ping == null) { r.ping = e.p; r.stale = true; }
  });
  // زنده‌ها (تازه یا کهنه) بالای لیست گروه‌بندی می‌شوند چون groupedRows بر اساس bestPing سورت می‌کند
  if (dropped > 0) logUi(LANG==='fa'?`از کش: ${dropped} کانفیگِ مردهٔ کهنه حذف شد`:`cache: ${dropped} long-dead configs dropped`);
  savePingCache(c);
}

// ---------- «کانفیگِ روز» — بهترین ۵ برای اشتراک با دوستان ----------
const DAILY_KEY = "daily5";
function updateDaily5(cfgs) {
  const ok = cfgs.filter(r => r.ping != null && r.ping >= 0)
                 .sort((a, b) => a.ping - b.ping).slice(0, 5)
                 .map(r => ({ link: r.link.split("#")[0], ping: r.ping }));
  if (ok.length >= 3) { try { localStorage.setItem(DAILY_KEY, JSON.stringify({ at: Date.now(), list: ok })); } catch (e) {} }
  renderDailyBtn();
}
function dailyText() {
  try {
    const d = JSON.parse(localStorage.getItem(DAILY_KEY) || "null");
    if (!d || !d.list.length) return "";
    const when = new Date(d.at).toLocaleDateString(LANG==='fa'?'fa-IR':'en-US');
    const head = LANG==='fa' ? `🌙 شبگرد — ${when} — ${d.list.length} سرورِ برتر:\n`
                             : `Shabgard — ${when} — top ${d.list.length} servers:\n`;
    return head + d.list.map((s,i)=>`${i+1}. ${s.link}`).join("\n");
  } catch (e) { return ""; }
}

const ISP_LIST = ["رایتل / سامانتل","همراه اول","ایرانسل","مخابرات","شاتل","آسیاتک",
  "پارس‌آنلاین","مبین‌نت","های‌وب","صبانت","رسپینا","زیتل","داتک","فن‌آوا","افرانت","پیشگامان","سایر"];

function applyNetbar() {
  const overridden = !!localStorage.getItem("isp_override");
  const label = LANG === "fa" ? "نتِ تو" : "Your net";
  const editT = overridden ? (LANG === "fa" ? "دستی" : "manual") : (LANG === "fa" ? "تغییر" : "change");
  const unknown = LANG === "fa" ? "نامشخص" : "Unknown";
  $("#netbar").innerHTML = `${label}: <b>${esc(localizeIsp(S.isp) || unknown)}</b>` +
    (S.ip ? ` <span class="dim">(${esc(S.ip)})</span>` : "") +
    ` <span class="edit-isp">${editT}</span>`;
}

async function detectNet() {
  const override = localStorage.getItem("isp_override");
  if (override) S.isp = override;
  try {
    const net = await call("detect_net");
    S.detectedIsp = net.isp || "نامشخص";
    S.ip = net.ip || "";
    if (!override) S.isp = S.detectedIsp;
  } catch (e) { if (!override) S.isp = "نامشخص"; }
  applyNetbar();
  renderList(); // تا ⭐های نتِ تو نمایش داده شوند
}

function openIspPicker() {
  const grid = $("#ispGrid"); grid.innerHTML = "";
  const cur = localStorage.getItem("isp_override");
  const opts = ["خودکار (" + (S.detectedIsp || "?") + ")", ...ISP_LIST];
  opts.forEach((name, i) => {
    const b = document.createElement("button");
    b.className = "isp-opt" + (((i === 0) && !cur) || name === cur ? " on" : "");
    b.textContent = name;
    b.onclick = () => {
      if (i === 0) { localStorage.removeItem("isp_override"); S.isp = S.detectedIsp || "نامشخص"; }
      else { localStorage.setItem("isp_override", name); S.isp = name; }
      applyNetbar(); renderList();
      $("#ispModal").classList.remove("open");
      status(t("your_net") + " " + S.isp);
    };
    grid.appendChild(b);
  });
  $("#ispModal").classList.add("open");
}
$("#netbar").onclick = openIspPicker;
$("#ispClose").onclick = () => $("#ispModal").classList.remove("open");
$("#ispModal").onclick = (e) => { if (e.target.id === "ispModal") $("#ispModal").classList.remove("open"); };

// ---------- تب‌ها ----------
$$(".tab").forEach(t => t.onclick = () => {
  $$(".tab").forEach(x => x.classList.remove("active"));
  $$(".panel").forEach(x => x.classList.remove("active"));
  t.classList.add("active");
  $("#tab-" + t.dataset.tab).classList.add("active");
});

// ---------- لیستِ سرورها ----------
function pingClass(p){ if(p==null) return ""; if(p<0) return "bad"; if(p<200) return "good"; if(p<400) return "mid"; return "bad"; }
function pingText(p){ if(p==null) return ""; if(p<0) return ""; return p+"ms"; }
// پینگِ کهنه (از کشِ دفعهٔ قبل) با «≈» مشخص می‌شود تا کاربر بداند تقریبی است
function stalePingText(r){ const base = pingText(r.ping); return (r.stale && r.ping >= 0) ? "≈" + base : base; }

const ccName = (cc) => {
  if (!cc || cc === "??") return LANG === "fa" ? "سایر" : "Other";
  const n = CC_NAMES[cc]; return n ? (n[LANG] || n.en) : cc;
};
// تشخیصِ کشور: ۱) پرچمِ ایموجی (🇩🇪 = دقیق‌ترین؛ ساب‌ها تقریباً همیشه دارند)
// ۲) کدِ ۲-حرفی که واقعاً در CC_NAMES باشد (کلماتی مثل VLESS/DNS/USِ داخل USA را
//    قبلاً کشور می‌گرفت و «USA نوشته ولی آلمان بود» می‌شد)
const FLAG_RE = /[\u{1F1E6}-\u{1F1FF}][\u{1F1E6}-\u{1F1FF}]/u;
function countryOf(r) {
  if (r.cc) return r.cc;
  const name = String(r.name);
  const fm = name.match(FLAG_RE);
  if (fm) {
    // 🇩🇪 → DE (regional indicator → حرف)
    const cc = String.fromCodePoint(...[...fm[0]].map(c => c.codePointAt(0) - 0x1F1E6 + 65));
    if (CC_NAMES[cc]) return cc;
  }
  const m = name.match(/\b([A-Z]{2})\b/g);
  if (m) { const hit = m.find(x => CC_NAMES[x]); if (hit) return hit; }
  return "??";
}

// گروه‌بندی per کشور: یک ردیف per کشور، بهترین کانفیگِ کارآمد داخلش. کشورِ همه‌مرده → غیب.
// کانفیگ‌های دستیِ خودت (از حافظه‌ی محلی؛ مستقل از ترتیبِ بارگذاری)
function mineLinks() {
  try { return new Set(JSON.parse(localStorage.getItem("manual") || "[]")); }
  catch (e) { return new Set(); }
}

function groupedRows() {
  const g = {};
  const mine = mineLinks();   // کانفیگ‌های خودت بخشِ جدا دارند → اینجا تکرار نشوند
  for (const r of S.rows) {
    if (r.link === "__relay__" || mine.has(r.link)) continue;
    const cc = countryOf(r);
    (g[cc] = g[cc] || { cc, configs: [] }).configs.push(r);
  }
  let arr = Object.values(g).map(x => {
    const working = x.configs.filter(r => r.ping != null && r.ping >= 0).sort((a, b) => a.ping - b.ping);
    const tested = x.configs.filter(r => r.ping != null).length;
    const best = working[0] || null;
    return { cc: x.cc, count: x.configs.length, configs: x.configs,
             bestLink: best ? best.link : x.configs[0].link,
             bestPing: best ? best.ping : null,
             dead: (tested === x.configs.length && working.length === 0) };
  });
  // ⚠️ اگر «همهٔ» گروه‌ها مرده باشند، فیلتر کردن یعنی لیستِ کاملاً خالی — تجربهٔ
  // وحشتناک («سرورها حذف شدند!»). در این حالت فقط زنده‌ها اول، مرده‌ها ته با رنگ.
  const anyAlive = arr.some(x => !x.dead);
  if (anyAlive) arr = arr.filter(x => !x.dead);
  arr.sort((a, b) => {
    const da = a.dead ? 1 : 0, db = b.dead ? 1 : 0;      // مرده‌ها ته
    if (da !== db) return da - db;
    return (a.bestPing != null && a.bestPing >= 0 ? a.bestPing : 1e9)
         - (b.bestPing != null && b.bestPing >= 0 ? b.bestPing : 1e9);
  });
  return arr;
}

function renderList() {
  const el = $("#serverList"); el.innerHTML = "";
  const remembered = rememberedSet();

  // ── بخشِ جدا برای کانفیگ‌های خودت (تکی، نه گروهیِ کشوری) تا گم نشوند ──
  const mineSet = mineLinks();
  if (mineSet.size) {
    let mineRows = S.rows.filter(r => mineSet.has(r.link));
    if (S.filter) mineRows = mineRows.filter(r => String(r.name).toLowerCase().includes(S.filter));
    if (S.onlyWorking) mineRows = mineRows.filter(r => r.ping != null && r.ping >= 0);
    if (mineRows.length) {
      const head = document.createElement("div");
      head.className = "grp-head";
      head.textContent = `${t("mc_added")} (${mineRows.length})`;
      el.appendChild(head);
      for (const r of mineRows) {
        const d = document.createElement("div");
        d.className = "srv mine" + (r.link === S.sel ? " sel" : "");
        d.innerHTML = `<span class="ping ${pingClass(r.ping)}">${stalePingText(r)}</span>
                       <span class="srv-name">${esc(r.name)}</span>
                       <span class="flag">${FLAGS[countryOf(r)] || ""}</span>`;
        d.onclick = () => selectServer(r.link);
        d.ondblclick = () => { selectServer(r.link); toggleConnect(); };
        el.appendChild(d);
      }
      const sep = document.createElement("div");
      sep.className = "grp-head";
      sep.textContent = `${t("tab_servers").replace(" ", "")}`;
      el.appendChild(sep);
    }
  }

  let groups = groupedRows();
  if (S.filter) groups = groups.filter(x => ccName(x.cc).toLowerCase().includes(S.filter) || x.cc.toLowerCase().includes(S.filter));
  if (S.onlyWorking && !groups.every(x => x.dead)) groups = groups.filter(x => !x.dead);
  for (const x of groups) {
    const sel = x.configs.some(c => c.link === S.sel);
    const d = document.createElement("div");
    // گروهِ همه‌مرده (فقط وقتی همهٔ لیست مرده است نمایش داده میشود) با محوشدگی
    d.className = "srv" + (sel ? " sel" : "") + (x.dead ? " deadgrp" : "");
    const star = remembered.has(x.bestLink) ? `<span class="star" title="${LANG==='fa'?'برای نتِ تو کار کرد':'works for your net'}"></span>` : "";
    const bestRow = x.bestLink && S.rows.find(r => r.link === x.bestLink);
    const pingLabel = x.dead ? (LANG === 'fa' ? "✕" : "✕") : (bestRow ? stalePingText(bestRow) : pingText(x.bestPing));
    d.innerHTML = `<span class="ping ${x.dead ? 'bad' : pingClass(x.bestPing)}">${pingLabel}</span>
                   <span class="srv-name">${star}${esc(ccName(x.cc))} <span class="cnt">${x.count}</span></span>
                   <span class="flag">${FLAGS[x.cc] || ""}</span>`;
    d.onclick = () => selectCountry(x);
    d.ondblclick = () => { selectCountry(x); toggleConnect(); };
    el.appendChild(d);
  }
  $("#serverCount").textContent = `${groups.length} ${LANG === 'fa' ? 'کشور' : 'countries'}`;
}

function selectCountry(x) {
  S.sel = x.bestLink; S.selCc = x.cc; S.selBestPing = x.bestPing;
  renderHero(); renderList(); updateConnect();
}

// انتخابِ یک لینکِ مشخص (برای test_all/connect_best)
function selectServer(link) {
  S.sel = link;
  const r = S.rows.find(x => x.link === link);
  if (r) { S.selCc = countryOf(r); S.selBestPing = r.ping; }
  renderHero(); renderList(); updateConnect();
}

// ---------- اتصال ----------
// اتصالِ واقعی با رعایتِ حالت (پراکسی / TUN / گیم). قبلاً Fragment و Routing
// مستقیم connect صدا می‌زدند و کاربر را بی‌خبر از TUN می‌انداختند بیرون.
async function doConnect(link) {
  const mode = connMode();
  if (mode === "tun" || mode === "game") {
    status(t(mode === "game" ? "game_starting" : "tun_starting"));
    let apps = [];
    try { apps = JSON.parse(localStorage.getItem("pa_rules") || "[]"); } catch (e) {}
    await call("connect_tun", { link, game: mode === "game", bypass: bypassNow(),
      boost: mode === "game", gameExe: localStorage.getItem("game_exe") || "", apps });
    status(t(mode === "game" ? "game_connected" : "tun_connected"));
  } else {
    await call("connect", { link, fragment: $("#swFragment").checked, bypass: bypassNow() });
    status(t("connected_lbl"));
  }
}

// وصل‌شدنِ دوباره با تنظیماتِ جدید (تعویضِ Fragment یا Routing وسطِ اتصال)
async function reconnectCurrent() {
  if (!S.connected || !S.sel || S.sel === "__relay__") return;
  S.busy = true; updateConnect(); renderHero();
  try {
    await doConnect(S.sel);
    S.connectedAt = Date.now(); S.exitInfo = null;
    call("exit_info").then(info => {
      if (S.connected && info && info.country) { S.exitInfo = info; renderHero(); }
    }).catch(() => {});
  } catch (e) {
    S.connected = false; S.connectedAt = null;
    status(t("failed") + " " + e);
  }
  S.busy = false; updateConnect(); renderHero();
}

function updateConnect() {
  const b = $("#connectBtn"), l = $("#connectLabel");
  b.classList.remove("connected","busy","idle","notready");
  if (S.busy){ b.classList.add("busy"); l.textContent=t("connecting"); b.title = t("cancel_hint"); }
  else if (S.connected){ b.classList.add("connected"); l.textContent=t("connected_lbl"); }
  else if (S.sel){ b.classList.add("notready"); l.textContent=t("connect"); }
  else { b.classList.add("idle"); l.textContent=t("pickfirst"); }
  const bd = $("#badge");
  bd.classList.toggle("on", S.connected);
  $("#badgeText").textContent = S.connected ? t("on") : t("off");
}

// لینک‌های بیرونی را در مرورگرِ سیستم باز کن
document.addEventListener("click", (e) => {
  const a = e.target.closest("a.lnk");
  if (!a) return;
  e.preventDefault();
  const url = a.getAttribute("href");
  if (TAURI) call("open_url", { url }); else window.open(url, "_blank");
});

async function toggleConnect() {
  // اگر در حالِ اتصال است، این کلیک یعنی «کنسل» (دیگر زردِ گیرکرده نداریم)
  if (S.busy) {
    try { await call("cancel_connect"); } catch (e) {}
    try { await call("disconnect"); } catch (e) {}
    S.busy = false; S.connected = false; S.connectedAt = null; S.exitInfo = null;
    status(t("cancelled")); updateConnect(); renderHero();
    return;
  }
  if (!S.connected && !S.sel) return;
  S.busy = true; updateConnect(); status(t("please_wait"));
  try {
    if (S.connected) {
      await call("disconnect"); S.connected = false; S.connectedAt = null; S.exitInfo = null; status(t("disconnect")); renderHero();
    } else {
      await doConnect(S.sel);
      S.connected = true; S.connectedAt = Date.now(); S.exitInfo = null; renderHero();
      call("exit_info").then(info => {
        if (S.connected && info && info.country) {
          // ⚠️ برچسبِ کشورِ کانفیگ (از پرچم ساب تلگرامی) اغلب غلط است — مثلاً
          // سرور US با مسیرِ آلمان. کشورِ *واقعی* خروجی معیار است.
          if (info.cc && S.selCc && info.cc !== S.selCc) {
            const row = S.rows.find(r => r.link === S.sel);
            if (row) { row.cc = info.cc; }
            S.selCc = info.cc;
          }
          S.exitInfo = info; renderHero();
        }
      }).catch(() => {});
      // در حالتِ گیم، به‌جای ادعا عددِ واقعی بده: مستقیم بهتر است یا از تونل؟
      if (connMode() === "game") {
        call("route_advice").then(a => {
          if (!a || a.direct_ms <= 0) return;
          const msg = a.better === "direct" ? t("adv_direct") : a.better === "tunnel" ? t("adv_tunnel") : t("adv_same");
          status(msg.replace("{d}", a.direct_ms).replace("{t}", a.tunnel_ms));
        }).catch(() => {});
      }
    }
  } catch (e) { status(t("failed") + " " + e); }
  S.busy = false; updateConnect();
}
$("#connectBtn").onclick = toggleConnect;

// ---------- دکمه‌ها ----------
// ══════════════════════════════════════════════════════════════════════════
//  تطبیقِ نت — «شبیه‌ساز»
//  گیت‌هاب هر کانفیگِ تأییدشده را برچسب زده (manifest.json: پروتکل، پورت،
//  transport، REALITY، پشتِ CDN بودن، UDP لازم است یا نه). اینجا آن برچسب‌ها
//  را با نتیجه‌ی تحلیلِ نتِ خودت تطبیق می‌دهیم و کانفیگ‌های بی‌فایده را از تست
//  کنار می‌گذاریم — به‌جای تستِ ۴۰۰ کانفیگ، ۳۰ تا. این همان چیزی است که
//  دیتای تو را می‌خورد.
//  صادقانه: گیت‌هاب فیلترینگِ ایران را نمی‌بیند (سرورش بیرون است)، پس نمی‌تواند
//  بگوید کدام از داخلِ ایران باز می‌شود؛ ولی این تطبیق ۹۰٪ گزینه‌های محکوم‌به‌شکست
//  را حذف می‌کند.
// ══════════════════════════════════════════════════════════════════════════
// manifest — آدرس base64 تا از رشته‌های ساده قابل‌خواندن نباشد
const MANIFEST_URL = atob("aHR0cHM6Ly9yYXcuZ2l0aHVidXNlcmNvbnRlbnQuY29tL21ydDBwMGwvY29uZmlnLWNsb3VkL21haW4vbWFuaWZlc3QuanNvbg==");
let MANIFEST = null;

function netProbe() {
  try { return JSON.parse(localStorage.getItem("netprobe") || "null"); } catch (e) { return null; }
}

// امتیازِ تناسبِ یک کانفیگ با نتِ کاربر (بزرگ‌تر = بهتر). null یعنی اصلاً نمی‌ارزد.
function fitScore(f, probe) {
  if (!probe) return 0;
  let s = 0;
  // hysteria2/tuic روی UDP کار می‌کنند — اگر UDP بسته است، محکوم به شکست‌اند
  if (f.udp) {
    if (probe.quic) s += 40;            // QUIC باز → بهترین گزینه (کم‌پینگ، ضدِ پکت‌لاس)
    else if (probe.udp_dns) s += 5;     // UDP نیمه‌باز → شاید
    else return null;                   // UDP بسته → اصلاً تستش نکن
  }
  // فیلترینگ روی SNI → REALITY جواب می‌دهد، TLSِ ساده معمولاً نه
  if (probe.sni_block) { if (f.re) s += 30; else if (f.tls) s -= 15; }
  else if (f.re) s += 8;
  // اگر IPهای کلادفلر مستقیم جواب می‌دهند، کانفیگ‌های پشتِ CDN روی این نت خوب‌اند
  if (f.cdn) s += probe.cdn_ok ? 22 : -8;
  // پورت‌هایی که روی نتِ کاربر واقعاً باز دیده شده‌اند
  const ports = probe.ports || [];
  if (ports.length) s += ports.includes(f.p) ? 15 : -10;
  // تأخیرِ اندازه‌گیری‌شده‌ی گیت‌هاب (کم‌تر بهتر)
  if (typeof f.ms === "number") s += Math.max(0, 25 - f.ms / 20);
  return s;
}

async function loadManifest() {
  if (MANIFEST) return MANIFEST;
  try {
    // timeout کوتاه — گیت‌هاب در ایران/قطعیِ نت معمولا در دسترس نیست؛ بدونِ این،
    // دکمهٔ «تست همه» تا تایم‌اوتِ طولانیِ مرورگر قفل می‌ماند.
    const r = await fetch(MANIFEST_URL, { cache: "no-store", signal: AbortSignal.timeout(8000) });
    const j = await r.json();
    MANIFEST = new Map((j.configs || []).map(c => [c.u, c]));
  } catch (e) { MANIFEST = new Map(); }
  return MANIFEST;
}

// از میانِ ردیف‌ها فقط آن‌هایی را برمی‌گرداند که به نتِ کاربر می‌خورند
async function pickForMyNet(rows, cap) {
  const probe = netProbe();
  if (!probe) return null;                       // هنوز تحلیل نکرده — همه را تست کن
  const man = await loadManifest();
  if (!man.size) return null;
  const scored = [];
  for (const r of rows) {
    const f = man.get(String(r.link).split("#")[0]);
    if (!f) continue;                            // در manifest نیست (کانفیگِ دستیِ خودت)
    const sc = fitScore(f, probe);
    if (sc === null) continue;                   // محکوم به شکست روی این نت
    scored.push([sc, r]);
  }
  if (scored.length < 8) return null;            // داده کم است — سخت‌گیری نکن
  scored.sort((a, b) => b[0] - a[0]);
  return scored.slice(0, cap).map(x => x[1]);
}

$("#btnTest").onclick = async () => {
  // تست دانه‌دانه انجام می‌شود، پس همین دکمه وسطِ کار «توقف» می‌شود
  if (S.testing) {
    try { await call("cancel_connect"); } catch (e) {}
    status(t("test_stopped"));
    return;
  }
  const all = S.rows.filter(r => r.link !== "__relay__");
  if (!all.length) { status(LANG==='fa'?"سروری برای تست نیست":"No servers"); return; }
  S.testing = true; $("#btnTest").textContent = t("stop_test");
  // فقط کانفیگ‌هایی که به نتِ خودت می‌خورند (صرفه‌جویی در دیتا) — اگر تحلیلِ نت
  // نداریم یا manifest نیامد، به حالتِ قدیمی (همه) برمی‌گردیم.
  status(t("matching_net"));
  const picked = await pickForMyNet(all, 90);
  const cfgs = picked || all;
  if (picked) status(t("matched_n").replace("{n}", picked.length).replace("{all}", all.length));
  S.testLinks = cfgs.map(r => r.link);
  cfgs.forEach(r => r.ping = null); // در حال تست
  renderList();
  status(LANG==='fa'?"در حال تست… (نتایج زنده می‌آیند)":"Testing… (live results)");
  let pings = [];
  let cancelled = false;
  try { pings = await call("test_all", { links: S.testLinks }); }
  catch (e) { status(t("failed") + " " + e); cancelled = true; }
  finally { S.testing = false; $("#btnTest").textContent = t("testAll"); }
  if (Array.isArray(pings) && pings.length && pings.every(p => p === -1)) cancelled = true;   // رد شدن از ورودیِ دوم/کنسل
  // نهایی (اگر eventی جا ماند)
  cfgs.forEach((r, i) => { if (r.ping == null && !cancelled) r.ping = (pings[i] ?? -1); });
  rememberWorking(cfgs.filter(r => r.ping >= 0).map(r => r.link));
  // کش فقط با نتیجهٔ کامل به‌روز شود — کشِ «همه -۱»ی کنسل‌شده، زنده‌های شناخته‌شده را می‌کُشد
  if (!cancelled) {
    updatePingCache(S.testLinks, pings);
    updateDaily5(cfgs);
    try { localStorage.setItem("last_test_at", String(Date.now())); } catch (e) {}
  }
  renderList(); if (S.sel) selectServer(S.sel);
  const ok = cfgs.filter(r => r.ping >= 0).length;
  status((LANG==='fa'?`تست تمام شد — ${ok} سرورِ کارآمد برای `:`Done — ${ok} working for `) + (localizeIsp(S.isp) || "?"));
};
$("#btnRefresh").onclick = () => loadServers();

// ---------- پرفراپ — کنترلِ per-app در TUN ----------
// قاعده: "<مقدار>:<p|d|b>:<n|p>"  → n=نامِ پروسه، p=مسیرِ کامل (برای UWP)
let paRules = [];
try { paRules = JSON.parse(localStorage.getItem("pa_rules") || "[]"); } catch (e) {}
function paSave() { try { localStorage.setItem("pa_rules", JSON.stringify(paRules)); } catch (e) {} renderPa(); }
const PA_MODE_FA = { p: "از تونل", d: "مستقیم", b: "بلاک" };
const PA_MODE_EN = { p: "via tunnel", d: "direct", b: "block" };
function paRuleKey(val, kind) { return val.toLowerCase() + ":" + kind; }
function paFind(val, kind) {
  const k = paRuleKey(val, kind);
  return paRules.find(r => r.startsWith(k));
}
function paSetRule(display, val, kind, mode) {
  // قبلی همین برنامه را بردار (هر kind) و جدید بگذار
  paRules = paRules.filter(r => !(r.toLowerCase().startsWith(paRuleKey(val, kind)) ||
                                  r.toLowerCase().startsWith(paRuleKey(val, kind === "p" ? "n" : "p"))));
  if (mode && mode !== "s") paRules.push(`${val}:${mode}:${kind}`);
  paSave();
}
function paDisplayOf(rule) {
  const [v, m, k] = rule.split(":");
  let label = v;
  if (k === "p") { const parts = v.split("\\"); label = parts[parts.length - 1]; }
  return label;
}
function renderPa() {
  const el = $("#paList"); if (!el) return;
  el.innerHTML = "";
  if (!paRules.length) { el.innerHTML = `<div class="mini" style="opacity:.6">${esc(LANG==='fa'?"قاعده‌ای نیست — از «برنامه‌های در حال اجرا» شروع کن":"No rules yet — scan running apps")}</div>`; return; }
  paRules.forEach((r, i) => {
    const [, mode] = r.split(":");
    const label = paDisplayOf(r);
    const d = document.createElement("div");
    d.className = "mini";
    const badge = mode === "b" ? "bad" : mode === "d" ? "mid" : "good";
    d.innerHTML = `<span class="ping ${badge}">${(LANG==='fa'?PA_MODE_FA:PA_MODE_EN)[mode] || mode}</span>
                   <span style="flex:1;text-align:right;font-size:12px;direction:ltr;overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="${esc(label)}">${esc(label)}</span>
                   <button class="x">×</button>`;
    d.querySelector(".x").onclick = () => { paRules.splice(i, 1); paSave(); };
    el.appendChild(d);
  });
}
// ── اسکنِ برنامه‌های در حال اجرا (لیستِ زنده با جستجو) ──
let PA_APPS = [];
async function paScan() {
  const btn = $("#paScan"), list = $("#paScanList");
  btn.disabled = true;
  status(LANG==='fa'?"در حال خواندنِ برنامه‌های باز…":"Reading running apps…");
  try { PA_APPS = await call("list_running_apps"); } catch (e) { PA_APPS = []; }
  btn.disabled = false;
  renderPaScan();
  list.style.display = "";
  status(PA_APPS.length
    ? (LANG==='fa'?`${PA_APPS.length} برنامهٔ باز — روی هرکدام کلیک کن تا حالتش را بگذاری`:`${PA_APPS.length} running apps — click one to set its mode`)
    : (LANG==='fa'?"برنامه‌ای پیدا نشد":"Nothing found"));
}
function renderPaScan() {
  const el = $("#paScanList"); if (!el) return;
  const q = ($("#paSearch")?.value || "").toLowerCase();
  el.innerHTML = "";
  // نوارِ جستجو بالای لیست
  if (!$("#paSearch")) {
    const s = document.createElement("input");
    s.id = "paSearch"; s.className = "search"; s.dir = "ltr";
    s.placeholder = LANG==='fa' ? "جستجو…" : "search…";
    s.style.cssText = "width:100%;margin-bottom:6px";
    s.oninput = renderPaScan;
    el.appendChild(s);
  }
  const apps = PA_APPS.filter(a => !q || a.name.includes(q) || a.title.toLowerCase().includes(q));
  for (const a of apps.slice(0, 40)) {
    const existing = paFind(a.path, "p") || paFind(a.exe, "n");
    const curMode = existing ? existing.split(":")[1] : "s";
    const badge = curMode === "s"
      ? `<span class="ping" style="opacity:.5">${LANG==='fa'?"سیستم":"system"}</span>`
      : `<span class="ping ${curMode==='b'?'bad':curMode==='d'?'mid':'good'}">${(LANG==='fa'?PA_MODE_FA:PA_MODE_EN)[curMode]}</span>`;
    const d = document.createElement("div");
    d.className = "mini";
    d.innerHTML = `${badge}
      <span style="flex:1;text-align:right;overflow:hidden">
        <span style="display:block;font-size:12.5px;font-weight:700;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(a.name)}${a.is_store ? ' <span style="opacity:.55;font-size:10.5px">· Store</span>' : ""}</span>
        <span style="display:block;font-size:10.5px;opacity:.55;direction:ltr;text-align:left;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(a.title)}</span>
      </span>
      <button class="x">×</button>`;
    // چرخهٔ حالت با کلیک: سیستم ← تونل ← مستقیم ← بلاک
    d.onclick = (ev) => {
      if (ev.target.classList.contains("x")) return;
      const order = ["s","p","d","b"];
      const next = order[(order.indexOf(curMode) + 1) % order.length];
      paSetRule(a.name, a.path, "p", next);
      renderPaScan();
    };
    // × = حذف قاعده (برگشت به سیستم)
    d.querySelector(".x").onclick = (ev) => { ev.stopPropagation(); paSetRule(a.name, a.path, "p", "s"); renderPaScan(); };
    el.appendChild(d);
  }
  if (!apps.length) el.insertAdjacentHTML("beforeend", `<div class="mini" style="opacity:.6">—</div>`);
}
const paScanBtn = $("#paScan");
if (paScanBtn) paScanBtn.onclick = paScan;
// افزودنِ دستی (مسیر یا نام)
const paAddBtn = $("#paAdd");
if (paAddBtn) paAddBtn.onclick = () => {
  const raw = ($("#paName").value || "").trim();
  if (!raw) return;
  const mode = $("#paMode").value;
  if (raw.includes("\\")) {           // مسیرِ کامل → regex مسیر (UWP هم می‌گیرد)
    paSetRule(raw.split("\\").pop(), raw, "p", mode);
  } else {                            // فقط اسم → process_name
    const name = raw.toLowerCase().endsWith(".exe") ? raw.toLowerCase() : raw.toLowerCase() + ".exe";
    if (!/^[a-z0-9._-]+\.exe$/.test(name)) {
      status(LANG==='fa' ? "نام نامعتبر — مثال: chrome.exe یا مسیرِ کامل" : "Invalid — e.g. chrome.exe or a full path");
      return;
    }
    paSetRule(raw, name, "n", mode);
  }
  $("#paName").value = "";
};
renderPa();

// ---------- آپدیتِ خودکار از GitHub Releases ----------
const APP_VERSION = "1.0.0";
let pendingVersion = null;
async function checkAppUpdate() {
  const note = $("#updNote"), btn = $("#btnUpdate");
  if (!note) return;
  try {
    const v = await call("check_update");
    if (v) {
      pendingVersion = v;
      note.textContent = t("upd_available").replace("{v}", v);
      btn.style.display = "";
    } else {
      note.textContent = t("upd_latest").replace("{v}", APP_VERSION);
      btn.style.display = "none";
    }
  } catch (e) {
    note.textContent = t("upd_check_fail");
  }
}
const updBtn = $("#btnUpdate");
if (updBtn) updBtn.onclick = async () => {
  updBtn.disabled = true; status(t("upd_downloading"));
  try {
    await call("install_update");   // دانلود + اجرا + خروجِ اپ
  } catch (e) { status(t("failed") + " " + e); }
  updBtn.disabled = false;
};
// چک وقتی مودال تنظیمات باز می‌شود
const setBtn2 = $("#setBtn");
if (setBtn2) setBtn2.addEventListener("click", () => { setTimeout(checkAppUpdate, 400); });

// ---------- «کانفیگِ روز» — بعد از اولین تستِ موفق ظاهر می‌شود ----------
function renderDailyBtn() {
  const b = $("#btnDaily");
  if (!b) return;
  let has = false;
  try { has = !!JSON.parse(localStorage.getItem(DAILY_KEY) || "null"); } catch (e) {}
  b.style.display = has ? "" : "none";
  b.title = LANG==='fa' ? "کانفیگِ روز — ۵ سرورِ برتر برای فرستادن به دوستان"
                        : "Daily top-5 — send to friends";
}
$("#btnDaily").onclick = async () => {
  const txt = dailyText();
  if (!txt) { status(LANG==='fa'?"هنوز تستی انجام نشده":"Run a test first"); return; }
  await copyText(txt);
  status(LANG==='fa' ? "📋 کانفیگِ روز کپی شد — برای دوستات بفرست" : "Daily top-5 copied — share it");
};
// راست‌کلیک / نگه‌داشتن → QR برای اسکن با موبایل
const dailyBtn = $("#btnDaily");
if (dailyBtn) {
  dailyBtn.oncontextmenu = (e) => { e.preventDefault(); showDailyQr(); };
  dailyBtn.title += LANG==='fa' ? " (راست‌کلیک = QR)" : " (right-click = QR)";
}
function showDailyQr() {
  const d = (() => { try { return JSON.parse(localStorage.getItem(DAILY_KEY) || "null"); } catch(e){ return null; } })();
  if (!d || !d.list.length) { status(LANG==='fa'?"هنوز تستی انجام نشده":"Run a test first"); return; }
  // اولین سرور برتر در QR — موبایل با اسکن مستقیم import میکند
  const link = d.list[0].link;
  try {
    const qr = window.qrcode(0, "M");
    qr.addData(link); qr.make();
    const c = $("#qrCanvas"), ctx = c.getContext("2d");
    const count = qr.getModuleCount(), cell = Math.floor(260 / count), size = cell * count;
    c.width = size + 20; c.height = size + 20;
    ctx.fillStyle = "#fff"; ctx.fillRect(0, 0, c.width, c.height);
    ctx.fillStyle = "#000";
    for (let r = 0; r < count; r++) for (let col = 0; col < count; col++)
      if (qr.isDark(r, col)) ctx.fillRect(10 + col*cell, 10 + r*cell, cell, cell);
    $("#qrModal").classList.add("open");
  } catch (e) { status("QR: " + e); }
}
const qrClose = $("#qrClose");
if (qrClose) qrClose.onclick = () => $("#qrModal").classList.remove("open");
const qrModal = $("#qrModal");
if (qrModal) qrModal.onclick = (e) => { if (e.target.id === "qrModal") qrClose.click(); };

// ---------- ایمپورت از کلپ‌بورد — وقتی اپ فوکوس میگیرد لینک کانفیگ چک می‌شود ----------
let lastClipCheck = 0;
window.addEventListener("focus", async () => {
  if (Date.now() - lastClipCheck < 3000) return;   // جلوی اسپم
  lastClipCheck = Date.now();
  try {
    const txt = await navigator.clipboard.readText();
    if (!txt) return;
    const links = txt.split("\n").map(x => x.trim())
      .filter(x => /^(vless|vmess|trojan|ss|hysteria2?|hy2|tuic|anytls):\/\//i.test(x));
    if (!links.length) return;
    const fresh = links.filter(l => !manual.includes(l));
    if (!fresh.length) return;
    status(LANG==='fa' ? `${fresh.length} کانفیگ در کلپ‌بورد — «افزودن کانفیگ» را بزن تا اضافه شوند`
                       : `${fresh.length} config(s) in clipboard — press "Add" to import`);
    // متن آماده در باکس — کاربر فقط دکمه را میزند
    const box = $("#mcBox"); if (box && !box.value.trim()) box.value = fresh.join("\n");
  } catch (e) { /* اجازهٔ کلپ‌بورد نیست — مهم نیست */ }
});

// Cloudflare WARP — مجانی، نامحدود، بدونِ اکانت. اولین بار ثبت‌نام می‌کند.
$("#btnWarp").onclick = async () => {
  if (S.busy) return;
  S.busy = true; updateConnect(); status(t("warp_starting"));
  const finish = (ok) => {
    if (ok) {
      S.connected = true; S.connectedAt = Date.now(); S.sel = "__warp__";
      S.selCc = ""; S.exitInfo = null; S.livePing = null;
      call("exit_info").then(i => { if (S.connected && i && i.country) { S.exitInfo = i; renderHero(); } }).catch(()=>{});
    } else { S.connected = false; S.connectedAt = null; }
    S.busy = false; updateConnect(); renderHero();
  };
  try {
    // ۱) اول مستقیم (سریع‌ترین حالت)
    await call("connect_warp", { carrier: null });
    status(t("warp_ok")); finish(true); return;
  } catch (e) { /* endpointِ پیش‌فرض بسته است */ }
  try {
    // ۲) اسکنِ endpointهای کلادفلر — همان ترفندی که WARP را در ایران باز می‌کند
    status(t("warp_scanning"));
    const r = await call("warp_scan", { count: 24 });
    const [ep, ms] = String(r).split("|");
    status(t("warp_found").replace("{ep}", ep).replace("{ms}", ms));
    await call("connect_warp", { carrier: null });
    status(t("warp_ok")); finish(true); return;
  } catch (e) { /* هیچ endpointی باز نبود */ }
  try {
    // ۳) WARP از داخلِ یکی از سرورهای خودت (سریع‌تر از gool)
    const carrier = pickCarrier();
    if (!carrier) throw new Error("no carrier");
    status(t("warp_via_carrier"));
    await call("connect_warp", { carrier });
    status(t("warp_ok")); finish(true); return;
  } catch (e) { /* حامل هم نشد */ }
  try {
    // ۴) آخرین راه: gool — کندترین است چون دو لایه تونل دارد
    status(t("gool_trying"));
    await call("connect_gool");
    status(t("gool_ok")); finish(true);
  } catch (e) {
    status(t("failed") + " " + e); finish(false);
  }
};
$("#btnBest").onclick = () => {
  const best = S.rows.filter(r=>r.link!=="__relay__"&&r.ping>=0).sort((a,b)=>a.ping-b.ping)[0];
  if(best){ selectServer(best.link); toggleConnect(); } else status(t("run_test_first"));
};
$("#btnOnlyWorking").onclick = (e) => {
  S.onlyWorking = !S.onlyWorking;
  e.target.classList.toggle("on", S.onlyWorking); renderList();
};
$("#search").oninput = (e) => { S.filter = e.target.value.trim().toLowerCase(); renderList(); };

$("#swFragment").onchange = async e => {
  status(e.target.checked ? t("frag_on") : t("frag_off"));
  // Fragment فقط در حالتِ پراکسی (هسته‌ی xray) معنا دارد
  if (connMode() !== "proxy") { status(t("frag_proxy_only")); return; }
  if (S.connected && S.sel) await reconnectCurrent();   // با رعایتِ حالت
};

// ---------- کانفیگِ من ----------
let manual = [], subs = [];
try { manual = JSON.parse(localStorage.getItem("manual") || "[]"); } catch(e){}
try { subs = JSON.parse(localStorage.getItem("subs") || "[]"); } catch(e){}
function saveMine(){ try { localStorage.setItem("manual", JSON.stringify(manual)); localStorage.setItem("subs", JSON.stringify(subs)); } catch(e){} }
$("#mcAdd").onclick = () => {
  const raw = $("#mcBox").value;
  const links = [];
  // کانفیگِ WireGuard (Windscribe / Proton / هر سرویسی که با اکانتِ خودت می‌دهد)
  if (/\[Interface\]/i.test(raw) && /\[Peer\]/i.test(raw)) {
    try {
      links.push("wg://" + btoa(unescape(encodeURIComponent(raw.trim()))));
    } catch (e) { status(t("wg_bad")); }
  } else {
    raw.split("\n").map(x => x.trim()).filter(x => x.includes("://")).forEach(x => links.push(x));
  }
  let added = 0;
  links.forEach(x => { if (!manual.includes(x)) { manual.push(x); added++; } });
  $("#mcBox").value = ""; saveMine(); renderMini();
  status(added ? (added + " " + t("mc_added_n")) : t("mc_none"));
};
$("#subAdd").onclick = () => {
  const u = $("#subEntry").value.trim(); if(u&&!subs.includes(u)){subs.push(u);$("#subEntry").value="";saveMine();renderMini();status(t("sub_added"));}
};
function renderMini() {
  const m = $("#mcList"); m.innerHTML = manual.length ? "" : `<div class="empty">${esc(t("mc_empty"))}</div>`;
  manual.forEach((l,i)=>{ const d=document.createElement("div");d.className="mini";
    d.innerHTML=`<button class="x"></button><span>${esc(l.slice(0,54))}</span>`;
    d.querySelector(".x").onclick=()=>{manual.splice(i,1);saveMine();renderMini();loadServers();}; m.appendChild(d); });
  const s = $("#subList"); s.innerHTML = subs.length ? "" : `<div class="empty">${esc(t("sub_empty"))}</div>`;
  subs.forEach((u,i)=>{ const d=document.createElement("div");d.className="mini";
    d.innerHTML=`<button class="x"></button><span>${esc(u)}</span>`;
    d.querySelector(".x").onclick=()=>{subs.splice(i,1);saveMine();renderMini();}; s.appendChild(d); });
}

// ---------- بلک‌اوت (ویزاردِ راه‌اندازی + رله‌ی جدا) ----------
let WORKER_TPL = "", GAS_TPL = "", relayCfg = null, relayOn = false;

async function loadRelayTemplates() {
  try { WORKER_TPL = await (await fetch("relay/worker.js")).text(); } catch(e){ WORKER_TPL = "// worker.js پیدا نشد"; }
  try { GAS_TPL = await (await fetch("relay/Code.gs")).text(); } catch(e){ GAS_TPL = "// Code.gs پیدا نشد"; }
  regenRelay();
}
// آدرسِ Worker را به شکلِ استاندارد درمی‌آورد. کاربر ممکن است کاملِ آدرس را با
// https:// و اسلشِ آخر پیست کند (کارِ کاملاً طبیعی) — قبلاً قالب دوباره https://
// می‌گذاشت و «https://https://…» می‌شد و رله با «خطای DNS» می‌افتاد.
function normWorker(v) {
  return String(v || "").trim()
    .replace(/^https?:\/\//i, "")   // پروتکلِ اضافه
    .replace(/\/+$/, "")             // اسلشِ انتهایی
    .replace(/\s+/g, "");
}

function regenRelay() {
  const key = $("#wzKey").value.trim();
  const wk  = normWorker($("#wzWorker").value);
  $("#workerCode").textContent = WORKER_TPL.replace(
    'const WORKER_URL = "myworker.workers.dev";',
    `const WORKER_URL = "${wk || "myworker.workers.dev"}";`);
  if (key && wk) {
    $("#gasCode").textContent = GAS_TPL
      .replace('const AUTH_KEY = "STRONG_SECRET_KEY";', `const AUTH_KEY = "${key}";`)
      .replace('const WORKER_URL_RAW = "https://example.workers.dev";', `const WORKER_URL_RAW = "${wk}";`);
  } else {
    $("#gasCode").textContent = t("gas_fill_first");
  }
}
["wzKey","wzWorker"].forEach(id => $("#"+id).addEventListener("input", regenRelay));

async function copyText(t) {
  try { await navigator.clipboard.writeText(t); status(t("copied")); }
  catch(e){ status(t("copy_fail")); }
}
$("#copyWorker").onclick = () => copyText($("#workerCode").textContent);
$("#copyGas").onclick   = () => copyText($("#gasCode").textContent);

function relayReady(cfg) {
  relayCfg = cfg;
  $("#relayConnect").disabled = false;
  $("#relayState").textContent = t("relay_ready"); $("#relayHint").textContent = t("relay_press");
  $("#relayHint").textContent = t("relay_press");
}
$("#wzSave").onclick = () => {
  const key = $("#wzKey").value.trim(), wk = normWorker($("#wzWorker").value), sid = $("#wzScript").value.trim();
  if (!key || !sid) { status(t("need_key_sid")); return; }
  const cfg = { auth_key:key, worker:wk, script_ids:[sid] };
  localStorage.setItem("relayCfg", JSON.stringify(cfg));
  relayReady(cfg);
  status(t("relay_saved"));
};
// وضعیتِ دکمه‌ی رله را با واقعیت هماهنگ می‌کند
function paintRelay() {
  const b = $("#relayConnect");
  b.classList.toggle("on", relayOn);
  b.textContent = relayOn ? t("relay_disconnect") : t("relay_connect");
  $("#relayState").textContent = relayOn ? t("relay_on") : (relayCfg ? t("relay_ready") : t("relay_state0"));
  $("#relayHint").textContent = relayOn ? t("relay_on_hint") : (relayCfg ? t("relay_press") : t("relay_hint0"));
}

$("#relayConnect").onclick = async () => {
  if (!relayCfg || S.busy) return;
  const b = $("#relayConnect");
  b.disabled = true; S.busy = true; updateConnect();
  try {
    if (relayOn) {
      await call("disconnect");
      relayOn = false; S.connected = false; S.connectedAt = null; S.exitInfo = null;
      status(t("relay_conn_off"));
    } else {
      // رله چند ثانیه طول می‌کشد (ساختِ گواهی + پیدا کردنِ IP گوگل)
      status(t("relay_starting"));
      await call("connect_relay", {
        authKey: relayCfg.auth_key,
        scriptIds: relayCfg.script_ids || [],
      });
      relayOn = true;
      S.connected = true; S.connectedAt = Date.now(); S.sel = "__relay__"; S.selCc = "";
      status(t("relay_conn_ok"));
      call("exit_info").then(info => {
        if (S.connected && info && info.country) { S.exitInfo = info; renderHero(); }
      }).catch(() => {});
    }
  } catch (e) {
    relayOn = false; S.connected = false; S.connectedAt = null;
    status(t("failed") + " " + e);
  }
  S.busy = false; b.disabled = false;
  paintRelay(); updateConnect(); renderHero();
};
try {
  const s = localStorage.getItem("relayCfg");
  if (s) { const c = JSON.parse(s);
    $("#wzKey").value = c.auth_key||""; $("#wzWorker").value = c.worker||"";
    $("#wzScript").value = (c.script_ids||[])[0]||""; relayReady(c); }
} catch(e){}
loadRelayTemplates();

// ---------- لاگِ پشت‌صحنه ----------
let logTimer = null;
async function refreshLog() {
  try {
    const lines = await call("get_log");
    const pre = $("#logPre");
    pre.textContent = (lines && lines.length) ? lines.join("\n") : t("log_empty");
    pre.scrollTop = pre.scrollHeight;
  } catch (e) {}
}
function openLog() {
  $("#logModal").classList.add("open");
  refreshLog();
  logTimer = setInterval(refreshLog, 1500);
}
$("#logClose").onclick = () => {
  $("#logModal").classList.remove("open");
  if (logTimer) { clearInterval(logTimer); logTimer = null; }
};
$("#logModal").onclick = (e) => { if (e.target.id === "logModal") $("#logClose").click(); };

// ---------- شروع ----------
function status(t){ $("#status").textContent = t; }
function logUi(m){ try { console.log("[shabgard] " + m); } catch(e){} }

// ---------- اسپارک‌لاینِ پینگ زنده — تاریخچهٔ جلسه در کاشی تأخیر ----------
const PING_HIST_MAX = 60;
let pingHist = [];
function pushPingHist(ms) {
  if (ms == null || ms < 0) return;
  pingHist.push(ms);
  if (pingHist.length > PING_HIST_MAX) pingHist.shift();
  drawPingSpark();
}
function drawPingSpark() {
  const c = $("#pingSpark"); if (!c || !pingHist.length) return;
  const ctx = c.getContext("2d");
  const w = c.width, h = c.height, n = pingHist.length;
  const max = Math.max(...pingHist, 100), min = Math.min(...pingHist);
  ctx.clearRect(0, 0, w, h);
  // رنگ بر اساس وضعیت: سبک خوب، کهربایی متوسط
  const avg = pingHist.reduce((a,b)=>a+b,0) / n;
  ctx.strokeStyle = avg < 180 ? "#25c26e" : avg < 300 ? "#e8a13a" : "#e05555";
  ctx.lineWidth = 1.5; ctx.beginPath();
  for (let i = 0; i < n; i++) {
    const x = (i / (PING_HIST_MAX - 1)) * w;
    const y = h - 3 - ((pingHist[i] - min) / Math.max(1, max - min)) * (h - 8);
    i ? ctx.lineTo(x, y) : ctx.moveTo(x, y);
  }
  ctx.stroke();
}
async function loadServers(opts) {
  const prevLinks = new Set((S.rows || []).map(r => r.link));
  status(t("fetching_servers"));
  try { S.rows = await call("list_servers", { subs: subs || [], manual: manual || [], game: connMode() === "game" }); }
  catch (e) { S.rows = []; status(t("fetch_fail") + " " + e); return; }
  // کشِ تست: زنده‌های قبلی اول با پینگِ کهنه، مرده‌های کهنه حذف
  await applyPingCache(S.rows);
  renderList();
  status(`${S.rows.length} ${t("servers_word")} — ${t("press_test")}`);
  // آیا ساب تازه، کانفیگِ جدیدی آورده؟ → اگر کاربر وصل نیست یک‌بار خودکار تست کن
  const fresh = S.rows.filter(r => r.link !== "__relay__" && !prevLinks.has(r.link)).length;
  if (opts && opts.autoTest && fresh > 0 && !S.connected && !S.testing && !S.busy) {
    status(LANG==='fa' ? `${fresh} کانفیگِ تازه رسید — تستِ خودکار…` : `${fresh} new configs — auto-testing…`);
    $("#btnTest").click();
  }
}
$("#routeSel").onchange = async () => {
  localStorage.setItem("route", $("#routeSel").value);
  const on = $("#routeSel").value === "bypass";
  status(t("route_lbl") + " " + (on ? t("route_bypass") : t("route_global")));
  await reconnectCurrent();   // با رعایتِ حالت (TUN/گیم از TUN بیرون نمی‌افتد)
};
// تعویضِ حالتِ اتصال (پراکسی / TUN / گیم) — اگر وصل بودیم، قطع کن تا کاربر با حالتِ جدید دوباره وصل شود
$("#modeSel").onchange = async () => {
  localStorage.setItem("mode", $("#modeSel").value);
  updateModeHint();
  const m = connMode();
  status(m === "tun" ? t("mode_tun") : m === "game" ? t("mode_game") : t("mode_proxy"));
  if (S.connected) {
    S.busy = true; updateConnect();
    try { await call("disconnect"); } catch (e) {}
    S.connected = false; S.connectedAt = null; S.exitInfo = null;
    S.busy = false; updateConnect(); renderHero();
  }
  // حالتِ گیم لیستِ مخصوصِ گیم (کم‌پینگ/کم‌جیتر) را می‌گیرد → لیست را تازه کن
  loadServers();
};
// ---------- تحلیل‌گرِ نت ----------
function anRow(ok, label, good, bad) {
  const mark = ok ? "" : "";
  return `<div class="an-row"><span class="an-k">${mark} ${esc(label)}</span>` +
         `<span class="an-v ${ok ? 'ok' : 'no'}">${esc(ok ? good : bad)}</span></div>`;
}
$("#btnAnalyze").onclick = async () => {
  $("#anModal").classList.add("open");
  $("#anBody").innerHTML = `<div class="an-wait">${esc(t("an_running"))}</div>`;
  try {
    const p = await call("analyze_net");
    const yes = t("an_yes"), no = t("an_no");
    let h = "";
    h += anRow(p.udp_dns, t("an_udp"), yes, no);
    h += anRow(p.quic, t("an_quic"), yes, no);
    h += anRow(!p.sni_block, t("an_sni"), t("an_clean"), t("an_dpi"));
    h += anRow(!p.dns_poison, t("an_dns"), t("an_clean"), t("an_poisoned"));
    h += anRow(p.cdn_ok, t("an_cdn"), yes, no);
    h += anRow((p.ports || []).length > 0, t("an_ports"), (p.ports || []).join(LANG === "fa" ? "، " : ", ") || "—", "—");
    const tips = String(p.advice || "").split(",").filter(Boolean).map(k => t("adv_" + k)).filter(Boolean);
    if (tips.length) {
      h += `<div class="an-advice"><b>${esc(t("an_advice"))}</b><br>• ${tips.map(esc).join("<br>• ")}</div>`;
    }
    $("#anBody").innerHTML = h;
    // نتیجه را نگه دار تا انتخابِ کانفیگ هوشمندتر شود (و بعداً به گیت‌هاب برود)
    try { localStorage.setItem("netprobe", JSON.stringify(p)); } catch (e) {}
  } catch (e) {
    $("#anBody").innerHTML = `<div class="an-wait">${esc(t("failed") + " " + e)}</div>`;
  }
};
$("#anClose").onclick = () => $("#anModal").classList.remove("open");

// اسمِ فایلِ اجراییِ بازی (اختیاری) — برای بالا بردنِ اولویتش در گیم‌مود
const gameExeInp = $("#gameExe");
if (gameExeInp) gameExeInp.oninput = () => localStorage.setItem("game_exe", gameExeInp.value.trim());
// نتایجِ زنده‌ی تست (event از Rust)
if (TAURI && window.__TAURI__.event) {
  window.__TAURI__.event.listen("test_result", (e) => {
    const [i, ping] = e.payload || [];
    const link = S.testLinks && S.testLinks[i];
    if (!link) return;
    const r = S.rows.find(x => x.link === link);
    if (r) { r.ping = ping; scheduleRender(); }
  });
}
applyLang();
loadServers(); renderMini(); updateConnect(); detectNet();
renderDailyBtn();

// تحلیلِ نت را یک‌بار در روز خودکار بگیر (چند کیلوبایت) تا تطبیقِ کانفیگ کار کند
// و «تست همه» به‌جای صدها کانفیگ، فقط مناسب‌های نتِ تو را تست کند.
(async () => {
  if (!TAURI) return;
  const last = Number(localStorage.getItem("netprobe_at") || 0);
  if (Date.now() - last < 24 * 3600 * 1000 && netProbe()) return;
  try {
    const p = await call("analyze_net");
    if (p) { localStorage.setItem("netprobe", JSON.stringify(p)); localStorage.setItem("netprobe_at", String(Date.now())); }
  } catch (e) {}
})();

// تستِ خودکار پس از آپدیت ساب: اگر از آخرین تست چند ساعت گذشته و هنوز وصل نیستیم،
// یک‌بار خودکار تست کن (کاربر هیچ کاری لازم نیست بکند). حداکثر یک‌بار در ۶ ساعت.
(async () => {
  if (!TAURI) return;
  const lastTest = Number(localStorage.getItem("last_test_at") || 0);
  if (Date.now() - lastTest < 6 * 3600 * 1000) return;
  // صبر کن لیست اول بیاید و تحلیلِ نت تمام شود
  setTimeout(() => {
    if (!S.connected && !S.testing && !S.busy && S.rows.length) $("#btnTest").click();
  }, 8000);
})();



// ---------- تنظیمات: ظاهر + زبان + لاگ (همه یک‌جا) ----------
function paintLangSeg() {
  document.querySelectorAll("#langSeg .seg-b").forEach(b =>
    b.classList.toggle("on", b.getAttribute("data-lang") === LANG));
}
$("#setBtn").onclick = async () => {
  renderThemeGrid(); paintLangSeg();
  try { const on = await call("autostart_get"); $("#swAutostart").checked = !!on; } catch (e) {}
  $("#setModal").classList.add("open");
};
$("#swAutostart").onchange = async (e) => {
  try { await call("autostart_set", { on: e.target.checked }); status(t(e.target.checked ? "start_on" : "start_off")); }
  catch (err) { e.target.checked = !e.target.checked; status(t("failed") + " " + err); }
};
$("#setClose").onclick = () => $("#setModal").classList.remove("open");
$("#setModal").onclick = (e) => { if (e.target.id === "setModal") $("#setClose").click(); };
document.querySelectorAll("#langSeg .seg-b").forEach(b => b.onclick = () => {
  LANG = b.getAttribute("data-lang");
  localStorage.setItem("lang", LANG);
  applyLang(); paintLangSeg(); renderThemeGrid();
  status(LANG === "fa" ? "زبان: فارسی" : "Language: English");
});
$("#openLog").onclick = () => { $("#setModal").classList.remove("open"); openLog(); };
paintLangSeg();


// تستِ ظرفیتِ پهنای باند (یک دانلودِ کوتاهِ واقعی از داخلِ تونل)
const bwBtn = $("#bwTest");
if (bwBtn) bwBtn.onclick = async () => {
  if (!S.connected) { status(t("bw_need_conn")); return; }
  bwBtn.disabled = true; status(t("bw_testing"));
  try {
    const [dn, up] = await call("bandwidth_test");
    if (dn > 0 || up > 0) {
      localStorage.setItem("bw_cap_dn", String(dn));
      localStorage.setItem("bw_cap_up", String(up));
      status(t("bw_done").replace("{d}", dn).replace("{u}", up));
    } else status(t("failed"));
  } catch (e) { status(t("failed") + " " + e); }
  bwBtn.disabled = false; paintTiles();
};


// ---------- نوارِ عنوانِ خودمان (چرومِ ویندوز خاموش است) ----------
(function () {
  if (!TAURI || !window.__TAURI__.window) return;
  const W = window.__TAURI__.window.getCurrentWindow();
  const set = (id, fn) => { const b = $(id); if (b) b.onclick = (e) => { e.stopPropagation(); fn(); }; };
  set("#wMin", () => W.minimize());
  set("#wMax", async () => { await W.toggleMaximize(); paintMax(); });
  set("#wClose", () => W.hide());          // ضربدر = رفتن به سینی (VPN وصل می‌ماند)
  async function paintMax() {
    try {
      const m = await W.isMaximized();
      const u = $("#wMax") && $("#wMax").querySelector("use");
      if (u) u.setAttribute("href", m ? "#ic-restore" : "#ic-max");
    } catch (e) {}
  }
  paintMax();
  // دابل‌کلیک روی نوار = بزرگ/کوچک
  const tb = document.querySelector(".titlebar");
  if (tb) tb.ondblclick = async (e) => {
    if (e.target.closest(".wnd") || e.target.closest(".icon-btn")) return;
    await W.toggleMaximize(); paintMax();
  };
})();


// ══════════════════════════════════════════════════════════════════════════
//  دراپ‌داونِ سفارشی — منوی بازشده‌ی <select> در WebView2 استایل نمی‌گیرد
//  (سفید و بیگانه با تم می‌شد). اینجا خودِ select برای منطق می‌ماند ولی
//  ظاهر و منو را خودمان می‌سازیم تا با هر ۷ تم هماهنگ باشد.
// ══════════════════════════════════════════════════════════════════════════
function enhanceSelect(sel) {
  if (!sel || sel.dataset.enhanced) return;
  sel.dataset.enhanced = "1";
  const wrap = sel.closest(".field-w") || sel.parentElement;
  wrap.classList.add("cs-wrap");
  const btn = document.createElement("button");
  btn.type = "button"; btn.className = "cs-btn";
  const menu = document.createElement("div");
  menu.className = "cs-menu";
  wrap.appendChild(btn); wrap.appendChild(menu);

  const paint = () => {
    const o = sel.options[sel.selectedIndex];
    btn.textContent = o ? o.text : "";
    menu.innerHTML = [...sel.options].map((op, i) =>
      `<div class="cs-item${i === sel.selectedIndex ? " on" : ""}" data-i="${i}">${esc(op.text)}</div>`).join("");
    menu.querySelectorAll(".cs-item").forEach(it => it.onclick = (e) => {
      e.stopPropagation();
      sel.selectedIndex = Number(it.getAttribute("data-i"));
      close(); paint();
      sel.dispatchEvent(new Event("change", { bubbles: true }));
    });
  };
  const close = () => { menu.classList.remove("open"); btn.classList.remove("open"); };
  const open = () => {
    document.querySelectorAll(".cs-menu.open").forEach(m => m.classList.remove("open"));
    document.querySelectorAll(".cs-btn.open").forEach(b => b.classList.remove("open"));
    menu.classList.add("open"); btn.classList.add("open");
  };
  btn.onclick = (e) => { e.stopPropagation(); menu.classList.contains("open") ? close() : open(); };
  document.addEventListener("click", close);
  // وقتی زبان/گزینه‌ها عوض شد، دوباره بکش
  sel._repaint = paint;
  paint();
}
function enhanceAllSelects() {
  ["#modeSel", "#routeSel"].forEach(id => {
    const el = $(id); if (!el) return;
    enhanceSelect(el);
    if (el._repaint) el._repaint();
  });
}
enhanceAllSelects();


// بازکردنِ اپ‌های Store/UWP روی پراکسیِ محلی (یک‌بار، با ادمین)
const uwpBtn = $("#btnUwp");
if (uwpBtn) uwpBtn.onclick = async () => {
  uwpBtn.disabled = true; status(t("uwp_working"));
  try { await call("uwp_exempt"); status(t("uwp_done")); }
  catch (e) { status(t("failed") + " " + e); }
  uwpBtn.disabled = false;
};

// حریم خصوصی — پاک‌کردنِ همهٔ ردهای محلی با یک کلیک
// (فایل‌های دیسک از Rust + داده‌های حساسِ localStorage از این‌جا)
// کلیدهایی که «تنظیمات»اند نه رَد — بعد از پاک‌سازی برگردانده می‌شوند
const KEEP_LS = ["theme", "lang", "mode", "route"];
const wipeBtn = $("#btnWipe");
if (wipeBtn) wipeBtn.onclick = async () => {
  wipeBtn.disabled = true;
  try {
    const [n, bytes] = await call("wipe_privacy");
    // همهٔ کلیدها پاک شود به‌جز تنظیماتِ ظاهری (ping_cache، ok_<isp>، daily5، netprobe و… همه رَدند)
    const keep = {};
    KEEP_LS.forEach(k => { const v = localStorage.getItem(k); if (v !== null) keep[k] = v; });
    localStorage.clear();
    Object.entries(keep).forEach(([k, v]) => localStorage.setItem(k, v));
    manual = []; subs = []; paRules = []; renderPa(); renderMini();
    const kb = Math.max(1, Math.round(bytes / 1024));
    status(t("priv_done").replace("{n}", n).replace("{kb}", kb));
  } catch (e) { status(t("failed") + " " + e); }
  wipeBtn.disabled = false;
};
