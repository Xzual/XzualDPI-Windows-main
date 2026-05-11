// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use local_ip_address::list_afinet_netifas;
use std::io::Write;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem};

struct TrayMenuState {
    menu: Mutex<Option<Menu<tauri::Wry>>>,
}


// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// P0-FIX-1: Sentinel dosyasÄ± sistemi â€” crash sonrasÄ± proxy kurtarma
// P0-FIX-2: Orijinal proxy ayarlarÄ± yedekleme / geri yÃ¼kleme
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

#[cfg(target_os = "windows")]
mod registry {
    use winreg::enums::*;
    use winreg::RegKey;

    const INTERNET_SETTINGS: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

    pub fn read_value_string(name: &str) -> Option<String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu.open_subkey(INTERNET_SETTINGS).ok()?;
        let val: String = key.get_value(name).ok()?;
        Some(val)
    }

    pub fn read_value_dword(name: &str) -> Option<u32> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu.open_subkey(INTERNET_SETTINGS).ok()?;
        key.get_value(name).ok()
    }

    pub fn set_proxy(proxy_addr: &str, port: u16) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(INTERNET_SETTINGS)
            .map_err(|e| format!("Registry aÃ§Ä±lamadÄ±: {}", e))?;

        key.set_value("ProxyServer", &format!("{}:{}", proxy_addr, port))
            .map_err(|e| format!("ProxyServer: {}", e))?;
        key.set_value("ProxyEnable", &1u32)
            .map_err(|e| format!("ProxyEnable: {}", e))?;
        let proxy_override = [
            "<local>",
            // âœ… FIX: LAN IP aralÄ±klarÄ± â€” olmazsa tarayÄ±cÄ± LAN'daki PAC sunucusuna
            //    SpoofDPI proxy Ã¼zerinden gider â†’ dÃ¶ngÃ¼ â†’ timeout
            "10.*",
            "172.16.*",
            "172.17.*",
            "172.18.*",
            "172.19.*",
            "172.20.*",
            "172.21.*",
            "172.22.*",
            "172.23.*",
            "172.24.*",
            "172.25.*",
            "172.26.*",
            "172.27.*",
            "172.28.*",
            "172.29.*",
            "172.30.*",
            "172.31.*",
            "192.168.*",
            // NCSI â€” WiFi "internet yok" simgesi fix
            "*.msftconnecttest.com",
            "*.msftncsi.com",
            "dns.msn.com",
            "ipv6.msftconnecttest.com",
            // Android/iOS connectivity check
            "connectivitycheck.gstatic.com",
            "connectivitycheck.android.com",
            "clients3.google.com",
            "play.googleapis.com",
            "captive.apple.com",
            "gsp1.apple.com",
            "connectivitycheck.samsung.com",
            // Windows Update
            "*.windowsupdate.com",
            "*.delivery.mp.microsoft.com",
            // â”€â”€ Oyun & Uygulama Launcher/Updater Bypass â”€â”€
            // Bu domainler DPI ile engellenmez ama bazÄ± uygulamalarÄ±n C++ HTTP
            // istemcileri SpoofDPI'nin TLS parÃ§alamasÄ±yla uyumsuz Ã§alÄ±ÅŸabilir.
            // Bypass ile direkt baÄŸlansÄ±nlar, oyun/uygulama trafiÄŸi proxy'den geÃ§sin.
            //
            // Steam
            "*.steamcontent.com",
            "*.steamstatic.com",
            "clientconfig.akamai.steamstatic.com",
            "*.cm.steampowered.com",
            // Epic Games
            "*.epicgames.com",
            "*.unrealengine.com",
            "download.epicgames.com",
            "launcher-public-service-prod06.ol.epicgames.com",
            // Riot Games (LoL, Valorant)
            "*.riotgames.com",
            "*.leagueoflegends.com",
            "riotgames-update.akamaized.net",
            // EA / Origin
            "*.ea.com",
            "*.origin.com",
            // Blizzard / Battle.net
            "*.blizzard.com",
            "*.battle.net",
            "blzddist1-a.akamaihd.net",
            // Ubisoft
            "*.ubisoft.com",
            "*.ubi.com",
            // Microsoft / Xbox
            "*.xboxlive.com",
            "*.xbox.com",
            "*.microsoft.com",
            // Genel CDN'ler (installer/updater daÄŸÄ±tÄ±mÄ±)
            "*.cachefly.net",
        ]
        .join(";");
        key.set_value("ProxyOverride", &proxy_override)
            .map_err(|e| format!("ProxyOverride: {}", e))?;
        Ok(())
    }

    pub fn clear_proxy() -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(INTERNET_SETTINGS)
            .map_err(|e| format!("Registry aÃ§Ä±lamadÄ±: {}", e))?;

        key.set_value("ProxyEnable", &0u32)
            .map_err(|e| format!("ProxyEnable: {}", e))?;
        let _ = key.delete_value("ProxyServer");
        let _ = key.delete_value("ProxyOverride");
        let _ = key.delete_value("AutoConfigURL");
        Ok(())
    }

    pub fn set_pac_proxy(pac_url: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(INTERNET_SETTINGS)
            .map_err(|e| format!("Registry aÃ§Ä±lamadÄ±: {}", e))?;

        key.set_value("AutoConfigURL", &pac_url.to_string())
            .map_err(|e| format!("AutoConfigURL: {}", e))?;
        key.set_value("ProxyEnable", &0u32)
            .map_err(|e| format!("ProxyEnable: {}", e))?;
        let _ = key.delete_value("ProxyServer");
        let _ = key.delete_value("ProxyOverride");
        Ok(())
    }

    pub fn restore_proxy(
        server: &str,
        enable: u32,
        override_val: Option<&str>,
    ) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(INTERNET_SETTINGS)
            .map_err(|e| format!("Registry aÃ§Ä±lamadÄ±: {}", e))?;

        key.set_value("ProxyServer", &server)
            .map_err(|e| format!("ProxyServer: {}", e))?;
        key.set_value("ProxyEnable", &enable)
            .map_err(|e| format!("ProxyEnable: {}", e))?;
        if let Some(ov) = override_val {
            key.set_value("ProxyOverride", &ov)
                .map_err(|e| format!("ProxyOverride: {}", e))?;
        }
        Ok(())
    }

    pub fn can_access() -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey(INTERNET_SETTINGS).is_ok()
    }
}

/// Sentinel dosya yolu â€” proxy aktifken var, kapanÄ±nca silinir.
/// Crash/BSOD/force-kill sonrasÄ± hÃ¢lÃ¢ duruyorsa â†’ dirty shutdown algÄ±lanÄ±r.
fn sentinel_path() -> std::path::PathBuf {
    std::env::temp_dir().join("xzualdpi_proxy_active.lock")
}

/// PAC dosyasÄ± yolu â€” AutoConfigURL ile proxy yapÄ±landÄ±rmasÄ± iÃ§in

/// Orijinal proxy ayarlarÄ±nÄ± tutan yapÄ±
#[derive(Debug, Clone, Default)]
struct OriginalProxySettings {
    proxy_enable: Option<u32>,
    proxy_server: Option<String>,
    proxy_override: Option<String>,
}

/// Orijinal proxy ayarlarÄ±nÄ± saklayan global state
fn original_proxy_store() -> &'static Mutex<Option<OriginalProxySettings>> {
    static STORE: OnceLock<Mutex<Option<OriginalProxySettings>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(None))
}

