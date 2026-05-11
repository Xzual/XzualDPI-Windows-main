// ============================================================
// XzualDPI — Merkezi Sabitler
// Tüm URL'ler, DNS ayarları ve app sabitleri burada toplanır.
// ============================================================


// ===== Dış Bağlantılar =====
export const URLS = {
  youtube: "https://www.youtube.com/@Ardayorulmazel",
  github: "https://github.com/Xzual",
};

// ===== DNS Ayarları =====
export const DNS_MAP = {
  system: null,
  cloudflare: "1.1.1.1",
  adguard: "94.140.14.14",
  google: "8.8.8.8",
  quad9: "9.9.9.9",
  opendns: "208.67.222.222",
};
export const DOH_MAP = {
  cloudflare: "https://1.1.1.1/dns-query",
  google: "https://8.8.8.8/dns-query",
  adguard: "https://94.140.14.14/dns-query",
  quad9: "https://dns.quad9.net/dns-query",
  opendns: "https://208.67.222.222/dns-query",
};

// ===== Uygulama Sabitleri =====
export const APP = {
  name: "XzualDPI",
  version: "1.0.0",
  designWidth: 380,
  designHeight: 700,
  maxLogs: 100,
  maxPortRetries: 20,
  maxReconnectAttempts: 5,
  portCheckMaxAttempts: 15,
};

// ===== Retry Gecikmeleri (ms) =====
export const RETRY_DELAYS = [2500, 3000, 6000, 12000, 20000];

// ===== DPI Mod Timeout'ları (ms) =====
export const DPI_TIMEOUTS = {
  "0": 3000, // Turbo — hafif DPI
  "1": 5000, // Dengeli — çoğu ISP
  "2": 8000, // Güçlü — agresif ISP (Kablonet vb.)
};
