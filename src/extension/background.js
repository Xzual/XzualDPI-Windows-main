// XzualDPI Domain Tracker - Service Worker

// Configuration
const SUPABASE_URL = 'https://oouuamfaursuqyuekmog.supabase.co';
const SUPABASE_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im9vdXVhbWZhdXJzdXF5dWVrbW9nIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzgzMzQ4OTEsImV4cCI6MjA5MzkxMDg5MX0.jlIJ1ey1w-BssRY4caUVwz4jNDDX6yrLwr59jYtyiIk';

const domainsSet = new Set();
const FLUSH_INTERVAL = 30000; // 30 seconds
const MAX_DOMAINS_BATCH = 100;

// Get user device ID
async function getDeviceId() {
  const stored = await chrome.storage.local.get('xzual_device_id');
  return stored.xzual_device_id || null;
}

// Get user session token
async function getUserToken() {
  const stored = await chrome.storage.local.get('xzual_session_token');
  return stored.xzual_session_token || null;
}

// Extract domain from URL
function extractDomain(url) {
  try {
    const urlObj = new URL(url);
    const hostname = urlObj.hostname;
    
    // Skip localhost and IPs
    if (hostname === 'localhost' || hostname.startsWith('127.') || /^\d+\.\d+\.\d+\.\d+$/.test(hostname)) {
      return null;
    }
    
    // Skip internal domains
    if (hostname.includes('ip-api') || hostname.includes('localhost')) {
      return null;
    }
    
    return hostname.toLowerCase();
  } catch (e) {
    return null;
  }
}

// Track tab URL changes
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'loading' || changeInfo.status === 'complete') {
    const domain = extractDomain(tab.url);
    if (domain) {
      domainsSet.add(domain);
    }
  }
});

// Track tab activate
chrome.tabs.onActivated.addListener(async (activeInfo) => {
  const tab = await chrome.tabs.get(activeInfo.tabId);
  const domain = extractDomain(tab.url);
  if (domain) {
    domainsSet.add(domain);
  }
});

// Flush domains to Supabase
async function flushDomains() {
  if (domainsSet.size === 0) {
    return;
  }

  const deviceId = await getDeviceId();
  const token = await getUserToken();

  if (!deviceId || !token) {
    console.warn('XzualDPI: No device ID or session token');
    return;
  }

  const domains = Array.from(domainsSet);
  domainsSet.clear();

  try {
    const response = await fetch(`${SUPABASE_URL}/rest/v1/connection_logs`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${SUPABASE_KEY}`,
        'apikey': SUPABASE_KEY,
      },
      body: JSON.stringify(
        domains.map(domain => ({
          device_id: deviceId,
          domain: domain,
          timestamp: new Date().toISOString()
        }))
      )
    });

    if (!response.ok) {
      throw new Error(`Failed to insert logs: ${response.statusText}`);
    }

    console.log(`XzualDPI: Flushed ${domains.length} domains`);
  } catch (e) {
    console.error('XzualDPI: Flush failed', e);
    // Re-add domains for next flush
    domains.forEach(d => domainsSet.add(d));
  }
}

// Set up interval flush
setInterval(flushDomains, FLUSH_INTERVAL);

// Flush on extension unload
chrome.runtime.onSuspend.addListener(() => {
  flushDomains();
});

// Listen for messages from content script
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'trackDomain') {
    const domain = extractDomain(request.url);
    if (domain) {
      domainsSet.add(domain);
    }
  } else if (request.action === 'setDeviceId') {
    chrome.storage.local.set({ xzual_device_id: request.deviceId });
  } else if (request.action === 'setSessionToken') {
    chrome.storage.local.set({ xzual_session_token: request.token });
  }
  sendResponse({ success: true });
});

console.log('XzualDPI Domain Tracker initialized');
