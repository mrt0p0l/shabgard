// بنرهای تبلیغاتی شبگرد — یکی برای هر تم، با رنگ‌های دقیقِ همان تم
const { chromium } = require("playwright");
const fs = require("fs");
const path = require("path");

// رنگ‌های دقیق از styles.css هر تم
const THEMES = [
  { id: "linear",   fa: "لینیر",       bg: "#050506", bg2: "#0a0a0f", card: "#12131a", edge: "#2a2c38", fg: "#EDEDEF", mut: "#8A8F98", accent: "#5E6AD2", grn: "#25c26e" },
  { id: "brutal",   fa: "بروتالیسم",   bg: "#f3f0e7", bg2: "#eae6da", card: "#ffffff", edge: "#0d0d0d", fg: "#0d0d0d", mut: "#5b5b5b", accent: "#ffe14d", grn: "#4fe07a" },
  { id: "bento",    fa: "بنتو",        bg: "#0e1014", bg2: "#151920", card: "#1b2029", edge: "#2b3140", fg: "#f0f2f6", mut: "#8d97a8", accent: "#4f8cff", grn: "#2ecc71" },
  { id: "oled",     fa: "OLED سوئیسی", bg: "#000000", bg2: "#000000", card: "#000000", edge: "#1c1c1c", fg: "#ffffff", mut: "#7a7a7a", accent: "#ffffff", grn: "#00e07a" },
  { id: "clay",     fa: "کِلِی نرم",   bg: "#1c1b2e", bg2: "#252340", card: "#2c2a4a", edge: "#3a3758", fg: "#efeaff", mut: "#a49dc4", accent: "#8b7bff", grn: "#71e8ad" },
  { id: "material", fa: "متریال",      bg: "#14131a", bg2: "#211f28", card: "#2b2833", edge: "#3a3745", fg: "#e7e0eb", mut: "#a49fb0", accent: "#d0bcff", grn: "#9ff2c0" },
  { id: "tactile",  fa: "تکتایل رامس", bg: "#d8d5cd", bg2: "#e2dfd6", card: "#e7e4dc", edge: "#b5b1a6", fg: "#1d1d1b", mut: "#6d6a63", accent: "#e2571f", grn: "#1d6b3f" },
];

function bannerSvg(t) {
  const dark = !["brutal", "tactile"].includes(t.id);
  const radius = { linear: 16, brutal: 0, bento: 18, oled: 2, clay: 26, material: 28, tactile: 8 }[t.id];
  const hard = t.id === "brutal" ? `stroke="#0d0d0d" stroke-width="3"` : "";
  return `<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="640" viewBox="0 0 1280 640">
<defs>
  <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="${t.bg2}"/><stop offset="1" stop-color="${t.bg}"/>
  </linearGradient>
  <radialGradient id="glow" cx=".5" cy=".5" r=".5">
    <stop offset="0" stop-color="${t.grn}" stop-opacity=".22"/><stop offset="1" stop-color="${t.grn}" stop-opacity="0"/>
  </radialGradient>
</defs>
<rect width="1280" height="640" fill="url(#bg)" rx="${radius}"/>

<!-- power button (hero element of the app) -->
<g transform="translate(1046,320)">
  <circle r="190" fill="url(#glow)"/>
  <circle r="118" fill="${t.card}" ${hard} stroke="${t.edge}" stroke-width="2"/>
  <circle r="118" fill="none" stroke="${t.grn}" stroke-width="4" opacity=".85"/>
  <g stroke="${t.grn}" stroke-width="11" stroke-linecap="round" fill="none">
    <line x1="0" y1="-58" x2="0" y2="-6"/>
    <path d="M -46,-34 A 56,56 0 1,0 46,-34" transform="translate(0,12)"/>
  </g>
</g>

<!-- brand -->
<text x="96" y="212" font-family="Vazirmatn,'Segoe UI',Tahoma,sans-serif" direction="rtl"
      font-size="118" font-weight="700" fill="${t.fg}">شبگرد</text>
<text x="100" y="264" font-family="'Segoe UI',Arial,sans-serif" font-size="27"
      letter-spacing="15" fill="${t.mut}">SHABGARD</text>

<rect x="98" y="296" width="86" height="6" rx="3" fill="${t.grn}"/>

<text x="98" y="368" font-family="Vazirmatn,'Segoe UI',Tahoma,sans-serif" direction="rtl"
      font-size="33" fill="${t.fg}">ضدِ سانسور · سریع · بدونِ حساب</text>

<!-- feature chips -->
<g font-family="Vazirmatn,'Segoe UI',Tahoma,sans-serif" font-size="20" fill="${t.fg}">
  <rect x="98"  y="420" width="168" height="52" rx="${Math.max(radius, 10)}" fill="${t.card}" stroke="${t.edge}" stroke-width="1.5"/>
  <circle cx="126" cy="446" r="5" fill="${t.grn}"/><text x="144" y="453">۹+ پروتکل</text>
  <rect x="280" y="420" width="188" height="52" rx="${Math.max(radius, 10)}" fill="${t.card}" stroke="${t.edge}" stroke-width="1.5"/>
  <circle cx="308" cy="446" r="5" fill="${t.accent}"/><text x="326" y="453">TUN و حالت گیم</text>
  <rect x="482" y="420" width="178" height="52" rx="${Math.max(radius, 10)}" fill="${t.card}" stroke="${t.edge}" stroke-width="1.5"/>
  <circle cx="510" cy="446" r="5" fill="${t.grn}"/><text x="528" y="453">بدون آنالیتیکس</text>
</g>

<!-- footer -->
<text x="98" y="556" font-family="Vazirmatn,'Segoe UI',Tahoma,sans-serif" font-size="21"
      fill="${t.mut}">متن‌باز · رایگان · github.com/shabgard-app/shabgard</text>
<text x="1182" y="600" text-anchor="end" font-family="'Segoe UI',Arial,sans-serif"
      font-size="17" letter-spacing="3" fill="${dark ? t.mut : t.mut}" opacity=".8">${t.fa.toUpperCase()}</text>
</svg>`;
}

(async () => {
  const browser = await chromium.launch();
  const outDir = path.join(__dirname);
  for (const t of THEMES) {
    const svgPath = path.join(outDir, `banner-${t.id}.svg`);
    fs.writeFileSync(svgPath, bannerSvg(t), "utf8");
    const page = await browser.newPage({ viewport: { width: 1280, height: 640 }, deviceScaleFactor: 2 });
    await page.goto("file:///" + svgPath.replace(/\\/g, "/"));
    await page.waitForTimeout(300);
    await page.screenshot({ path: path.join(outDir, `banner-${t.id}.png`) });
    await page.close();
    console.log("done:", t.id);
  }
  await browser.close();
})();
