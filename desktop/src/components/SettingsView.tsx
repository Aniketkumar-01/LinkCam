import React, { useState } from 'react';
import { Camera, Check, Folder, Info, ShieldCheck, Video } from 'lucide-react';

export const SettingsView: React.FC = () => {
  const [savePath, setSavePath] = useState('C:\\Users\\User\\Pictures\\LinkCam');
  const [vcamEnabled, setVcamEnabled] = useState(true);
  const [hardwareAccel, setHardwareAccel] = useState(true);
  const [autoClipboard, setAutoClipboard] = useState(true);

  return (
    <div className="flex-1 h-full p-8 overflow-y-auto max-w-4xl mx-auto space-y-8">
      <div>
        <h2 className="text-xl font-semibold text-slate-100 tracking-tight">Settings & Preferences</h2>
        <p className="text-sm text-slate-400 mt-1">
          Configure Windows Virtual Camera system integration and file storage.
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
          <h3 className="text-sm font-semibold text-slate-100">Instant Capture Delivery</h3>

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

            {/* Folder Destination */}
            <div className="pt-3 border-t border-white/[0.06] space-y-1.5">
              <div className="font-medium text-slate-200">Save Destination</div>
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={savePath}
                  onChange={(e) => setSavePath(e.target.value)}
                  className="flex-1 bg-black/40 border border-white/[0.08] rounded-lg px-3 py-2 text-xs font-mono text-slate-300 focus:outline-none focus:border-blue-500"
                />
                <button className="fluent-btn-subtle px-3 py-2 text-xs flex items-center gap-1.5">
                  <Folder className="w-3.5 h-3.5 text-slate-400" />
                  <span>Browse</span>
                </button>
              </div>
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
            LinkCam operates 100% locally on your local network, direct mobile hotspot, or USB cable. No video frames, audio, or document captures are ever routed through external servers or stored in the cloud.
          </p>
        </div>
      </div>
    </div>
  );
};
