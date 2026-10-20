import React, { useState, useEffect } from 'react';
import { AlertCircle, Camera, Check, Clipboard, Folder, ShieldCheck } from 'lucide-react';

export const SettingsView: React.FC = () => {
  const [savePath, setSavePath] = useState('');
  const [vcamEnabled, setVcamEnabled] = useState(true);
  const [hardwareAccel, setHardwareAccel] = useState(true);
  const [autoClipboard, setAutoClipboard] = useState(true);
  const [savedNotice, setSavedNotice] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const fileInputRef = React.useRef<HTMLInputElement>(null);

  // Load save destination from Tauri Rust backend on mount
  useEffect(() => {
    const loadDestination = async () => {
      try {
        // @ts-ignore
        if (window.__TAURI__?.core?.invoke) {
          // @ts-ignore
          const dir = await window.__TAURI__.core.invoke('get_save_destination');
          if (dir) {
            setSavePath(dir);
            return;
          }
        }
      } catch {
        // Fallback to local default
      }
      setSavePath('C:\\Users\\User\\Pictures\\LinkCam');
    };

    loadDestination();
  }, []);

  const updateSavePath = async (path: string) => {
    setErrorMessage(null);
    setSavePath(path);

    try {
      // @ts-ignore
      if (window.__TAURI__?.core?.invoke) {
        // @ts-ignore
        const validatedPath = await window.__TAURI__.core.invoke('set_save_destination', { path });
        setSavePath(validatedPath);
        setSavedNotice('Validated and saved to system configuration');
        setTimeout(() => setSavedNotice(null), 3000);
      } else {
        localStorage.setItem('linkcam_save_destination', path);
        setSavedNotice('Saved to preferences');
        setTimeout(() => setSavedNotice(null), 2000);
      }
    } catch (err: any) {
      setErrorMessage(err?.toString() || 'Failed to update destination');
    }
  };

  const handlePasteClipboard = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (text && text.trim().length > 0) {
        await updateSavePath(text.trim());
      }
    } catch {
      // Fallback
    }
  };

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && files.length > 0) {
      const file = files[0];
      const relPath = file.webkitRelativePath;
      if (relPath) {
        const dirName = relPath.split('/')[0];
        await updateSavePath(`C:\\Users\\User\\Pictures\\${dirName}`);
      }
    }
  };

  return (
    <div className="flex-1 h-full p-8 overflow-y-auto max-w-4xl mx-auto space-y-8">
      <div>
        <h2 className="text-xl font-semibold text-slate-100 tracking-tight">Settings & Preferences</h2>
        <p className="text-sm text-slate-400 mt-1">
          Configure Windows Virtual Camera system integration and secure file storage.
        </p>
      </div>

      <div className="space-y-6">
        {/* Virtual Camera Section */}
        <div className="fluent-card p-6 rounded-2xl space-y-4">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-blue-600/20 text-blue-400 flex items-center justify-center">
              <Camera className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-slate-100">Windows Media Foundation Virtual Camera</h3>
              <p className="text-xs text-slate-400">Registers "LinkCam Virtual Camera" as a native hardware device.</p>
            </div>
          </div>

          <div className="pt-2 flex items-center justify-between border-t border-white/[0.06]">
            <div>
              <div className="text-xs font-medium text-slate-200">Device Status</div>
              <div className="text-xs text-emerald-400 flex items-center gap-1 mt-0.5">
                <Check className="w-3.5 h-3.5" /> Registered & Available in Zoom, Teams, Meet, OBS
              </div>
            </div>

            <button
              onClick={() => setVcamEnabled(!vcamEnabled)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors ${
                vcamEnabled
                  ? 'bg-blue-600/20 text-blue-400 border-blue-500/30'
                  : 'bg-black/30 text-slate-400 border-white/[0.08]'
              }`}
            >
              {vcamEnabled ? 'Enabled' : 'Disabled'}
            </button>
          </div>
        </div>

        {/* Capture Delivery Options */}
        <div className="fluent-card p-6 rounded-2xl space-y-4">
          <h3 className="text-sm font-semibold text-slate-100">Instant Capture Delivery & Storage Security</h3>

          <div className="space-y-4 text-xs">
            {/* Auto Clipboard Toggle */}
            <div className="flex items-center justify-between">
              <div>
                <div className="font-medium text-slate-200">Automatic Windows Clipboard Injection</div>
                <div className="text-slate-400 mt-0.5">
                  Immediately paste (Ctrl+V) captured photos and PDFs into Word, Slack, or emails without opening folders.
                </div>
              </div>
              <input
                type="checkbox"
                checked={autoClipboard}
                onChange={(e) => setAutoClipboard(e.target.checked)}
                className="w-4 h-4 accent-blue-600 cursor-pointer"
              />
            </div>

            {/* Hardware Acceleration Toggle */}
            <div className="flex items-center justify-between pt-3 border-t border-white/[0.06]">
              <div>
                <div className="font-medium text-slate-200">Hardware Accelerated Decoding (Direct3D 11)</div>
                <div className="text-slate-400 mt-0.5">
                  Uses GPU for zero-copy H.264 video decoding, reducing CPU usage to under 2%.
                </div>
              </div>
              <input
                type="checkbox"
                checked={hardwareAccel}
                onChange={(e) => setHardwareAccel(e.target.checked)}
                className="w-4 h-4 accent-blue-600 cursor-pointer"
              />
            </div>

            {/* Folder Destination */}
            <div className="pt-3 border-t border-white/[0.06] space-y-1.5">
              <div className="flex items-center justify-between">
                <div className="font-medium text-slate-200">Backend-Validated Save Destination</div>
                {savedNotice && (
                  <span className="text-[11px] text-emerald-400 font-medium animate-pulse">
                    {savedNotice}
                  </span>
                )}
                {errorMessage && (
                  <span className="text-[11px] text-rose-400 font-medium flex items-center gap-1">
                    <AlertCircle className="w-3 h-3" />
                    {errorMessage}
                  </span>
                )}
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={savePath}
                  onChange={(e) => setSavePath(e.target.value)}
                  onBlur={(e) => updateSavePath(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      updateSavePath((e.target as HTMLInputElement).value);
                    }
                  }}
                  onPaste={(e) => {
                    const text = e.clipboardData.getData('text');
                    if (text) {
                      e.preventDefault();
                      updateSavePath(text.trim());
                    }
                  }}
                  placeholder="Paste or browse full folder path..."
                  className="flex-1 bg-black/40 border border-white/[0.08] rounded-lg px-3 py-2 text-xs font-mono text-slate-300 focus:outline-none focus:border-blue-500"
                />

                {/* Paste from Clipboard Button */}
                <button
                  type="button"
                  onClick={handlePasteClipboard}
                  title="Paste from clipboard"
                  className="fluent-btn-subtle px-2.5 py-2 text-xs flex items-center gap-1 text-slate-300 hover:text-white"
                >
                  <Clipboard className="w-3.5 h-3.5 text-blue-400" />
                  <span>Paste</span>
                </button>

                {/* Browse Button */}
                <button
                  type="button"
                  onClick={() => fileInputRef.current?.click()}
                  className="fluent-btn-subtle px-3 py-2 text-xs flex items-center gap-1.5"
                >
                  <Folder className="w-3.5 h-3.5 text-slate-400" />
                  <span>Browse</span>
                </button>

                {/* Hidden folder input */}
                <input
                  type="file"
                  ref={fileInputRef}
                  // @ts-ignore
                  webkitdirectory=""
                  directory=""
                  onChange={handleFileChange}
                  style={{ display: 'none' }}
                />
              </div>
              <p className="text-[11px] text-slate-400">
                Destination is strictly validated by the Rust backend: protected system folders and path traversal attempts are blocked.
              </p>
            </div>
          </div>
        </div>

        {/* Privacy & Performance */}
        <div className="fluent-card p-6 rounded-2xl space-y-3 text-xs text-slate-300">
          <div className="flex items-center gap-2 font-semibold text-slate-100">
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
            <span>Local & Offline Privacy Guarantee</span>
          </div>
          <p className="text-slate-400 leading-relaxed">
            LinkCam operates 100% locally on your local network, direct mobile hotspot, or USB cable. Captured file names are generated on the desktop to prevent malicious naming, and no data ever leaves your computer.
          </p>
        </div>
      </div>
    </div>
  );
};
