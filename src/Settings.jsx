import React, { useState, useEffect, useRef, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  ChevronLeft, ChevronDown, Globe, Power, Zap, RotateCw, Activity, Pin,
  Youtube, Coffee, AlertTriangle, Check, Wrench, Languages, Bell, Shield, Settings as SettingsIcon,
  Search, Cpu, Info
} from 'lucide-react';
import { ask, message } from '@tauri-apps/plugin-dialog';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { Command } from '@tauri-apps/plugin-shell';
import { openUrl } from '@tauri-apps/plugin-opener';
import { invoke } from '@tauri-apps/api/core';
import { getTranslations, SUPPORTED_LANGUAGES } from './i18n';
import { URLS } from './constants';
import { ISP_PROFILES, CHUNK_SIZES, DEFAULT_CHUNKS, BYPASS_PROFILES } from './profiles';
import './App.css';

const Toggle = ({ checked, onChange }) => (
  <div
    className={`v2-toggle ${checked ? 'active' : ''}`}
    onClick={(e) => {
      e.stopPropagation();
      onChange(!checked);
    }}
  >
    <div className="v2-toggle-thumb" />
  </div>
);

const Settings = ({ onBack, config, updateConfig, dnsLatencies, setDnsLatencies }) => {
  const [activeTab, setActiveTab] = useState('general');
  const scrollRef = useRef(null);

  const [expandedISP, setExpandedISP] = useState(null);
  const [driverInstalled, setDriverInstalled] = useState(false);
  const [needsRestart, setNeedsRestart] = useState(false);
  const [showNpcapDetails, setShowNpcapDetails] = useState(false);

  useEffect(() => {
    invoke('check_driver').then(setDriverInstalled);
  }, []);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = 0;
    }
  }, [activeTab]);

  const latencies = dnsLatencies || {};
  const [isChecking, setIsChecking] = useState(false);
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [sortedProviders, setSortedProviders] = useState([]);
  const [fixStatus, setFixStatus] = useState('idle');
  const [speedTest, setSpeedTest] = useState({ status: 'idle', ping: null, speed: null });

  const lang = config.language || 'tr';
  const t = getTranslations(lang);

  const DNS_PROVIDERS = useMemo(() => [
    { id: 'system', name: t.dnsSystemDefault, desc: t.dnsSystemDefaultDesc, ip: null },
    { id: 'cloudflare', name: 'Cloudflare', desc: t.dnsCfDesc, ip: '1.1.1.1' },
    { id: 'adguard', name: 'AdGuard', desc: t.dnsAdguardDesc, ip: '94.140.14.14' },
    { id: 'google', name: 'Google', desc: t.dnsGoogleDesc, ip: '8.8.8.8' },
    { id: 'quad9', name: 'Quad9', desc: t.dnsQuad9Desc, ip: '9.9.9.9' },
    { id: 'opendns', name: 'OpenDNS', desc: t.dnsOpenDnsDesc, ip: '208.67.222.222' }
  ], [t]);

  useEffect(() => {
    if (Object.keys(latencies).length > 0) {
      const systemDns = DNS_PROVIDERS.find(p => p.id === 'system');
      const otherDns = DNS_PROVIDERS.filter(p => p.id !== 'system')
        .sort((a, b) => (latencies[a.id] || 999) - (latencies[b.id] || 999));
      setSortedProviders(systemDns ? [systemDns, ...otherDns] : otherDns);
    } else {
      setSortedProviders(DNS_PROVIDERS);
    }
  }, [lang, latencies, DNS_PROVIDERS]);

  useEffect(() => {
    checkAutostart();
  }, []);

  const checkAutostart = async () => {
    try {
      const active = await isEnabled();
      setAutostartEnabled(active);
    } catch (e) {
      console.error('Autostart check failed:', e);
    }
  };

  const toggleAutostart = async (val) => {
    try {
      if (val) await enable(); else await disable();
      setAutostartEnabled(val);
      updateConfig('autoStart', val);
    } catch (e) {
      console.error('Autostart toggle failed:', e);
    }
  };

  const checkAllLatencies = async (forceSelectBest = false) => {
    setIsChecking(true);
    const newLatencies = {};
    const pingableProviders = DNS_PROVIDERS.filter(p => p.ip !== null);

    const results = await Promise.allSettled(
      pingableProviders.map(async (provider) => {
        try {
          const latency = await invoke('check_dns_latency', { dnsIp: provider.ip });
          return { id: provider.id, latency };
        } catch (e) {
          return { id: provider.id, latency: 999 };
        }
      })
    );

    results.forEach(result => {
      if (result.status === 'fulfilled') {
        newLatencies[result.value.id] = result.value.latency;
      }
    });

    setDnsLatencies(newLatencies);

    const otherDns = DNS_PROVIDERS.filter(p => p.id !== 'system').sort((a, b) =>
      (newLatencies[a.id] || 999) - (newLatencies[b.id] || 999)
    );

    if (forceSelectBest || config.dnsMode === 'auto') {
      const bestDns = otherDns[0];
      if (bestDns) updateConfig('selectedDns', bestDns.id);
    }
    setIsChecking(false);
  };

  const runSpeedTest = async () => {
    setSpeedTest({ status: 'running', ping: null, speed: null });
    try {
      const startPing = performance.now();
      await fetch('https://www.google.com/favicon.ico', { mode: 'no-cors', cache: 'no-store' });
      const ping = Math.round(performance.now() - startPing);
      const startSpeed = performance.now();
      const response = await fetch('https://cdnjs.cloudflare.com/ajax/libs/three.js/0.160.0/three.module.min.js', { cache: 'no-store' });
      const blob = await response.blob();
      const duration = (performance.now() - startSpeed) / 1000;
      const sizeMb = (blob.size * 8) / (1024 * 1024);
      const mbps = (sizeMb / duration).toFixed(2);
      setSpeedTest({ status: 'done', ping, speed: mbps });
    } catch (err) {
      setSpeedTest({ status: 'done', ping: '?', speed: '?' });
    }
  };

  const handleFixInternet = async () => {
    if (fixStatus === 'fixing') return;
    setFixStatus('fixing');
    try {
      await invoke('clear_system_proxy');
      window.dispatchEvent(new CustomEvent('Xzual-force-disconnect', { detail: { reason: 'manual-fix' } }));
      setFixStatus('fixed');
      setTimeout(() => setFixStatus('idle'), 2000);
    } catch (e) {
      setFixStatus('error');
      setTimeout(() => setFixStatus('idle'), 2000);
    }
  };

  const renderGeneralTab = () => (
    <motion.div
      key="general-tab"
      initial={{ opacity: 0, x: -15 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 15 }}
      transition={{ duration: 0.2 }}
      style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}
    >
      {/* 1. DİL SEÇİMİ */}
      <div className="v2-section">
        <div className="v2-section-title">{t.language}</div>
        <div className="v2-card">
          {SUPPORTED_LANGUAGES.map((l, index) => (
            <React.Fragment key={l.code}>
              <div
                className={`v2-item hover-effect ${lang === l.code ? 'v2-selected' : ''}`}
                style={{
                  background: lang === l.code ? 'rgba(234, 179, 8, 0.1)' : 'transparent',
                  padding: '12px 16px',
                  cursor: 'pointer'
                }}
                onClick={() => updateConfig('language', l.code)}
              >
                <div className="v2-icon" style={{ background: lang === l.code ? 'var(--accent-main-subtle)' : 'rgba(255, 255, 255, 0.05)', color: lang === l.code ? 'var(--accent-main)' : '#94a3b8' }}>
                  <span style={{ fontSize: '1.2rem' }}>{l.flag}</span>
                </div>
                <div className="v2-item-text">
                  <h3 style={{ color: lang === l.code ? 'var(--accent-main)' : '#f8fafc' }}>{l.name}</h3>
                  <p style={{ fontSize: '0.75rem' }}>{l.code.toUpperCase()}</p>
                </div>
                <div className={`v2-radio ${lang === l.code ? 'on' : ''}`}>
                  {lang === l.code && <div className="v2-radio-dot" />}
                </div>
              </div>
              {index < SUPPORTED_LANGUAGES.length - 1 && <div className="v2-divider" />}
            </React.Fragment>
          ))}
        </div>
      </div>

      {/* 2. OTOMASYON VE BAŞLANGIÇ */}
      <div className="v2-section">
        <div className="v2-section-title">{t.sectionGeneral}</div>
        <div className="v2-card">
          <div className="v2-item">
            <div className="v2-icon yellow"><Zap size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.autoConnect}</h3>
              <p>{t.autoConnectDesc}</p>
            </div>
            <Toggle checked={config.autoConnect} onChange={(v) => updateConfig('autoConnect', v)} />
          </div>
          <div className="v2-divider" />
          <div className="v2-item">
            <div className="v2-icon green"><Power size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.autoStart}</h3>
              <p>{t.autoStartDesc}</p>
            </div>
            <Toggle checked={autostartEnabled} onChange={toggleAutostart} />
          </div>
          <div className="v2-divider" />
          <div className="v2-item">
            <div className="v2-icon blue"><Pin size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.minimizeToTray}</h3>
              <p>{t.minimizeToTrayDesc}</p>
            </div>
            <Toggle checked={config.minimizeToTray} onChange={(v) => updateConfig('minimizeToTray', v)} />
          </div>
        </div>
      </div>
    </motion.div>
  );

  const renderNetworkTab = () => (
    <motion.div
      key="network-tab"
      initial={{ opacity: 0, x: -15 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 15 }}
      transition={{ duration: 0.2 }}
      style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}
    >
      {/* 1. BAĞLANTI YÖNTEMLERİ (DPI MODLARI) */}
      <div className="v2-section">
        <div className="v2-section-title">{t.sectionBypass}</div>
        <div className="v2-card">
          {/* Turbo Mod */}
          <div
            className={`v2-item hover-effect ${config.dpiMethod === '0' ? 'v2-selected' : ''}`}
            onClick={() => updateConfig({ dpiMethod: '0', httpsChunkSize: 4, selectedIspProfile: 'custom' })}
          >
            <div className="v2-icon yellow"><Activity size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.modeTurboName}</h3>
              <p>{t.modeTurboDesc}</p>
            </div>
            <div className={`v2-radio ${config.dpiMethod === '0' ? 'on' : ''}`}>
              {config.dpiMethod === '0' && <div className="v2-radio-dot" />}
            </div>
          </div>
          <div className="v2-divider" />
          {/* Balanced Mod */}
          <div
            className={`v2-item hover-effect ${config.dpiMethod === '1' ? 'v2-selected' : ''}`}
            onClick={() => updateConfig({ dpiMethod: '1', httpsChunkSize: 2, selectedIspProfile: 'custom' })}
          >
            <div className="v2-icon green"><Zap size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.modeBalancedName}</h3>
              <p>{t.modeBalancedDesc}</p>
            </div>
            <div className={`v2-radio ${config.dpiMethod === '1' ? 'on' : ''}`}>
              {config.dpiMethod === '1' && <div className="v2-radio-dot" />}
            </div>
          </div>
          <div className="v2-divider" />
          {/* Strong Mod */}
          <div
            className={`v2-item hover-effect ${config.dpiMethod === '2' ? 'v2-selected' : ''}`}
            onClick={() => updateConfig({ dpiMethod: '2', httpsChunkSize: 1, selectedIspProfile: 'custom' })}
          >
            <div className="v2-icon blue"><Shield size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.modeStrongName}</h3>
              <p>{t.modeStrongDesc}</p>
            </div>
            <div className={`v2-radio ${config.dpiMethod === '2' ? 'on' : ''}`}>
              {config.dpiMethod === '2' && <div className="v2-radio-dot" />}
            </div>
          </div>

          {/* Npcap Settings (Conditional) */}
          {config.dpiMethod === '2' && (
            <div style={{ padding: '0 16px 12px' }}>
              <div className="v2-divider" style={{ margin: '8px 0' }} />
              {driverInstalled ? (
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div className="v2-item-text">
                    <h4 style={{ color: '#d8b4fe', fontSize: '0.85rem', margin: 0 }}>{t.advancedFeaturesToggle}</h4>
                    <p style={{ fontSize: '0.7rem', margin: 0 }}>{t.advancedFeaturesToggleDesc}</p>
                  </div>
                  <Toggle checked={config.advancedBypass !== false} onChange={(v) => updateConfig('advancedBypass', v)} />
                </div>
              ) : (
                <button
                  className="v2-btn-minimal"
                  onClick={() => setShowNpcapDetails(!showNpcapDetails)}
                >
                  <Info size={14} /> {t.advancedNpcapHint}
                </button>
              )}
            </div>
          )}
        </div>
      </div>

      {/* 2. DNS AYARLARI */}
      <div className="v2-section">
        <div className="v2-section-title">{t.sectionDns}</div>
        <div className="v2-card">
          <div className="v2-item">
            <div className="v2-item-text">
              <h3>{t.dnsSystemDefault}</h3>
              <p>{t.dnsSystemDefaultDesc}</p>
            </div>
            <Toggle
              checked={config.selectedDns === 'system'}
              onChange={(v) => updateConfig({ selectedDns: v ? 'system' : 'cloudflare', dnsMode: 'manual' })}
            />
          </div>
          {config.selectedDns !== 'system' && (
            <>
              <div className="v2-divider" />
              <div className="v2-item">
                <div className="v2-item-text">
                  <h3>{t.dnsAutoSelect}</h3>
                  <p>{t.dnsAutoSelectDesc}</p>
                </div>
                <Toggle
                  checked={config.dnsMode === 'auto'}
                  onChange={(v) => {
                    updateConfig('dnsMode', v ? 'auto' : 'manual');
                    if (v) checkAllLatencies(true);
                  }}
                />
              </div>

              <div style={{ padding: '0 16px 16px' }}>
                <button
                  className="v2-action-btn"
                  onClick={() => checkAllLatencies()}
                  disabled={isChecking}
                  style={{ marginBottom: '12px' }}
                >
                  {isChecking ? <RotateCw size={16} className="spinning" /> : <Activity size={16} />}
                  {isChecking ? t.dnsChecking : t.dnsCheckSpeed}
                </button>

                {/* DNS Liste Görünümü */}
                <div className="v2-dns-list">
                  {sortedProviders.filter(p => p.id !== 'system').map((provider, idx) => {
                    const isSelected = config.selectedDns === provider.id;
                    const latency = latencies[provider.id];

                    return (
                      <React.Fragment key={provider.id}>
                        <div
                          className={`v2-dns-item ${isSelected ? 'active' : ''} ${config.dnsMode === 'auto' ? 'readonly' : ''}`}
                          onClick={() => config.dnsMode !== 'auto' && updateConfig('selectedDns', provider.id)}
                        >
                          <div className="v2-dns-info">
                            <div className="v2-dns-name">
                              {provider.name}
                              {isSelected && <Check size={14} style={{ marginLeft: '6px', color: 'var(--accent-main)' }} />}
                            </div>
                            <div className="v2-dns-desc">{provider.desc}</div>
                          </div>
                          <div className={`v2-dns-latency ${latency > 200 ? 'high' : latency > 100 ? 'mid' : 'low'}`}>
                            {latency ? `${latency}ms` : '--'}
                          </div>
                        </div>
                        {idx < sortedProviders.filter(p => p.id !== 'system').length - 1 && <div className="v2-divider-thin" />}
                      </React.Fragment>
                    );
                  })}
                </div>
              </div>
            </>
          )}
        </div>
      </div>

      {/* 3. EKSTRA AĞ AYARLARI */}
      <div className="v2-section">
        <div className="v2-section-title">{t.sectionExtraNetwork}</div>
        <div className="v2-card">
          <div className="v2-item">
            <div className="v2-icon purple"><Globe size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.lanSharing}</h3>
              <p>{t.lanSharingDesc}</p>
            </div>
            <Toggle checked={config.lanSharing} onChange={(v) => updateConfig('lanSharing', v)} />
          </div>
          <div className="v2-divider" />
          <div className="v2-item">
            <div className="v2-icon red"><Activity size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.ipv4ForceTitle}</h3>
              <p>{t.ipv4ForceDesc}</p>
            </div>
            <Toggle checked={config.ipv4Only !== false} onChange={(v) => updateConfig('ipv4Only', v)} />
          </div>
          <div className="v2-divider" />
          <div className="v2-item">
            <div className="v2-icon blue"><Youtube size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.winHttpForceTitle}</h3>
              <p>{t.winHttpForceDesc}</p>
            </div>
            <Toggle checked={config.enableWinhttp !== false} onChange={(v) => updateConfig('enableWinhttp', v)} />
          </div>
        </div>
      </div>

      {/* 4. UYGULAMA FİLTRELEME (APPS) */}
      <div className="v2-section">
        <div className="v2-section-title">Uygulama Filtreleme</div>
        <div className="v2-card">
          <div style={{ padding: '16px' }}>
            <p style={{ fontSize: '0.75rem', color: 'var(--text-tertiary)', marginBottom: '12px' }}>
              Aşağıdaki servisleri seçerek sadece bu servislerin Xzual üzerinden geçmesini sağlayabilirsiniz.
            </p>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {BYPASS_PROFILES.map(profile => {
                const isEnabled = (config.whitelist || "").includes(profile.domains);
                return (
                  <div key={profile.id} className="v2-item" style={{ padding: '8px 0' }}>
                    <div className="v2-item-text">
                      <h3 style={{ fontSize: '0.85rem' }}>{profile.name}</h3>
                    </div>
                    <Toggle
                      checked={isEnabled}
                      onChange={(v) => {
                        let currentList = config.whitelist || "";
                        if (v) {
                          if (!currentList.includes(profile.domains)) {
                            currentList = currentList ? `${currentList},${profile.domains}` : profile.domains;
                          }
                        } else {
                          currentList = currentList.replace(profile.domains, "").replace(",,", ",").replace(/^,|,$/, "");
                        }
                        updateConfig('whitelist', currentList);
                      }}
                    />
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>


    </motion.div>
  );

  const renderNotificationsTab = () => (
    <motion.div
      key="notifications-tab"
      initial={{ opacity: 0, x: -15 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 15 }}
      transition={{ duration: 0.2 }}
      style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}
    >
      <div className="v2-section">
        <div className="v2-section-title">{t.sectionNotifications}</div>
        <div className="v2-card">
          <div className="v2-item">
            <div className="v2-icon blue"><Bell size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.notifications}</h3>
              <p>{t.notificationsDesc}</p>
            </div>
            <Toggle checked={config.notifications !== false} onChange={(v) => updateConfig('notifications', v)} />
          </div>
        </div>
      </div>
    </motion.div>
  );

  const renderSystemTab = () => (
    <motion.div
      key="system-tab"
      initial={{ opacity: 0, x: -15 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 15 }}
      transition={{ duration: 0.2 }}
      style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}
    >
      {/* 1. SORUN GİDERME */}
      <div className="v2-section">
        <div className="v2-section-title">{t.sectionTroubleshoot}</div>
        <div className="v2-card" style={{ background: fixStatus === 'fixing' ? '#b45309' : fixStatus === 'fixed' ? '#10b981' : fixStatus === 'error' ? '#ef4444' : 'rgba(255,255,255,0.03)', border: 'none' }}>
          <div className="v2-item hover-effect" onClick={handleFixInternet} style={{ cursor: fixStatus === 'idle' ? 'pointer' : 'default' }}>
            <div className="v2-icon" style={{ background: '#fff', color: '#000' }}>
              <Wrench size={20} className={fixStatus === 'fixing' ? 'spinning-slow' : ''} />
            </div>
            <div className="v2-item-text">
              <h3 style={{ color: '#fff' }}>{fixStatus === 'fixing' ? t.fixRepairing : fixStatus === 'fixed' ? t.fixDone : fixStatus === 'error' ? t.fixError : t.fixInternet}</h3>
              <p style={{ color: 'rgba(255,255,255,0.7)' }}>{fixStatus === 'fixing' ? t.fixRepairingDesc : fixStatus === 'fixed' ? t.fixDoneDesc : fixStatus === 'error' ? t.fixErrorDesc : t.fixInternetDesc}</p>
            </div>
          </div>
        </div>
      </div>

      {/* 2. HIZ TESTİ */}
      <div className="v2-section">
        <div className="v2-section-title">{t.speedTestTitle}</div>
        <div className="v2-card">
          <div className="v2-item" style={{ flexDirection: 'column', alignItems: 'flex-start', gap: '1rem', padding: '1.25rem' }}>
            <div style={{ display: 'flex', gap: '1.5rem', width: '100%' }}>
              <div style={{ flex: 1 }}>
                <p style={{ fontSize: '0.7rem', color: '#94a3b8', marginBottom: '4px' }}>{t.speedTestResultPing}</p>
                <div style={{ fontSize: '1.2rem', fontWeight: 700, color: 'var(--accent-main)' }}>{speedTest.ping ? `${speedTest.ping} ms` : '--'}</div>
              </div>
              <div style={{ flex: 1 }}>
                <p style={{ fontSize: '0.7rem', color: '#94a3b8', marginBottom: '4px' }}>{t.speedTestResultSpeed}</p>
                <div style={{ fontSize: '1.2rem', fontWeight: 700, color: 'var(--accent-main)' }}>{speedTest.speed ? `${speedTest.speed} Mbps` : '--'}</div>
              </div>
            </div>
            <button
              className="v2-action-btn"
              onClick={runSpeedTest}
              disabled={speedTest.status === 'running'}
              style={{ background: 'var(--accent-main)', color: '#000', border: 'none' }}
            >
              {speedTest.status === 'running' ? <RotateCw size={16} className="spinning" /> : <Zap size={16} />}
              {speedTest.status === 'running' ? t.speedTestRunning : t.btnRunSpeedTest}
            </button>
          </div>
        </div>
      </div>

      {/* 3. GELİŞTİRİCİ VE BİLGİ */}
      <div className="v2-section">
        <div className="v2-section-title">{t.sectionDev}</div>
        <div className="v2-card">
          <div className="v2-item">
            <div className="v2-icon"><Cpu size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.appName} v1.0.0</h3>
              <p>Built with Tauri & Rust</p>
            </div>
            <button
              className="v2-action-btn"
              style={{ width: 'auto', padding: '6px 12px', fontSize: '0.75rem' }}
              onClick={async () => {
                setFixStatus('checking');
                try {
                  // GitHub üzerinden en güncel sürümü kontrol et
                  const response = await fetch('https://raw.githubusercontent.com/Xzual/XzualDPI-Windows/main/package.json');
                  if (!response.ok) throw new Error("Dosya bulunamadı");
                  
                  const data = await response.json();
                  const latestVersion = data.version;
                  const currentVersion = "1.0.0";

                  if (latestVersion !== currentVersion) {
                    const confirmUpdate = await ask(`Yeni bir sürüm mevcut: v${latestVersion}\n\nGüncelleme sayfasını açmak ister misiniz?`, {
                      title: 'XzualDPI Güncelleme',
                      kind: 'info',
                      okLabel: 'Evet',
                      cancelLabel: 'Daha Sonra'
                    });
                    
                    if (confirmUpdate) {
                      await openUrl("https://github.com/Xzual/XzualDPI-Windows-main/releases");
                    }
                  } else {
                    await message("XzualDPI şu an en güncel sürümde (v1.0.0).", { title: 'XzualDPI', kind: 'info' });
                  }
                } catch (e) {
                  console.error("Güncelleme hatası:", e);
                  await message("Güncelleme kontrolü başarısız oldu. Lütfen internetinizi veya depo adresini kontrol edin.", { title: 'Hata', kind: 'error' });
                } finally {
                  setFixStatus('idle');
                }
              }}
            >
              {fixStatus === 'checking' ? <RotateCw size={14} className="spinning" /> : <RotateCw size={14} />}
              {fixStatus === 'checking' ? 'Kontrol ediliyor...' : 'Güncelleme Kontrol'}
            </button>
          </div>
          <div className="v2-divider" />
          <div className="v2-item">
            <div className="v2-icon gray"><Info size={20} /></div>
            <div className="v2-item-text">
              <h3>{t.sectionNotice}</h3>
              <p style={{ fontSize: '0.7rem', lineHeight: '1.4' }}>{t.noticeDesc}</p>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );

  return (
    <div className="v2-settings-overlay">
      <div className="v2-settings-header">
        <button className="v2-back-btn" onClick={onBack}><ChevronLeft size={28} /></button>
        <h1>{t.settingsTitle}</h1>
      </div>

      <div className="v2-settings-content" ref={scrollRef}>
        <AnimatePresence mode="wait">
          {activeTab === 'general' && renderGeneralTab()}
          {activeTab === 'network' && renderNetworkTab()}
          {activeTab === 'notifications' && renderNotificationsTab()}
          {activeTab === 'system' && renderSystemTab()}
        </AnimatePresence>
      </div>

      <nav className="bottom-nav">
        <button className={`nav-btn ${activeTab === 'general' ? 'active' : ''}`} onClick={() => setActiveTab('general')}>
          <SettingsIcon size={20} />
          <span>{t.tabGeneral}</span>
        </button>
        <button className={`nav-btn ${activeTab === 'network' ? 'active' : ''}`} onClick={() => setActiveTab('network')}>
          <Globe size={20} />
          <span>{t.tabNetwork}</span>
        </button>
        <button className={`nav-btn ${activeTab === 'notifications' ? 'active' : ''}`} onClick={() => setActiveTab('notifications')}>
          <Bell size={20} />
          <span>{t.tabNotification}</span>
        </button>
        <button className={`nav-btn ${activeTab === 'system' ? 'active' : ''}`} onClick={() => setActiveTab('system')}>
          <Shield size={20} />
          <span>{t.tabSystem}</span>
        </button>
      </nav>
    </div>
  );
};

export default Settings;
