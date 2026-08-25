// ساخت پکیج پورتابل شبگرد — باز کن، اجرا کن، بدون نصب
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname);
const exe = path.join(root, "src-tauri", "target", "release", "shabgard.exe");
const outDir = path.join(root, "Shabgard-Portable");
const zipPath = path.join(path.dirname(root), "Desktop", "شبگرد", "شبگرد-پورتابل.zip");

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(path.join(outDir, "data"), { recursive: true });

// exe
fs.copyFileSync(exe, path.join(outDir, "shabgard.exe"));
// نشانگرِ پورتابل
fs.writeFileSync(path.join(outDir, "portable.txt"), "Shabgard portable mode — data lives in .\\data\r\n");
// راهنمای کوتاه
fs.writeFileSync(path.join(outDir, "!راهنما.txt"),
`شبگرد — نسخهٔ پورتابل
======================

▶ اجرا: shabgard.exe

• هیچی نصب نمی‌شود، در رجیستری نمی‌رود.
• همهٔ داده‌ها (لاگ، تنظیمات، WARP) داخل پوشهٔ «data» همین پوشه است.
• حذف کامل = پاک کردن همین پوشه. تمام.

⚠️ برای حالت TUN/گیم یک‌بار Run as Administrator لازم است.
`, "utf8");

// زیپ
fs.mkdirSync(path.dirname(zipPath), { recursive: true });
if (fs.existsSync(zipPath)) fs.unlinkSync(zipPath);
execSync(`powershell -NoProfile -Command "Compress-Archive -Force -Path '${outDir}\\*' -DestinationPath '${zipPath}'"`);
console.log("PORTABLE ZIP:", zipPath, fs.statSync(zipPath).size, "bytes");
