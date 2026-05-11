import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';
import pngToIco from 'png-to-ico';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
    const rceditModule = await import('rcedit');
    const rcedit = rceditModule.rcedit || rceditModule.default;
    
    // Updated paths to match new lowercase naming convention
    const exePath = path.join(__dirname, 'src-tauri', 'binaries', 'xzual-proxy-x86_64-pc-windows-msvc.exe');
    const logoPngPath = path.join(__dirname, 'public', 'xzual-logo.png');
    const enginePngPath = path.join(__dirname, 'public', 'xzual-engine.png');
    const iconPath = path.join(__dirname, 'src-tauri', 'icons', 'icon.ico');
    const engineIconPath = path.join(__dirname, 'src-tauri', 'icons', 'xzual-engine.ico');
    const uninstallIconPath = path.join(__dirname, 'src-tauri', 'icons', 'uninstall.ico');

    console.log(`Converting ${logoPngPath} to ICO...`);
    try {
        const buf = await pngToIco(logoPngPath);
        fs.writeFileSync(iconPath, buf);
        console.log(`Updated icon.ico at ${iconPath}`);
        
        const uninstallBuf = await pngToIco(path.join(__dirname, 'public', 'xzual-uninstall.png'));
        fs.writeFileSync(uninstallIconPath, uninstallBuf);
        console.log(`Updated uninstall.ico at ${uninstallIconPath}`);

        if (fs.existsSync(enginePngPath)) {
            const engineBuf = await pngToIco(enginePngPath);
            fs.writeFileSync(engineIconPath, engineBuf);
            console.log(`Updated xzual-engine.ico at ${engineIconPath}`);
        }
    } catch (err) {
        console.error('Failed to convert PNG to ICO:', err);
    }

    if (fs.existsSync(exePath)) {
        console.log(`Updating icon for ${exePath} using ${iconPath}`);
        try {
            await rcedit(exePath, {
                icon: iconPath,
                'version-string': {
                    ProductName: 'XzualDPI',
                    FileDescription: 'XzualDPI Service',
                    CompanyName: 'ConsolAktif',
                    LegalCopyright: 'Copyright © 2026 ConsolAktif'
                }
            });
            console.log('Proxy binary icon updated successfully!');
        } catch (e) {
            console.error('Failed to update proxy binary icon:', e);
        }
    } else {
        console.warn(`Proxy binary not found at ${exePath}. Skipping binary icon update.`);
    }
}

main();
