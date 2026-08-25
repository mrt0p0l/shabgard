; شبگرد — هوک‌های نصابِ NSIS (Tauri v2: installerHooks)
;
; ۱) PREINSTALL: هسته‌ها (xray/sing-box/wintun/geoip) دانلود می‌شوند —
;    به‌جای bundle داخلِ installer (۴۰MB → ~۱۰MB). اگر دانلود شکست،
;    پیام می‌دهد ولی نصب ادامه می‌یابد (کاربر می‌تواند هسته را دستی بگذارد).
; ۲) UNINSTALL: اپ/هسته‌ها بسته + کل داده‌ها و کلید استارتاپ پاک می‌شوند.

!macro NSIS_HOOK_PREINSTALL
  DetailPrint "دانلودِ هسته‌ها… (xray / sing-box)"
  CreateDirectory "$INSTDIR\binaries"
  ; ── xray v26.3.27 ──
  nsExec::ExecToLog 'powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri \"https://github.com/XTLS/Xray-core/releases/download/v26.3.27/Xray-windows-64.zip\" -OutFile \"$TEMP\xray.zip\" -UseBasicParsing"'
  nsExec::ExecToLog 'powershell -NoProfile -Command "Expand-Archive -Force \"$TEMP\xray.zip\" \"$TEMP\xrayex\""; Copy-Item \"$TEMP\xrayex\xray.exe\" \"$INSTDIR\binaries\"; Copy-Item \"$TEMP\xrayex\geoip.dat\" \"$INSTDIR\binaries\"'
  Delete "$TEMP\xray.zip"
  RMDir /r "$TEMP\xrayex"
  ; ── sing-box v1.13.19 ──
  nsExec::ExecToLog 'powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri \"https://github.com/SagerNet/sing-box/releases/download/v1.13.19/sing-box-1.13.19-windows-amd64.zip\" -OutFile \"$TEMP\sb.zip\" -UseBasicParsing"'
  nsExec::ExecToLog 'powershell -NoProfile -Command "Expand-Archive -Force \"$TEMP\sb.zip\" \"$TEMP\sbin\"; $f=(Get-ChildItem \"$TEMP\sbin\" -Recurse -Filter sing-box.exe).FullName; Copy-Item $f \"$INSTDIR\binaries\""'
  Delete "$TEMP\sb.zip"
  RMDir /r "$TEMP\sbin"
  ; ── wintun 0.14.1 ──
  nsExec::ExecToLog 'powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri \"https://www.wintun.net/builds/wintun-0.14.1.zip\" -OutFile \"$TEMP\wt.zip\" -UseBasicParsing"'
  nsExec::ExecToLog 'powershell -NoProfile -Command "Expand-Archive -Force \"$TEMP\wt.zip\" \"$TEMP\wtx\"; Copy-Item \"$TEMP\wtx\wintun\bin\amd64\wintun.dll\" \"$INSTDIR\binaries\""'
  Delete "$TEMP\wt.zip"
  RMDir /r "$TEMP\wtx"
  DetailPrint "هسته‌ها آماده شدند"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; اپ ممکن است در سینی باشد — اول ببندش تا فایل‌ها آزاد شوند
  nsExec::Exec 'taskkill /f /im shabgard.exe'
  nsExec::Exec 'taskkill /f /im xray.exe'
  nsExec::Exec 'taskkill /f /im sing-box.exe'
  Sleep 800
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Shabgard"
  RMDir /r "$LOCALAPPDATA\Shabgard"
!macroend
