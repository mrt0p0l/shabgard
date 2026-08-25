<div dir="rtl">

# 🌙 شبگرد — Shabgard

کلاینتِ ضدِ سانسورِ ویندوزی — سریع، سبک، بدونِ حسابِ کاربری.

شبگرد یک اپِ متن‌باز برای دورزدنِ فیلترینگ است: کانفیگ‌های آماده از منابعِ عمومی می‌گیرد، تست می‌کند، و با یک کلیک وصل می‌شود. بدون ثبت‌نام، بدون آنالیتیکس، بدون ارسالِ هیچ داده‌ای به هیچ سروری.

![Screenshot](screenshots/main.png)

## ✨ امکانات

- **پروتکل‌ها:** VLESS / VMess / Trojan / Shadowsocks / Hysteria2 / TUIC / WireGuard / AnyTLS / XHTTP + REALITY، ECH، Fragment
- **حالت‌های اتصال:** پراکسیِ سیستم · TUN (همه‌ی برنامه‌ها) · گیم (کم‌تأخیر + بوستِ سیستم)
- **Cloudflare WARP داخلی:** ثبت‌نام خودکار + اسکنر endpoint + gool (WARP دولایه) — همه مجانی
- **پرفراپ:** کنترل هر برنامه — از تونل برود، مستقیم، یا بلاک (برنامه‌های Store هم پشتیبانی می‌شوند)
- **موتور تست سریع:** همه‌ی سرورها در یک پروسه مشترک — ده‌ها برابر سریع‌تر از روش سنتی
- **کش هوشمند:** کانفیگ‌های مرده فراموش می‌شوند، زنده‌ها اول لیست می‌آیند
- **بلک‌اوت:** رله شخصی از مسیر Google + Cloudflare برای وقتی اینترنت تقریباً قطع است
- **حریم خصوصی:** دکمه «پاک‌کردن داده‌های من» + لاگ با سقف ۵ مگ + هیچ ردی بیرون نمی‌ماند
- **آپدیت خودکار** + دانلود هسته‌ها موقع نصب

## 📥 نصب

1. آخرین `Shabgard_x.x.x_x64-setup.exe` را از [Releases](../../releases) دانلود کن
2. نصب کن (هسته‌های xray/sing-box موقع نصب خودکار دانلود می‌شوند)
3. تمام — دکمه را بزن

## 🔒 حریم خصوصی

- **هیچ اطلاعاتی برای هیچ سروری ارسال نمی‌شود** — بدون حساب، بدون آنالیتیکس
- تنها ترافیک شبکه: گرفتن لیست کانفیگ از منابع عمومی + تست سرعت/پینگ
- کل سورس باز است؛ خودت می‌توانی چک کنی

## 🛠 ساخت از سورس

```bash
npm install
npm run tauri build
```

نیازمندی: Node.js 18+، Rust (MSVC)، WebView2

## ⚖️ مجوز

MIT — آزاد برای استفاده، تغییر، و انتشار.

</div>

---

<div dir="ltr">

# 🌙 Shabgard

Windows anti-censorship client — fast, lightweight, no account needed.

Shabgard fetches configs from public sources, tests them, and connects with one click. No signup, no analytics, nothing sent to any server.

## Features

- **Protocols:** VLESS / VMess / Trojan / Shadowsocks / Hysteria2 / TUIC / WireGuard / AnyTLS / XHTTP + REALITY, ECH, Fragment
- **Connection modes:** System proxy · TUN (all apps) · Game mode (low latency + system boost)
- **Built-in Cloudflare WARP:** auto-registration + endpoint scanner + gool (WARP-in-WARP)
- **Proxifier-style routing:** per-app tunnel/direct/block (Store apps supported)
- **Fast test engine:** all servers in one shared process — orders of magnitude faster
- **Smart cache:** dead configs are forgotten, alive ones float to the top
- **Blackout relay:** personal relay via Google + Cloudflare when the internet is nearly shut down
- **Privacy:** "Wipe my data" button, 5MB log cap, zero traces left behind
- **Auto-update** + cores downloaded at install time

## Install

Grab the latest setup from [Releases](../../releases), install, done. Cores (xray/sing-box) download during setup automatically.

## Privacy

Nothing is sent to any server — no account, no telemetry. The only network traffic is fetching public config lists and speed probes. Source is open; verify yourself.

## Build

```bash
npm install
npm run tauri build
```

Requires: Node.js 18+, Rust (MSVC), WebView2

## License

MIT

</div>