/// Proxy ayarlarÄ±nÄ± set etmeden Ã–NCE mevcut deÄŸerleri yedekler
#[cfg(target_os = "windows")]
fn backup_proxy_settings() {
    let settings = OriginalProxySettings {
        proxy_enable: registry::read_value_dword("ProxyEnable"),
        proxy_server: registry::read_value_string("ProxyServer"),
        proxy_override: registry::read_value_string("ProxyOverride"),
    };

    if let Ok(mut guard) = original_proxy_store().lock() {
        // Sadece ilk backup'Ä± al â€” sonraki set_system_proxy Ã§aÄŸrÄ±larÄ± Ã¼zerine yazmasÄ±n
        if guard.is_none() {
            eprintln!("[PROXY-BACKUP] Orijinal ayarlar yedeklendi: {:?}", settings);
            *guard = Some(settings);
        }
    }
}

/// Yedeklenen proxy ayarlarÄ±nÄ± geri yÃ¼kler.
/// EÄŸer orijinal ayarlarda proxy aktifse â†’ geri yÃ¼kle
/// EÄŸer orijinal ayarlarda proxy yoksa â†’ sil (mevcut davranÄ±ÅŸ)
#[cfg(target_os = "windows")]
fn restore_proxy_settings() -> bool {
    let original = match original_proxy_store().lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => {
            eprintln!("[WARN] proxy backup lock poisoned, recovering");
            poisoned.into_inner().clone()
        }
    };

    if let Some(orig) = original {
        // Orijinal ProxyServer varsa geri yÃ¼kle (kurumsal proxy korumasÄ±)
        if let Some(ref server) = orig.proxy_server {
            if !server.is_empty() && !server.starts_with("127.0.0.1:") {
                eprintln!("[PROXY-RESTORE] Kurumsal proxy geri yÃ¼kleniyor: {}", server);

                let enable_val = orig.proxy_enable.unwrap_or(0);
                let _ = registry::restore_proxy(server, enable_val, orig.proxy_override.as_deref());

                return true; // Geri yÃ¼kleme yapÄ±ldÄ±, silme iÅŸlemine geÃ§me
            }
        }
    }
    // Orijinal proxy yoktu veya bizimkiyle aynÄ±ydÄ± â†’ normal silme prosedÃ¼rÃ¼ (mevcut davranÄ±ÅŸ)
    false
}

/// Sanal aÄŸ adaptÃ¶rlerini filtreleyen akÄ±llÄ± LAN IP bulucu.
/// VirtualBox, VMware, Hamachi, VPN gibi sanal adaptÃ¶rleri atlar.
fn get_safe_lan_ip() -> String {
    // Filtrelenecek sanal adaptÃ¶r anahtar kelimeleri (kÃ¼Ã§Ã¼k harf)
    const VIRTUAL_KEYWORDS: &[&str] = &[
        "virtual",
        "vmware",
        "vmnet",
        "vbox",
        "virtualbox",
        "pseudo",
        "hamachi",
        "vpn",
        "vethernet",
        "loopback",
        "docker",
        "wsl",
        "hyper-v",
        "bluetooth",
        "teredo",
        "isatap",
        "6to4",
        "tap-",
        "tun",
        "warp",
        "tailscale",
        "zerotier",
        "nordlynx",
        "wireguard",
        "proton",
        "mullvad",
        "windscribe",
        "surfshark",
        "host-only",
        "hostonly",
        "vEthernet",
        "npcap",
        "miniport",
    ];

    /// Bilinen sanal aÄŸ IP aralÄ±klarÄ±nÄ± kontrol eder.
    /// AdaptÃ¶r adÄ± filtreleri yakalayamadÄ±ÄŸÄ±nda (Windows generic isimlendirme) bu devreye girer.
    fn is_virtual_ip_range(ip: &std::net::Ipv4Addr) -> bool {
        let octets = ip.octets();
        match (octets[0], octets[1]) {
            // VirtualBox Host-Only: 192.168.56.x (varsayÄ±lan)
            (192, 168) if octets[2] == 56 => true,
            // VMware NAT: 192.168.19x.x
            (192, 168) if octets[2] >= 190 => true,
            // Docker default bridge: 172.17.x.x
            (172, 17) => true,
            // WSL: 172.x.x.x (genellikle 172.16-31 arasÄ± ama 172.17+ sanal olma ihtimali yÃ¼ksek)
            // Hamachi: 25.x.x.x
            (25, _) => true,
            // APIPA (otomatik atanmÄ±ÅŸ, aÄŸ baÄŸlantÄ±sÄ± yok): 169.254.x.x
            (169, 254) => true,
            _ => false,
        }
    }

    if let Ok(netifs) = list_afinet_netifas() {
        // Debug: TÃ¼m arayÃ¼zleri logla (sorun tespiti iÃ§in)
        for (name, ip) in &netifs {
            eprintln!("[NET-DEBUG] Interface: '{}' â†’ {}", name, ip);
        }

        // PASS 1: GerÃ§ek adaptÃ¶r + gerÃ§ek IP aralÄ±ÄŸÄ±
        for (name, ip) in &netifs {
            if let IpAddr::V4(v4) = ip {
                if v4.is_loopback() || v4.is_link_local() {
                    continue;
                }
                let name_lower = name.to_lowercase();
                let is_virtual_name = VIRTUAL_KEYWORDS.iter().any(|kw| name_lower.contains(kw));
                let is_virtual_range = is_virtual_ip_range(v4);

                if !is_virtual_name && !is_virtual_range {
                    eprintln!(
                        "[NET-SELECT] âœ… GerÃ§ek adaptÃ¶r seÃ§ildi: '{}' â†’ {}",
                        name, v4
                    );
                    return v4.to_string();
                }
            }
        }

        // PASS 2: Fallback â€” ad filtresi atlayÄ±p sadece IP aralÄ±ÄŸÄ± kontrol et
        for (name, ip) in &netifs {
            if let IpAddr::V4(v4) = ip {
                if !v4.is_loopback() && !v4.is_link_local() && !is_virtual_ip_range(v4) {
                    eprintln!("[NET-SELECT] âš ï¸ Fallback adaptÃ¶r: '{}' â†’ {}", name, v4);
                    return v4.to_string();
                }
            }
        }

        // PASS 3: Son Ã§are â€” sanal bile olsa bir IP ver (sadece loopback olmasÄ±n)
        for (_, ip) in &netifs {
            if let IpAddr::V4(v4) = ip {
                if !v4.is_loopback() {
                    return v4.to_string();
                }
            }
        }
    }

    "127.0.0.1".to_string()
}

/// Basit string hash â€” PAC body deÄŸiÅŸti mi kontrolÃ¼ iÃ§in
fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

/// Ã–n-derlenmiÅŸ PAC HTTP yanÄ±tÄ± â€” her istekte format! Ã§aÄŸÄ±rmaz
pub struct PacCache {
    pub pac_response: Vec<u8>,
    pub body_hash: u64,
}

/// PAC sunucusu durumu: thread handle + shutdown flag + dinamik body
pub struct PacServerState {
    pub join_handle: Mutex<Option<thread::JoinHandle<()>>>,
    pub shutdown: Arc<AtomicBool>,
    pub pac_body: Arc<Mutex<String>>,
    pub pac_cache: Arc<Mutex<PacCache>>,
    pub pac_port: Mutex<u16>,
    pub pac_url: Mutex<String>,
}

impl Default for PacServerState {
    fn default() -> Self {
        Self {
            join_handle: Mutex::new(None),
            shutdown: Arc::new(AtomicBool::new(false)),
            pac_body: Arc::new(Mutex::new(make_pac_direct_body())),
            pac_cache: Arc::new(Mutex::new(PacCache {
                pac_response: Vec::new(),
                body_hash: 0,
            })),
            pac_port: Mutex::new(0),
            pac_url: Mutex::new(String::new()),
        }
    }
}

const PAC_PORT_START: u16 = 8787;
const PAC_PORT_END: u16 = 8887;
const SUPPORT_URL: &str = "https://www.patreon.com/join/ConsolAktif";

