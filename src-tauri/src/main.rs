// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // âœ… Sorun 1: Panic handler â€” uygulama Ã§Ã¶kerse proxy'yi temizle
    // Bu sayede kullanÄ±cÄ± internet eriÅŸimini kaybetmez
    std::panic::set_hook(Box::new(|panic_info| {
        // Proxy'yi temizlemeye Ã§alÄ±ÅŸ (best-effort)
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            // ProxyEnable = 0 yap
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                    "/v",
                    "ProxyEnable",
                    "/t",
                    "REG_DWORD",
                    "/d",
                    "0",
                    "/f",
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .status();

            // ProxyServer değerini temizle
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                    "/v",
                    "ProxyServer",
                    "/t",
                    "REG_SZ",
                    "/d",
                    "",
                    "/f",
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .status();

            // AutoConfigURL değerini temizle (PAC modu için)
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                    "/v",
                    "AutoConfigURL",
                    "/t",
                    "REG_SZ",
                    "/d",
                    "",
                    "/f",
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .status();

            // Zombi bypass-proxy sÃ¼reÃ§lerini de Ã¶ldÃ¼r
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/IM", "xzual-proxy.exe"])
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }

        eprintln!("XzualDPI PANIC: {}", panic_info);
    }));

    xzual_tauri_lib::run()
}

