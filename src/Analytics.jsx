import React, { useMemo } from 'react';
import { motion } from 'framer-motion';
import { Activity, Globe, Zap, BarChart2, ShieldCheck, Clock } from 'lucide-react';

const Analytics = ({ stats, t }) => {
  const totalBypassed = stats.totalBypassed || 0;
  const sessionBypassed = stats.sessionBypassed || 0;
  const uptime = stats.uptime || "00:00:00";
  
  // Simulated data for graph
  const chartData = useMemo(() => {
    return Array.from({ length: 12 }, (_, i) => ({
      val: Math.floor(Math.random() * 60) + 20,
      label: `${i * 2}:00`
    }));
  }, []);

  return (
    <motion.div 
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className="analytics-container"
      style={{ padding: '1.5rem', overflowY: 'auto', flex: 1, paddingBottom: '80px' }}
    >
      <div className="v2-section-title" style={{ marginBottom: '1.5rem', display: 'flex', alignItems: 'center', gap: '8px' }}>
        <BarChart2 size={18} color="var(--accent-main)" />
        {t.navAnalytics || "Ağ Analizi"}
      </div>

      {/* Grid Stats */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', marginBottom: '1.5rem' }}>
        <div className="v2-card" style={{ padding: '16px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-tertiary)', fontSize: '0.75rem', fontWeight: 600 }}>
            <Zap size={14} color="var(--accent-yellow)" />
            {t.statsTotalBypass || "TOPLAM BYPASS"}
          </div>
          <div style={{ fontSize: '1.5rem', fontWeight: 800, color: 'var(--text-primary)' }}>
            {totalBypassed.toLocaleString()}
          </div>
          <div style={{ fontSize: '0.65rem', color: 'var(--accent-green)' }}>+ {sessionBypassed} bu oturumda</div>
        </div>

        <div className="v2-card" style={{ padding: '16px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-tertiary)', fontSize: '0.75rem', fontWeight: 600 }}>
            <Clock size={14} color="var(--accent-blue)" />
            {t.statsUptime || "ÇALIŞMA SÜRESİ"}
          </div>
          <div style={{ fontSize: '1.5rem', fontWeight: 800, color: 'var(--text-primary)' }}>
            {uptime}
          </div>
          <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)' }}>Kesintisiz Koruma</div>
        </div>
      </div>

      {/* Activity Chart */}
      <div className="v2-card" style={{ padding: '1.5rem', marginBottom: '1.5rem' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
          <h3 style={{ fontSize: '0.9rem', fontWeight: 700, margin: 0 }}>{t.statsActivity || "24 Saatlik Aktivite"}</h3>
          <ShieldCheck size={16} color="var(--accent-green)" />
        </div>
        
        <div style={{ height: '120px', display: 'flex', alignItems: 'flex-end', gap: '6px', justifyContent: 'space-between' }}>
          {chartData.map((d, i) => (
            <div key={i} style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '8px' }}>
              <motion.div 
                initial={{ height: 0 }}
                animate={{ height: `${d.val}%` }}
                style={{ 
                  width: '100%', 
                  background: i === 11 ? 'var(--accent-main)' : 'var(--bg-surface)', 
                  borderRadius: '4px',
                  minHeight: '4px'
                }} 
              />
              <span style={{ fontSize: '0.6rem', color: 'var(--text-tertiary)', fontWeight: 600 }}>{d.label}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Security Score */}
      <div className="v2-card" style={{ padding: '1.25rem', display: 'flex', alignItems: 'center', gap: '16px', background: 'linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, transparent 100%)' }}>
        <div style={{ position: 'relative', width: '48px', height: '48px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <svg width="48" height="48" viewBox="0 0 48 48">
            <circle cx="24" cy="24" r="20" fill="none" stroke="rgba(16, 185, 129, 0.1)" strokeWidth="4" />
            <circle cx="24" cy="24" r="20" fill="none" stroke="var(--accent-green)" strokeWidth="4" strokeDasharray="125.6" strokeDashoffset="25" />
          </svg>
          <span style={{ position: 'absolute', fontSize: '0.75rem', fontWeight: 800 }}>92</span>
        </div>
        <div style={{ flex: 1 }}>
          <h4 style={{ fontSize: '0.85rem', fontWeight: 700, margin: 0, marginBottom: '2px' }}>Ağ Güvenlik Skoru</h4>
          <p style={{ fontSize: '0.7rem', color: 'var(--text-secondary)', margin: 0 }}>DPI engellemeleri başarıyla atlatılıyor.</p>
        </div>
      </div>

    </motion.div>
  );
};

export default Analytics;