/// BaÄŸlantÄ± kesildiÄŸinde kullanÄ±lan fallback PAC: tÃ¼m trafiÄŸi DIRECT yÃ¶nlendirir
/// Bu sayede cihazlar internet eriÅŸimini kaybetmez
fn make_pac_direct_body() -> String {
    r#"function FindProxyForURL(url, host) {
    // XzualDPI proxy devre dÄ±ÅŸÄ± â€” tÃ¼m trafik doÄŸrudan Ã§Ä±kÄ±ÅŸ
    // Bu PAC dosyasÄ± otomatik olarak sunulur; ayar deÄŸiÅŸikliÄŸi gerekmez
    return "DIRECT";
}
"#
    .to_string()
}

/// Production PAC: yerel aÄŸ DIRECT, diÄŸerleri PROXY ip:port; DIRECT (fail-safe)
/// dnsResolve Ã§aÄŸrÄ±larÄ± try-catch ile korunuyor â€” DNS timeout olursa PAC script Ã§Ã¶kmez
fn make_pac_body(lan_ip: &str, proxy_port: u16, whitelist: &str, blacklist: &str) -> String {
    let proxy = format!("{}:{}", lan_ip, proxy_port);
    
    // Parse lists into JS array strings
    let wl_js = whitelist.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| format!("\"*{}*\"", l))
        .collect::<Vec<_>>()
        .join(",");
    
    let bl_js = blacklist.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| format!("\"*{}*\"", l))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"function FindProxyForURL(url, host) {{
    var whitelist = [{}];
    var blacklist = [{}];

    // 1) Blacklist Check -> DIRECT (Always skip bypass)
    for (var i = 0; i < blacklist.length; i++) {{
        if (shExpMatch(host, blacklist[i])) return "DIRECT";
    }}

    // 2) Whitelist Check -> PROXY (Always bypass)
    for (var i = 0; i < whitelist.length; i++) {{
        if (shExpMatch(host, whitelist[i])) return "PROXY {}; DIRECT";
    }}

    // 3) Localhost & Internal -> DIRECT
    if (isPlainHostName(host) ||
        host === "localhost" ||
        shExpMatch(host, "127.*") ||
        shExpMatch(host, "10.*") ||
        shExpMatch(host, "192.168.*") ||
        shExpMatch(host, "172.16.*") || shExpMatch(host, "172.17.*") ||
        shExpMatch(host, "172.18.*") || shExpMatch(host, "172.19.*") ||
        shExpMatch(host, "172.2?.*") || shExpMatch(host, "172.30.*") ||
        shExpMatch(host, "172.31.*") ||
        shExpMatch(host, "*.local") ||
        shExpMatch(host, "*.localhost") ||
        shExpMatch(host, "*.internal"))
        return "DIRECT";

    // 4) Default Bypass
    return "PROXY {}; DIRECT";
}}"#,
        wl_js, bl_js, proxy, proxy
    )
}

fn make_setup_html(pac_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="tr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=0">
<title>XzualDPI - Setup</title>
<style>
:root {{
    --bg: #09090b;
    --card: #18181b;
    --accent: #3b82f6;
    --accent-glow: rgba(59, 130, 246, 0.3);
    --text: #ffffff;
    --text-dim: #a1a1aa;
    --success: #10b981;
}}
* {{ box-sizing: border-box; margin: 0; padding: 0; font-family: 'Inter', system-ui, -apple-system, sans-serif; }}
body {{ 
    background-color: var(--bg); 
    color: var(--text); 
    display: flex; 
    flex-direction: column; 
    align-items: center; 
    min-height: 100vh; 
    padding: 20px;
    background-image: radial-gradient(circle at top right, rgba(59, 130, 246, 0.05), transparent);
}}
.container {{ width: 100%; max-width: 400px; }}
.header {{ text-align: center; margin-bottom: 30px; margin-top: 20px; }}
.logo-icon {{ font-size: 40px; margin-bottom: 10px; filter: drop-shadow(0 0 10px var(--accent-glow)); }}
.title {{ font-size: 24px; font-weight: 800; letter-spacing: -0.5px; }}
.subtitle {{ font-size: 14px; color: var(--text-dim); margin-top: 5px; }}

.card {{ 
    background: var(--card); 
    border: 1px solid rgba(255,255,255,0.1); 
    border-radius: 24px; 
    padding: 24px; 
    box-shadow: 0 20px 40px rgba(0,0,0,0.4);
}}
.card-title {{ font-weight: 700; margin-bottom: 20px; display: flex; align-items: center; gap: 10px; font-size: 16px; }}

.input-box {{
    background: #27272a;
    border: 1px solid #3f3f46;
    border-radius: 12px;
    padding: 14px;
    color: var(--accent);
    font-family: monospace;
    font-size: 14px;
    word-break: break-all;
    margin-bottom: 15px;
    text-align: center;
}}

.btn {{
    width: 100%;
    padding: 14px;
    border-radius: 12px;
    border: none;
    background: var(--accent);
    color: white;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 4px 12px var(--accent-glow);
}}
.btn:active {{ transform: scale(0.98); }}
.btn.success {{ background: var(--success); box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3); }}

.steps {{ margin-top: 30px; list-style: none; }}
.step {{ display: flex; gap: 15px; margin-bottom: 20px; }}
.step-num {{ 
    flex: none; 
    width: 24px; 
    height: 24px; 
    background: rgba(255,255,255,0.1); 
    border-radius: 50%; 
    display: flex; 
    align-items: center; 
    justify-content: center; 
    font-size: 12px; 
    font-weight: 800;
}}
.step-text b {{ display: block; font-size: 14px; margin-bottom: 4px; color: var(--text); }}
.step-text p {{ font-size: 13px; color: var(--text-dim); line-height: 1.4; }}

.footer {{ margin-top: auto; padding: 30px; font-size: 12px; color: var(--text-dim); text-align: center; }}
</style>
</head>
<body>
    <div class="container">
        <header class="header">
            <div class="logo-icon">ğŸ›¡ï¸</div>
            <h1 class="title">XzualDPI</h1>
            <p class="subtitle" data-tr="Mobil Kurulum SayfasÄ±" data-en="Mobile Setup Page">Mobil Kurulum SayfasÄ±</p>
        </header>

        <div class="card">
            <div class="card-title">ğŸ“± <span data-tr="Otomatik YapÄ±landÄ±rma" data-en="Auto Config">Otomatik YapÄ±landÄ±rma</span></div>
            <div class="input-box" id="url-box">{}</div>
            <button class="btn" id="copy-btn" data-tr="Adresi Kopyala" data-en="Copy Address">Adresi Kopyala</button>
            
            <div class="steps">
                <div class="step">
                    <div class="step-num">1</div>
                    <div class="step-text">
                        <b data-tr="Wi-Fi AyarlarÄ±na Gidin" data-en="Go to Wi-Fi Settings">Wi-Fi AyarlarÄ±na Gidin</b>
                        <p data-tr="BaÄŸlÄ± olduÄŸunuz aÄŸÄ±n yanÄ±ndaki Ayarlar simgesine dokunun." data-en="Tap the Settings icon next to your connected network.">BaÄŸlÄ± olduÄŸunuz aÄŸÄ±n yanÄ±ndaki Ayarlar simgesine dokunun.</p>
                    </div>
                </div>
                <div class="step">
                    <div class="step-num">2</div>
                    <div class="step-text">
                        <b data-tr="Proxy'yi Otomatik YapÄ±n" data-en="Set Proxy to Automatic">Proxy'yi Otomatik YapÄ±n</b>
                        <p data-tr="Proxy ayarÄ±nÄ± 'Otomatik' veya 'PAC' olarak deÄŸiÅŸtirin." data-en="Change Proxy setting to 'Automatic' or 'PAC'.">Proxy ayarÄ±nÄ± 'Otomatik' veya 'PAC' olarak deÄŸiÅŸtirin.</p>
                    </div>
                </div>
                <div class="step">
                    <div class="step-num">3</div>
                    <div class="step-text">
                        <b data-tr="URL'yi YapÄ±ÅŸtÄ±rÄ±n" data-en="Paste the URL">URL'yi YapÄ±ÅŸtÄ±rÄ±n</b>
                        <p data-tr="YukarÄ±daki kopyaladÄ±ÄŸÄ±nÄ±z adresi PAC URL kÄ±smÄ±na yapÄ±ÅŸtÄ±rÄ±p kaydedin." data-en="Paste the copied address into the PAC URL field and save.">YukarÄ±daki kopyaladÄ±ÄŸÄ±nÄ±z adresi PAC URL kÄ±smÄ±na yapÄ±ÅŸtÄ±rÄ±p kaydedin.</p>
                    </div>
                </div>
            </div>
        </div>

        <div class="footer">
            &copy; 2024 XzualDPI Premium â€¢ Consolas
        </div>
    </div>

    <script>
        const url = document.getElementById('url-box').innerText;
        const btn = document.getElementById('copy-btn');
        
        btn.onclick = () => {{
            navigator.clipboard.writeText(url).then(() => {{
                const originalText = btn.innerText;
                btn.innerText = 'âœ“ KopyalandÄ±!';
                btn.classList.add('success');
                setTimeout(() => {{
                    btn.innerText = originalText;
                    btn.classList.remove('success');
                }}, 2000);
            }});
        }};
    </script>
