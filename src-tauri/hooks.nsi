; â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
; XzualDPI NSIS Installer Hooks
; â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

; â”€â”€â”€ KURULUM Ã–NCESÄ° â”€â”€â”€
!macro NSIS_HOOK_PREINSTALL
    ; Eski sÃ¼rÃ¼m Ã§alÄ±ÅŸÄ±yorsa kapat
    nsExec::ExecToStack 'taskkill /F /IM XzualDPI.exe'
    Pop $0
    nsExec::ExecToStack 'taskkill /F /IM Xzual-proxy.exe'
    Pop $0
    Sleep 500

    ; Proxy temizle (upgrade sÄ±rasÄ±nda internet kopmasÄ±n)
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Internet Settings" "ProxyEnable" 0
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Internet Settings" "ProxyServer"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Internet Settings" "ProxyOverride"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Internet Settings" "AutoConfigURL"

    ; WinHTTP sÄ±fÄ±rla
    nsExec::ExecToStack 'netsh winhttp reset proxy'
    Pop $0

    ; Sentinel temizle
    Delete "$TEMP\xzualdpi_proxy_active.lock"
    Delete "$TEMP\xzualdpi_sidecar.pid"
!macroend

; â”€â”€â”€ KALDIRMA Ã–NCESÄ° â”€â”€â”€
!macro NSIS_HOOK_PREUNINSTALL
    ; 0. UygulamayÄ± kapat
    nsExec::ExecToStack 'taskkill /F /IM XzualDPI.exe'
    Pop $0
    Sleep 1000

    ; 1. Proxy ayarlarÄ±nÄ± sÄ±fÄ±rla (WinINet)
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Internet Settings" "ProxyEnable" 0
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Internet Settings" "ProxyServer"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Internet Settings" "ProxyOverride"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Internet Settings" "AutoConfigURL"

    ; 1.5 WinHTTP Proxy ayarlarÄ±nÄ± sÄ±fÄ±rla (Kritik: Arka plan servisleri bozulmasÄ±n)
    nsExec::ExecToStack 'netsh winhttp reset proxy'
    Pop $0

    ; 2. Sentinel ve PID dosyalarÄ±nÄ± temizle
    Delete "$TEMP\xzualdpi_proxy_active.lock"
    Delete "$TEMP\xzualdpi_sidecar.pid"

    ; 3. Zombi sidecar Ã¶ldÃ¼r
    nsExec::ExecToStack 'taskkill /F /IM Xzual-proxy.exe'
    Pop $0

    ; 4. Firewall kurallarÄ±nÄ± temizle
    nsExec::ExecToStack 'netsh advfirewall firewall delete rule name=XzualDPI_Proxy'
    Pop $0
    nsExec::ExecToStack 'netsh advfirewall firewall delete rule name=XzualDPI_PAC'
    Pop $0

    ; 5. Autostart registry kaydÄ±nÄ± temizle
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "XzualDPI"

    ; 6. DNS Ã¶nbelleÄŸini temizle
    nsExec::ExecToStack 'ipconfig /flushdns'
    Pop $0
!macroend

