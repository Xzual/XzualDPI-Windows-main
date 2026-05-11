import React, { useState } from 'react';
import { supabase } from './supabaseClient';
import { Shield, Mail, Lock, Loader2 } from 'lucide-react';
import { motion } from 'framer-motion';

export default function Auth({ onAuthSuccess }) {
  const [loading, setLoading] = useState(false);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [isLogin, setIsLogin] = useState(true);
  const [errorMsg, setErrorMsg] = useState(null);
  const [successMsg, setSuccessMsg] = useState(null);

  const handleAuth = async (e) => {
    e.preventDefault();
    setLoading(true);
    setErrorMsg(null);
    setSuccessMsg(null);

    try {
      if (isLogin) {
        const { error } = await supabase.auth.signInWithPassword({
          email,
          password,
        });
        if (error) throw error;
        onAuthSuccess();
      } else {
        const { error } = await supabase.auth.signUp({
          email,
          password,
        });
        if (error) throw error;
        setSuccessMsg('Kayıt başarılı! Lütfen giriş yapın.');
        setIsLogin(true);
      }
    } catch (error) {
      setErrorMsg(error.error_description || error.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{
      width: '100%', height: '100vh', display: 'flex', flexDirection: 'column',
      justifyContent: 'center', alignItems: 'center', padding: '2rem',
      background: 'linear-gradient(to bottom, #0f172a, #020617)',
      color: 'white', position: 'absolute', zIndex: 9999, top: 0, left: 0
    }}>
      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        style={{
          width: '100%', maxWidth: '320px', background: 'rgba(255,255,255,0.03)',
          border: '1px solid rgba(255,255,255,0.1)', borderRadius: '16px',
          padding: '2rem', backdropFilter: 'blur(10px)',
          display: 'flex', flexDirection: 'column', gap: '1.5rem',
          boxShadow: '0 8px 32px rgba(0,0,0,0.5)'
        }}
      >
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.5rem' }}>
          <Shield size={48} color="#eab308" />
          <h2 style={{ margin: 0, fontSize: '1.25rem', fontWeight: 800, letterSpacing: '1px' }}>
            {isLogin ? 'XzualDPI Giriş' : 'Yeni Kayıt'}
          </h2>
          <p style={{ margin: 0, fontSize: '0.8rem', color: '#94a3b8', textAlign: 'center' }}>
            Devam etmek için {isLogin ? 'giriş yapın' : 'kayıt olun'}
          </p>
        </div>

        {errorMsg && (
          <div style={{ padding: '10px', background: 'rgba(239, 68, 68, 0.1)', border: '1px solid rgba(239, 68, 68, 0.3)', borderRadius: '8px', color: '#f87171', fontSize: '0.8rem', textAlign: 'center' }}>
            {errorMsg}
          </div>
        )}
        
        {successMsg && (
          <div style={{ padding: '10px', background: 'rgba(34, 197, 94, 0.1)', border: '1px solid rgba(34, 197, 94, 0.3)', borderRadius: '8px', color: '#4ade80', fontSize: '0.8rem', textAlign: 'center' }}>
            {successMsg}
          </div>
        )}

        <form onSubmit={handleAuth} style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          <div style={{ position: 'relative' }}>
            <Mail size={16} color="#94a3b8" style={{ position: 'absolute', left: '12px', top: '12px' }} />
            <input 
              type="email" 
              placeholder="E-posta Adresi" 
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
              style={{
                width: '100%', padding: '10px 10px 10px 36px',
                background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)',
                borderRadius: '8px', color: 'white', fontSize: '0.9rem', outline: 'none'
              }}
            />
          </div>
          
          <div style={{ position: 'relative' }}>
            <Lock size={16} color="#94a3b8" style={{ position: 'absolute', left: '12px', top: '12px' }} />
            <input 
              type="password" 
              placeholder="Şifre" 
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              minLength={6}
              style={{
                width: '100%', padding: '10px 10px 10px 36px',
                background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)',
                borderRadius: '8px', color: 'white', fontSize: '0.9rem', outline: 'none'
              }}
            />
          </div>

          <button 
            type="submit" 
            disabled={loading}
            style={{
              marginTop: '0.5rem', width: '100%', padding: '12px',
              background: '#eab308', color: '#000', border: 'none',
              borderRadius: '8px', fontWeight: 800, fontSize: '0.9rem',
              cursor: loading ? 'not-allowed' : 'pointer',
              display: 'flex', justifyContent: 'center', alignItems: 'center', gap: '8px',
              opacity: loading ? 0.7 : 1
            }}
          >
            {loading && <Loader2 size={16} style={{ animation: 'spin 1s linear infinite' }} />}
            {isLogin ? 'GİRİŞ YAP' : 'KAYIT OL'}
          </button>
        </form>

        <div style={{ textAlign: 'center', marginTop: '0.5rem' }}>
          <button 
            type="button" 
            onClick={() => { setIsLogin(!isLogin); setErrorMsg(null); setSuccessMsg(null); }}
            style={{
              background: 'none', border: 'none', color: '#94a3b8',
              fontSize: '0.8rem', cursor: 'pointer', textDecoration: 'underline'
            }}
          >
            {isLogin ? 'Hesabınız yok mu? Kayıt Olun' : 'Zaten hesabınız var mı? Giriş Yapın'}
          </button>
        </div>
      </motion.div>
    </div>
  );
}