</body>
</html>"#,
        pac_url
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Absolute URL'den path kÄ±smÄ±nÄ± Ã§Ä±karÄ±r.
/// "http://192.168.1.5:8787/proxy.pac" â†’ "/proxy.pac"
/// "http://192.168.1.5:8787/"          â†’ "/"
/// "/proxy.pac"                         â†’ "/proxy.pac"  (zaten relative)
fn normalize_path(raw: &str) -> &str {
    if let Some(pos) = raw.find("://") {
        let after_scheme = &raw[pos + 3..];
        if let Some(slash_pos) = after_scheme.find('/') {
            return &after_scheme[slash_pos..];
        }
        return "/";
    }
    raw
}

fn handle_pac_request(
    stream: TcpStream,
    pac_body: &Arc<Mutex<String>>,
    pac_cache: &Arc<Mutex<PacCache>>,
    pac_url: &str,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    let mut reader = std::io::BufReader::new(stream);
    let mut first_line = String::new();

    if std::io::BufRead::read_line(&mut reader, &mut first_line).is_err() || first_line.is_empty() {
        return;
    }

    // TCP RST Ã¶nleme â€” request header'larÄ± tamamen tÃ¼ketilmeli
    let mut discard = String::new();
    while let Ok(n) = std::io::BufRead::read_line(&mut reader, &mut discard) {
        if n <= 2 {
            break;
        }
        discard.clear();
    }

    let mut stream = reader.into_inner();

    // âœ… FIX: Absolute URL desteÄŸi â€” proxy Ã¼zerinden gelen istekleri de doÄŸru parse et
    let raw_path = first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/");
    let path = normalize_path(raw_path);

    let is_get = first_line.to_uppercase().starts_with("GET ");

    // â”€â”€ 1) Logo â”€â”€
    if is_get && path == "/logo" {
        let img = include_bytes!("../icons/128x128.png");
        let hdr = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
            img.len()
        );
        let _ = stream.write_all(hdr.as_bytes());
        let _ = stream.write_all(img);
        let _ = stream.flush();
        return;
    }

    // â”€â”€ 2) PAC dosyasÄ± (/proxy.pac veya /wpad.dat) â”€â”€
    // BazÄ± senaryolarda tarayÄ±cÄ±lar absolute URL (http://ip:port/proxy.pac) olarak gÃ¶nderebilir.
    if is_get && (path.ends_with("/proxy.pac") || path.ends_with("/wpad.dat")) {
        let current_body = pac_body
            .lock()
            .map(|b| b.clone())
            .unwrap_or_else(|_| make_pac_direct_body());
        let current_hash = simple_hash(&current_body);

        // Dinamik Cache-Control: PROXY aktifken 60s, DIRECT modda 0
        let is_direct_mode = !current_body.contains("PROXY");
        let cache_header = if is_direct_mode {
            "Cache-Control: no-cache, no-store, must-revalidate, max-age=0"
        } else {
            "Cache-Control: max-age=60"
        };

        let mode_bit: u64 = if is_direct_mode { 1 } else { 0 };
        let cache_key = current_hash.wrapping_add(mode_bit);

        if let Ok(mut cache) = pac_cache.lock() {
            if cache.body_hash != cache_key || cache.pac_response.is_empty() {
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n{}\r\nContent-Length: {}\r\n\r\n{}",
                    cache_header,
                    current_body.len(),
                    current_body
                );
                cache.pac_response = response.into_bytes();
                cache.body_hash = cache_key;
            }
            let _ = stream.write_all(&cache.pac_response);
        } else {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n{}\r\nContent-Length: {}\r\n\r\n{}",
                cache_header,
                current_body.len(),
                current_body
            );
            let _ = stream.write_all(response.as_bytes());
        }
        let _ = stream.flush();
        return; // â† Ã–NEMLÄ°: Burada fonksiyondan Ã§Ä±k
    }

    // â”€â”€ 3) GET olmayan istekler â”€â”€
    if !is_get {
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n");
        let _ = stream.flush();
        return;
    }

    // â”€â”€ 4) HTML kurulum sayfasÄ± (/) veya 404 â”€â”€
    let (status, content_type, body) = if path == "/" || path.is_empty() {
        (
            "200 OK",
            "text/html; charset=utf-8",
            make_setup_html(pac_url),
        )
    } else {
        ("404 Not Found", "text/plain", String::new())
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

#[derive(serde::Serialize)]
struct PacResponse {
    pac_port: u16,
}

/// P1-FIX: PAC sunucusu eÅŸzamanlÄ± baÄŸlantÄ± limiti
const MAX_PAC_CONNECTIONS: u32 = 50;

#[cfg(target_os = "windows")]
fn manage_firewall_rules(enable: bool, proxy_port: u16, pac_port: u16) {
    std::thread::spawn(move || {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // Ã–nce mevcut kurallarÄ± temizle
        let _ = std::process::Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                "name=XzualDPI_Proxy",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        let _ = std::process::Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                "name=XzualDPI_PAC",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if enable {
            let _ = std::process::Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "add",
                    "rule",
                    "name=XzualDPI_Proxy",
                    "dir=in",
                    "action=allow",
                    "protocol=TCP",
                    &format!("localport={}", proxy_port),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            let _ = std::process::Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "add",
                    "rule",
                    "name=XzualDPI_PAC",
                    "dir=in",
                    "action=allow",
                    "protocol=TCP",
                    &format!("localport={}", pac_port),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    });
}

#[tauri::command]
fn start_pac_server(
    proxy_port: u16,
    whitelist: String,
    blacklist: String,
    state: tauri::State<'_, PacServerState>,
) -> Result<PacResponse, String> {
    let lan_ip = get_safe_lan_ip();

    // PAC body'yi gÃ¼ncelle â€” proxy moduna geÃ§
    let new_pac_body = make_pac_body(&lan_ip, proxy_port, &whitelist, &blacklist);
    if let Ok(mut body) = state.pac_body.lock() {
        *body = new_pac_body;
    }

    // Sunucu zaten Ã§alÄ±ÅŸÄ±yorsa, sadece body gÃ¼ncellendi â€” port bilgisini dÃ¶ndÃ¼r
    let guard = state.join_handle.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        let current_port = *state.pac_port.lock().map_err(|e| e.to_string())?;
        // PAC URL'yi de gÃ¼ncelle (port aynÄ± kalsa bile proxy_port deÄŸiÅŸmiÅŸ olabilir)
        if let Ok(mut url) = state.pac_url.lock() {
            *url = format!("http://{}:{}/proxy.pac", lan_ip, current_port);
        }
        return Ok(PacResponse {
            pac_port: current_port,
        });
    }
    drop(guard); // Lock'u serbest bÄ±rak

    // P1-FIX: LAN paylaÅŸÄ±mÄ± her zaman 0.0.0.0'a bind eder (fonksiyon zaten sadece LAN aktifken Ã§aÄŸrÄ±lÄ±r)
    // Ama yerel cihazlarÄ±n gÃ¼venliÄŸi iÃ§in bind adresi sabitlenir
    let bind_addr = "0.0.0.0";

    // Dinamik PAC port: 8787-8887 arasÄ±nda mÃ¼sait olanÄ± bul
    let mut found_port: u16 = 0;
    let mut listener_result = None;
    for port in PAC_PORT_START..=PAC_PORT_END {
        match TcpListener::bind((bind_addr, port)) {
            Ok(l) => {
                found_port = port;
                listener_result = Some(l);
                break;
            }
            Err(_) => continue,
        }
    }
    // Fallback: OS'tan rastgele port iste
    if listener_result.is_none() {
        match TcpListener::bind((bind_addr, 0u16)) {
            Ok(l) => {
                if let Ok(addr) = l.local_addr() {
                    found_port = addr.port();
                }
                listener_result = Some(l);
            }
            Err(e) => return Err(format!("PAC iÃ§in uygun port bulunamadÄ±: {}", e)),
        }
    }
    let listener = listener_result.unwrap();
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    manage_firewall_rules(true, proxy_port, found_port);

    let pac_url = format!("http://{}:{}/proxy.pac", lan_ip, found_port);

    // State'e kaydet
    if let Ok(mut p) = state.pac_port.lock() {
        *p = found_port;
    }
    if let Ok(mut u) = state.pac_url.lock() {
        *u = pac_url.clone();
    }

    let shutdown = Arc::clone(&state.shutdown);
    shutdown.store(false, Ordering::Relaxed);
    let pac_body_arc = Arc::clone(&state.pac_body);
    let pac_cache_arc = Arc::clone(&state.pac_cache);
    let pac_url_for_thread = pac_url.clone();

    // P1-FIX: Thread limiti iÃ§in atomik sayaÃ§
    let active_connections = Arc::new(std::sync::atomic::AtomicU32::new(0));

    let join_handle = thread::spawn(move || {
        while !shutdown.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let current = active_connections.load(Ordering::Relaxed);
                    if current >= MAX_PAC_CONNECTIONS {
                        drop(stream);
                        continue;
                    }
                    active_connections.fetch_add(1, Ordering::Relaxed);

                    let body = Arc::clone(&pac_body_arc);
                    let cache = Arc::clone(&pac_cache_arc);
                    let url = pac_url_for_thread.clone();
                    let conn_counter = Arc::clone(&active_connections);
                    thread::spawn(move || {
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_nodelay(true);
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                        handle_pac_request(stream, &body, &cache, &url);
                        conn_counter.fetch_sub(1, Ordering::Relaxed);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // âœ… 5ms â†’ 50ms: CPU wake-up %90 azalÄ±r, PAC latency hÃ¢lÃ¢ imperceptible
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => {}
            }
        }
    });

    let mut guard = state.join_handle.lock().map_err(|e| e.to_string())?;
    *guard = Some(join_handle);
    Ok(PacResponse {
        pac_port: found_port,
    })
}

/// BaÄŸlantÄ± kesildiÄŸinde PAC body'yi DIRECT moduna geÃ§ir.
/// Sunucu Ã§alÄ±ÅŸmaya devam eder â€” cihazlar internet eriÅŸimini kaybetmez.
#[tauri::command]
fn stop_pac_server(state: tauri::State<'_, PacServerState>) -> Result<(), String> {
    // Sunucuyu kapatmak yerine PAC body'yi DIRECT moduna geÃ§ir
    if let Ok(mut body) = state.pac_body.lock() {
        *body = make_pac_direct_body();
    }

    // âœ… P0-FIX: Cache'i hemen invalidate et
    if let Ok(mut cache) = state.pac_cache.lock() {
        cache.body_hash = 0;
        cache.pac_response.clear();
    }

    #[cfg(target_os = "windows")]
    manage_firewall_rules(false, 0, 0);

    Ok(())
}

#[derive(serde::Serialize)]
struct ConfigResponse {
    port: u16,
    lan_ip: String,
    bind_address: String,
}

#[tauri::command]
fn get_sidecar_config(
    allow_lan_sharing: bool,
    enable_game_mode: bool,
) -> Result<ConfigResponse, String> {
    // Game Mode (WinHTTP) aÃ§Ä±kken 0.0.0.0'a bind et â€” UWP uygulamalarÄ± (Roblox vb.)
    // AppContainer sandbox yÃ¼zÃ¼nden 127.0.0.1'e eriÅŸemez, LAN IP Ã¼zerinden baÄŸlanÄ±r
    let bind_addr = if allow_lan_sharing || enable_game_mode {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };

    // Ã–ncelikli Portlar: 8080 - 8090 arasÄ± kontrol et
    let mut selected_port = 0;
    for port in 8080..=8090 {
        if TcpListener::bind((bind_addr, port)).is_ok() {
            selected_port = port;
            break;
        }
    }

    // Fallback: EÄŸer hepsi doluysa, sistemden rastgele bir port iste (Port 0)
    if selected_port == 0 {
        if let Ok(listener) = TcpListener::bind((bind_addr, 0)) {
            if let Ok(addr) = listener.local_addr() {
                selected_port = addr.port();
            }
        }
    }

    if selected_port == 0 {
        return Err("Uygun port bulunamadÄ±.".to_string());
    }

    // Yerel IP Adresini Bul (LAN PaylaÅŸÄ±mÄ± iÃ§in) â€” Sanal adaptÃ¶rleri filtreler
    let lan_ip = get_safe_lan_ip();

    Ok(ConfigResponse {
        port: selected_port,
        lan_ip,
        bind_address: bind_addr.to_string(),
    })
}

/// Registry proxy iÅŸlemlerini serialize eden global lock
/// set_system_proxy ve clear_system_proxy eÅŸ zamanlÄ± Ã§aÄŸrÄ±labilir (reconnect sÄ±rasÄ±nda)
fn proxy_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// P0-FIX-3: Poisoned mutex recovery â€” panic sonrasÄ± bile proxy temizleme Ã§alÄ±ÅŸsÄ±n
fn acquire_proxy_lock() -> std::sync::MutexGuard<'static, ()> {
    match proxy_lock().lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("[WARN] Proxy lock was poisoned (previous panic?), recovering");
            poisoned.into_inner()
        }
    }
}

