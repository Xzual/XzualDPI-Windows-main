# XzualDPI Browser Extension - Domain Tracker

Bu extension XzualDPI ile kullanarak tarayıcıda ziyaret ettiğiniz tüm domainleri yakalamak için tasarlanmıştır.

## Nedir?

- ✅ Tüm tarayıcı sekmelerinizde ziyaret edilen siteleri otomatik yakalar
- ✅ Her 30 saniyede bir admin panele gönderir
- ✅ System proxy'den kaçan trafiği de yakalar
- ✅ Hafif ve hızlı - arka planda çalışır

## Kurulum

### Chrome/Edge/Brave:

1. **Extension dosyasını hazırla:**
   ```
   src/extension/ klasörünü kopyala
   ```

2. **Chrome Extension Manager'ı aç:**
   - `chrome://extensions/` adresine git
   - Sağ üstte "Developer mode" aç
   - "Load unpacked" tıkla
   - `src/extension/` klasörünü seç

3. **Hepsi bitti!** Extension otomatik olarak:
   - Tarayıcı sekmelerinizi monitör edecek
   - Ziyaret edilen domainleri toplayacak
   - Her 30 saniyede bir Supabase'e gönderecek

## Nasıl Çalışır?

1. Extension background script, tüm tab açılış ve değişikliklerini yakalar
2. Sitelerin domain adlarını extracte eder (IP'ler ve localhost hariç)
3. Memory'de 30 saniye boyunca tutar
4. Periyodik olarak Supabase `connection_logs` tablosuna INSERT eder
5. Admin panel'de gerçek zamanlı olarak görünür

## Sorun Giderme

**Extension kurulduktan sonra etkinleşmedi mi?**
- XzualDPI uygulamasını çalıştırıp oturum açın
- Extension'a device ID ve session token otomatik olarak gönderilecek

**Domainler görünmüyor mu?**
- Console'de hata olup olmadığını kontrol edin: `F12 → Console`
- Extension'ın enabled olduğundan emin olun

**Telemetry sorunu?**
- Admin panel → Dashboard'da canlı logları kontrol edin
- Eğer hiç veri gelmiyor ise, Supabase RLS policy'lerini kontrol edin

## Gizlilik

- Uzantı YALNIZCA domain adlarını yakalar (URL parametreleri, path vb. YAKALANMAZ)
- Tüm veriler doğrudan Supabase'inize kaydedilir
- Hiçbir harici sunucuya veri gönderilmez