#[tauri::command]
fn clear_system_proxy() -> Result<(), String> {
    let _guard = acquire_proxy_lock(); // P0-FIX-3: Poisoned mutex recovery
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // P0-FIX-2: Ã–nce orijinal ayarlarÄ± geri yÃ¼klemeyi dene
        let has_original = restore_proxy_settings();

        if !has_original {
            let _ = registry::clear_proxy();
        }

        // 4. DNS Ã–nbelleÄŸini Temizle (Race condition / DNS sorunlarÄ±nÄ± Ã¶nler)
        let _ = Command::new("ipconfig")
            .arg("/flushdns")
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        // 5. Notify browsers about the change
        notify_proxy_change();

        // 6. Native/C++ ve arka plan servisleri iÃ§in WinHTTP sistem proxy'sini sÄ±fÄ±rla
        let _ = std::process::Command::new("netsh")
            .args(&["winhttp", "reset", "proxy"])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        manage_firewall_rules(false, 0, 0);
    }

    // P0-FIX-1: Sentinel dosyasÄ±nÄ± sil â€” proxy artÄ±k aktif deÄŸil
    let _ = std::fs::remove_file(sentinel_path());

    // P0-FIX-2: Backup'Ä± temizle â€” geri yÃ¼kleme tamamlandÄ±
    if let Ok(mut guard) = original_proxy_store().lock() {
        *guard = None;
    }

    Ok(())
}

/// Notify Windows that internet settings have changed
/// This forces browsers to immediately pick up the new proxy settings
#[cfg(target_os = "windows")]
fn notify_proxy_change() {
    use std::ptr::null_mut;
    use winapi::um::wininet::{
        InternetSetOptionW, INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED,
    };

    unsafe {
        // Notify that settings have changed
        InternetSetOptionW(null_mut(), INTERNET_OPTION_SETTINGS_CHANGED, null_mut(), 0);
        InternetSetOptionW(null_mut(), INTERNET_OPTION_REFRESH, null_mut(), 0);
    }
}

/// P1-FIX: UWP AppContainer'larÄ± arka planda otomatik olarak Loopback Proxy iÃ§in yetkilendirir.
/// Bu sayede Roblox, Speedtest ve diÄŸer Windows MaÄŸaza uygulamalarÄ± 127.0.0.1 proxy sunucusuna baÅŸarÄ±lÄ± ÅŸekilde baÄŸlanabilir.
#[cfg(target_os = "windows")]
fn exempt_all_uwp_apps() {
    std::thread::spawn(|| {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let script = r#"
            try {
                $packages = Get-AppxPackage -ErrorAction SilentlyContinue
                foreach ($pkg in $packages) {
                    if ($pkg.PackageFamilyName) {
                        CheckNetIsolation.exe LoopbackExempt -a "-n=$($pkg.PackageFamilyName)"
                    }
                }
            } catch {}
        "#;

        let _ = std::process::Command::new("powershell")
            .args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", script])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    });
}

#[tauri::command]
fn set_system_proxy(port: u16, enable_winhttp: bool, pac_url: Option<String>) -> Result<(), String> {
    let _guard = acquire_proxy_lock(); // P0-FIX-3: Poisoned mutex recovery
                                       // âœ… Port aralÄ±ÄŸÄ± validasyonu
    if port < 1024 {
        return Err("GeÃ§ersiz port numarasÄ± (1024-65535 arasÄ± olmalÄ±)".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;

        if !registry::can_access() {
            return Err(
                "Registry yazma izni yok. UygulamayÄ± yÃ¶netici olarak Ã§alÄ±ÅŸtÄ±rÄ±n.".to_string(),
            );
        }

        // P0-FIX-2: Proxy ayarlamadan Ã–NCE mevcut ayarlarÄ± yedekle
        backup_proxy_settings();

        // âœ… CRITICAL FIX: Asla LAN IP kullanma! Roblox vb. UWP UygulamalarÄ± 'privateNetworkClientServer'
        // yetkisine sahip DEÄÄ°LDÄ°R. Bu yÃ¼zden 192.168.x.x (LAN IP) Ã¼zerinden baÄŸlandÄ±klarÄ±nda sistem
        // gÃ¼venlik duvarÄ± (AppContainer) baÄŸlantÄ±yÄ± tamamen keser.
        // UWP LoopbackExempt (Sanal Ä°zolasyon KaldÄ±rma) SADECE "127.0.0.1" iÃ§in Ã§alÄ±ÅŸÄ±r.
        let proxy_addr = "127.0.0.1".to_string();

        if let Some(url) = pac_url {
            registry::set_pac_proxy(&url).map_err(|e| {
                let _ = registry::clear_proxy();
                format!("PAC Registry gÃ¼ncelleme baÅŸarÄ±sÄ±z: {}", e)
            })?;
        } else {
            registry::set_proxy(&proxy_addr, port).map_err(|e| {
                let _ = registry::clear_proxy();
                format!("Registry gÃ¼ncelleme baÅŸarÄ±sÄ±z, geri alÄ±ndÄ±: {}", e)
            })?;
        }

        // 3. CRITICAL: Notify Windows about the change so browsers pick it up immediately
        notify_proxy_change();

        // 4. UWP (Windows MaÄŸaza) uygulamalarÄ± iÃ§in loopback isolation yetkisini bypass et
        exempt_all_uwp_apps();

        // 5. Native/C++ ve arka plan servisleri iÃ§in WinHTTP sistem proxy'si ayarla
        if enable_winhttp {
            // WinHTTP bypass listesini Registry ProxyOverride ile senkronize tut
            let winhttp_bypass = format!(
                "bypass-list=\"<local>;{};*.steamcontent.com;*.steamstatic.com;*.cm.steampowered.com;*.epicgames.com;*.unrealengine.com;*.riotgames.com;*.leagueoflegends.com;*.ea.com;*.origin.com;*.blizzard.com;*.battle.net;*.ubisoft.com;*.ubi.com;*.xboxlive.com;*.xbox.com;*.microsoft.com;*.cachefly.net;*.msftconnecttest.com;*.windowsupdate.com\"",
                proxy_addr
            );
            let _ = std::process::Command::new("netsh")
                .args(&[
                    "winhttp",
                    "set",
                    "proxy",
                    &format!("{}:{}", proxy_addr, port),
                    &winhttp_bypass,
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }

    // P0-FIX-1: Sentinel dosyasÄ± oluÅŸtur â€” proxy artÄ±k aktif
    let _ = std::fs::write(sentinel_path(), format!("port={}", port));

    Ok(())
}

/// P1-FIX: Tooltip uzunluk sÄ±nÄ±rÄ± â€” Windows tooltip limiti 128 karakter
#[tauri::command]
fn update_tray_tooltip(app: tauri::AppHandle, tooltip: String) -> Result<(), String> {
    let sanitized: String = tooltip.chars().take(128).collect();
    if let Some(tray) = app.tray_by_id("tray") {
        let _ = tray.set_tooltip(Some(sanitized));
        
        // Menüdeki "Bağlan" metnini duruma göre güncelle
        if let Some(state) = app.try_state::<TrayMenuState>() {
            if let Ok(m_guard) = state.menu.lock() {
                if let Some(menu) = &*m_guard {
                    if let Some(item) = menu.get("toggle") {
                        if let Some(menu_item) = item.as_menuitem() {
                            let is_connected = tooltip.contains("Bağlı") || tooltip.contains("Connected");
                            let _ = menu_item.set_text(if is_connected { "Bağlantıyı Kes" } else { "Bağlan" });
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// P1-FIX: Port aralÄ±ÄŸÄ± kÄ±sÄ±tlama â€” XSS ile localhost port taramasÄ± engellenir
#[tauri::command]
fn check_port_open(port: u16) -> bool {
    // Sadece privileged portlarÄ± engelle, dinamik portlara (OS atamasÄ±) izin ver
    if port < 1024 {
        return false;
    }
    TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_millis(500),
    )
    .is_ok()
}

#[tauri::command]
fn check_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use std::ptr;
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
        use winapi::um::securitybaseapi::GetTokenInformation;
        use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};

        unsafe {
            let mut token: HANDLE = ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return false;
            }

            let mut elevation: TOKEN_ELEVATION = mem::zeroed();
            let mut size: u32 = 0;
            let result = GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut size,
            );

            CloseHandle(token);
            result != 0 && elevation.TokenIsElevated != 0
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}

fn perform_app_exit(app: &tauri::AppHandle) {
    // clear_system_proxy zaten RunEvent::ExitRequested'da Ã§aÄŸrÄ±lacak
    // Burada tekrar Ã§aÄŸÄ±rma â€” app.exit() ExitRequested tetikler
    app.exit(0);
}

/// Uygulama aÃ§Ä±ldÄ±ÄŸÄ±nda eski xzual-proxy sÃ¼reÃ§lerini temizle (Zombi sÃ¼reÃ§ Ã¶nleme)
#[tauri::command]
fn save_sidecar_pid(pid: u32) {
    let pid_file = std::env::temp_dir().join("xzualdpi_sidecar.pid");
    let _ = std::fs::write(&pid_file, pid.to_string());
}

/// Uygulama aÃ§Ä±ldÄ±ÄŸÄ±nda eski Xzual-proxy sÃ¼reÃ§lerini temizle (Zombi sÃ¼reÃ§ Ã¶nleme)
#[tauri::command]
fn kill_zombie_sidecar() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let pid_file = std::env::temp_dir().join("xzualdpi_sidecar.pid");
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if pid > 0 {
                    let output = std::process::Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string()])
                        .creation_flags(CREATE_NO_WINDOW)
                        .output();

                    let _ = std::fs::remove_file(&pid_file);

                    if let Ok(out) = output {
                        if out.status.success() {
                            return Ok(format!("Zombi sÃ¼reÃ§ (PID {}) durduruldu.", pid));
                        }
                    }
                }
            }
        }
        Ok("Zombi PID dosyasÄ± bulunamadÄ±.".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok("Zombi temizleme sadece Windows'ta desteklenir.".to_string())
    }
}

/// P0-FIX: Ortadaki Adam (Network Reconnaissance) Riskini Engellemek Ä°Ã§in Ã–zel Ping DoÄŸrulayÄ±cÄ±
#[tauri::command]
async fn check_dns_latency(dns_ip: String) -> Result<u32, String> {
    // Güvenlik kontrolü: IP'nin geçerli olduğunu doğrula
    let addr_res = format!("{}:53", dns_ip).parse::<std::net::SocketAddr>();
    if addr_res.is_err() {
        return Err("Geçersiz IP adresi".to_string());
    }
    let addr = addr_res.unwrap();

    // Özel ağları (Localhost, LAN vb.) taramayı engelle (Opsiyonel Güvenlik)
    if let std::net::IpAddr::V4(v4) = addr.ip() {
        if v4.is_loopback() || v4.is_private() || v4.is_link_local() {
            // Sadece bilinen DNS servisleri ise izin ver (P0-FIX)
            let allowed_anyway = ["1.1.1.1", "8.8.8.8", "9.9.9.9", "94.140.14.14", "208.67.222.222"];
            if !allowed_anyway.contains(&dns_ip.as_str()) {
                return Err("Güvenlik nedeniyle yerel ağ taraması yapılamaz.".to_string());
            }
        }
    }

    let start = std::time::Instant::now();
    let addr = format!("{}:53", dns_ip)
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;

    match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(1500)) {
        Ok(_) => Ok(start.elapsed().as_millis() as u32),
        Err(_) => Ok(999),
    }
}

/// P0-FIX-1: Uygulama baÅŸlangÄ±cÄ±nda crash/BSOD sonrasÄ± kalan kirli proxy'yi temizle
/// Sentinel dosyasÄ± varsa = Ã¶nceki oturum dÃ¼zgÃ¼n kapanmamÄ±ÅŸ demektir
#[tauri::command]
fn startup_proxy_cleanup() -> Result<bool, String> {
    let sentinel = sentinel_path();

    if sentinel.exists() {
        eprintln!("[STARTUP] âš ï¸ Dirty shutdown detected â€” sentinel file found");
        eprintln!("[STARTUP] Cleaning orphaned proxy settings...");

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            use std::process::Command;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            let _ = registry::clear_proxy();

            // DNS cache temizle
            let _ = Command::new("ipconfig")
                .arg("/flushdns")
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();

            // TarayÄ±cÄ±lara bildir
            notify_proxy_change();

            // âœ… Sadece dirty shutdown'da firewall temizle
            manage_firewall_rules(false, 0, 0);
        }

        let _ = std::fs::remove_file(&sentinel);
        eprintln!("[STARTUP] âœ… Orphaned proxy + firewall rules cleaned");

        return Ok(true);
    }

    // âœ… FIX: Temiz baÅŸlangÄ±Ã§ta firewall temizleme YAPMA
    // Eski kod: manage_firewall_rules(false, 0, 0) â€” autoConnect ile race condition yaratÄ±yordu
    // Sentinel yoksa zaten Ã¶nceki oturum dÃ¼zgÃ¼n kapanmÄ±ÅŸ, firewall kurallarÄ± da temizlenmiÅŸ demektir

    Ok(false) // Temiz baÅŸlangÄ±Ã§
}

// 1. SÃ¼rÃ¼cÃ¼ kontrolÃ¼ (lib.rs iÃ§ine ekle)
#[tauri::command]
fn check_driver() -> bool {
    std::path::Path::new("C:\\Windows\\System32\\wpcap.dll").exists()
        || std::path::Path::new("C:\\Windows\\SysWOW64\\wpcap.dll").exists()
}

// 2. SÃ¼rÃ¼cÃ¼ kurulumu (lib.rs iÃ§ine ekle)
#[tauri::command]
fn install_driver(app: tauri::AppHandle) -> Result<(), String> {
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("binaries/npcap-installer.exe");

    if !resource_path.exists() {
        return Err("SÃ¼rÃ¼cÃ¼ dosyasÄ± bulunamadÄ±. LÃ¼tfen uygulamayÄ± yeniden yÃ¼kleyin.".into());
    }

    // P0-FIX: Driver kurulumunu gÃ¶rÃ¼nÃ¼r yaptÄ±k (/S kaldÄ±rÄ±ldÄ±, CREATE_NO_WINDOW kaldÄ±rÄ±ldÄ±)
    // Bu sayede kullanÄ±cÄ± UAC (YÃ¶netici Ä°zni) uyarÄ±sÄ±nÄ± gÃ¶rebilir ve kurulumu tamamlayabilir.
    let status = std::process::Command::new(resource_path)
        .status() // Normal status call, shows window
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("Kurulum kullanÄ±cÄ± tarafÄ±ndan iptal edildi veya baÅŸarÄ±sÄ±z oldu.".into())
    }
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    perform_app_exit(&app);
}

#[tauri::command]
fn update_tray_status(app: tauri::AppHandle, is_connected: bool) -> Result<(), String> {
    let tray = app.tray_by_id("tray").ok_or("Tray icon not found")?;
    
    // Update Tooltip
    let status_text = if is_connected { "XzualDPI - Aktif (Korunuyor)" } else { "XzualDPI - Devre Dışı" };
    let _ = tray.set_tooltip(Some(status_text));

    // Update Tray Menu "Toggle" text
    if let Some(state) = app.try_state::<TrayMenuState>() {
        if let Ok(menu_lock) = state.menu.lock() {
            if let Some(menu) = menu_lock.as_ref() {
                if let Some(item) = menu.get("toggle") {
                    if let Some(menu_item) = item.as_menuitem() {
                        let _ = menu_item.set_text(if is_connected { "Bağlantıyı Kes" } else { "Bağlan" });
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // P0-FIX: Single-instance enforcement â€” aynÄ± anda sadece bir XzualDPI Ã§alÄ±ÅŸabilir
    #[cfg(target_os = "windows")]
    {
        use std::ptr::null_mut;
        use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
        use winapi::um::errhandlingapi::GetLastError;
        use winapi::um::synchapi::CreateMutexW;

        let mutex_name: Vec<u16> = "Global\\XzualDPI_SingleInstance\0".encode_utf16().collect();

        unsafe {
            let handle = CreateMutexW(null_mut(), 0, mutex_name.as_ptr());
            if handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
                eprintln!("[STARTUP] âŒ XzualDPI zaten Ã§alÄ±ÅŸÄ±yor â€” Ã§Ä±kÄ±lÄ±yor");

                use winapi::um::winuser::{
                    FindWindowW, IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE,
                };
                let window_name: Vec<u16> = "XzualDPI\0".encode_utf16().collect();
                let hwnd = FindWindowW(null_mut(), window_name.as_ptr());
                if !hwnd.is_null() {
                    if IsIconic(hwnd) != 0 {
                        ShowWindow(hwnd, SW_RESTORE);
                    }
                    SetForegroundWindow(hwnd);
                }

                // Sessizce Ã§Ä±k (Multi-user ortamÄ±nda diÄŸer kullanÄ±cÄ±larÄ± rahatsÄ±z etme)
                std::process::exit(0);
            }
            // Windows process sonlandÄ±ÄŸÄ±nda mutex handle'Ä±nÄ± otomatik temizler
            let _ = handle;
        }
    }

    tauri::Builder::default()
        .manage(PacServerState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::tray::TrayIconBuilder;
                use tauri::Manager;

                let toggle_i = MenuItem::with_id(app, "toggle", "Bağlan", true, None::<&str>)?;
                let mode_sni = MenuItem::with_id(app, "mode_sni", "SNI Modu", true, None::<&str>)?;
                let mode_chunk = MenuItem::with_id(app, "mode_chunk", "Chunk Modu", true, None::<&str>)?;
                let mode_fake = MenuItem::with_id(app, "mode_fake", "Güçlü Mod", true, None::<&str>)?;

                let mode_menu = Submenu::with_items(app, "Bypass Modu", true, &[&mode_sni, &mode_chunk, &mode_fake])?;

                let show_i = MenuItem::with_id(app, "show", "UygulamayÄ± AÃ§", true, None::<&str>)?;
                let support_i =
                    MenuItem::with_id(app, "support", "Destekle â¤", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", "Ã‡Ä±kÄ±ÅŸ", true, None::<&str>)?;


                let s1 = PredefinedMenuItem::separator(app)?;
                let s2 = PredefinedMenuItem::separator(app)?;

                let menu = Menu::with_items(app, &[
                    &toggle_i, 
                    &mode_menu, 
                    &s1, 
                    &show_i, 
                    &MenuItem::with_id(app, "stats", "İstatistikler", true, None::<&str>)?,
                    &support_i, 
                    &s2, 
                    &quit_i
                ])?;

                // âœ… App menÃ¼ olarak da set et ki discovery (get_menu_item) Ã§alÄ±ÅŸsÄ±n
                // Tray menü handle'ını saklıyoruz (command ile güncelleme yapabilmek için)
                if let Some(state) = app.try_state::<TrayMenuState>() {
                    if let Ok(mut m) = state.menu.lock() {
                        *m = Some(menu.clone());
                    }
                }

                // âœ… Debounce iÃ§in flag
                let is_showing = Arc::new(AtomicBool::new(false));

                let _tray = TrayIconBuilder::with_id("tray")
                    .menu(&menu)
                    .show_menu_on_left_click(false) // âœ… Sol tÄ±kta menÃ¼ aÃ§Ä±lmasÄ±n, sadece saÄŸ tÄ±kta
                    .icon(app.default_window_icon().unwrap().clone())
                    .tooltip("XzualDPI - KapalÄ±")
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus(); // âœ… Pencereyi kapatmadan Ã¶nce onay kutusu iÃ§in Ã¶ne getir!

                                let _ = window.emit("tray_quit", ());
                                let _ = window.close();
                            } else {
                                perform_app_exit(app);
                            }
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "support" => {
                            use tauri_plugin_opener::OpenerExt;
                            app.opener()
                                .open_url(SUPPORT_URL, None::<&str>)
                                .unwrap_or(());
                        }
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray_toggle_connection", ());
                            }
                        }
                        "mode_sni" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray_change_mode", "0");
                            }
                        }
                        "mode_chunk" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray_change_mode", "1");
                            }
                        }
                        "mode_fake" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray_change_mode", "2");
                            }
                        }
                        "stats" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("tray_show_stats", ());
                            }
                        }
                        _ => {}
                    })
                    .on_tray_icon_event({
                        let is_showing = Arc::clone(&is_showing);
                        move |tray, event| {
                            use tauri::tray::{MouseButton, TrayIconEvent};

                            match event {
                                // âœ… Sol tÄ±k: pencereyi Ã¶ne getir
                                TrayIconEvent::Click {
                                    button: MouseButton::Left,
                                    ..
                                } => {
                                    if is_showing.load(Ordering::Relaxed) {
                                        return;
                                    }
                                    is_showing.store(true, Ordering::Relaxed);

                                    let app = tray.app_handle();
                                    if let Some(window) = app.get_webview_window("main") {
                                        let _ = window.unminimize();
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }

                                    let is_showing_clone = Arc::clone(&is_showing);
                                    std::thread::spawn(move || {
                                        std::thread::sleep(std::time::Duration::from_millis(300));
                                        is_showing_clone.store(false, Ordering::Relaxed);
                                    });
                                }
                                // âœ… Ã‡ift tÄ±k: pencereyi Ã¶ne getir
                                TrayIconEvent::DoubleClick { .. } => {
                                    let app = tray.app_handle();
                                    if let Some(window) = app.get_webview_window("main") {
                                        let _ = window.unminimize();
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                                // SaÄŸ tÄ±k: menÃ¼ otomatik aÃ§Ä±lÄ±r
                                _ => {}
                            }
                        }
                    })
                    .build(app)?;

                // LAYER 2: Window close cleanup
                if let Some(window) = app.get_webview_window("main") {
                    let app_handle = app.handle().clone();
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::Destroyed = event {
                            let _ = clear_system_proxy();
                            // âœ… P2-FIX: PAC'i de DIRECT'e geÃ§ir
                            if let Some(pac_state) = app_handle.try_state::<PacServerState>() {
                                if let Ok(mut body) = pac_state.pac_body.lock() {
                                    *body = make_pac_direct_body();
                                }
                                if let Ok(mut cache) = pac_state.pac_cache.lock() {
                                    cache.body_hash = 0;
                                    cache.pac_response.clear();
                                }
                            }
                        }
                    });
                }
            }
            Ok(())
        })
        .manage(TrayMenuState { menu: Mutex::new(None) })
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // notification plugin zaten yukarÄ±da kayÄ±tlÄ±, tekrar ekleme
        .invoke_handler(tauri::generate_handler![
            clear_system_proxy,
            set_system_proxy,
            update_tray_tooltip,
            check_admin,
            check_port_open,
            get_sidecar_config,
            start_pac_server,
            stop_pac_server,
            kill_zombie_sidecar,
            check_dns_latency,
            save_sidecar_pid,
            startup_proxy_cleanup,
            check_driver,
            install_driver,
            update_tray_status,
            quit_app
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // LAYER 3: App exit cleanup (fallback)
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let _ = clear_system_proxy();
                if let Some(state) = app_handle.try_state::<PacServerState>() {
                    // Grace period'u kÄ±salt â€” App.jsx zaten 1.5s (ÅŸimdi 0.5s) bekledi
                    // DIRECT'e geÃ§ ama uzun bekleme
                    if let Ok(mut body) = state.pac_body.lock() {
                        *body = make_pac_direct_body();
                    }
                    if let Ok(mut cache) = state.pac_cache.lock() {
                        cache.body_hash = 0;
                        cache.pac_response.clear();
                    }
                    // 500ms yeterli â€” cihazlar genelde 200ms iÃ§inde PAC'i Ã§eker
                    std::thread::sleep(Duration::from_millis(500));
                    state.shutdown.store(true, Ordering::Relaxed);
                    if let Ok(mut guard) = state.join_handle.lock() {
                        let _ = guard.take();
                    }
                    #[cfg(target_os = "windows")]
                    manage_firewall_rules(false, 0, 0);
                }
            }
        });
}

