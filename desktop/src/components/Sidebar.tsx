import React from 'react';
import { Camera, Layers, QrCode, Settings, Smartphone, Wifi, Zap } from 'lucide-react';

export type ActiveTab = 'studio' | 'captures' | 'pairing' | 'settings';

interface SidebarProps {
  activeTab: ActiveTab;
  onSelectTab: (tab: ActiveTab) => void;
  isConnected: boolean;
  deviceName: string;
  batteryPct: number;
  connectionMode: 'Usb' | 'DirectHotspot' | 'WiFi';
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeTab,
  onSelectTab,
  isConnected,
  deviceName,
  batteryPct,
  connectionMode,
}) => {
  const navItems = [
    { id: 'studio' as ActiveTab, label: 'Live Studio', icon: Camera },
    { id: 'captures' as ActiveTab, label: 'Instant Captures', icon: Layers },
    { id: 'pairing' as ActiveTab, label: 'Pair Devices', icon: QrCode },
    { id: 'settings' as ActiveTab, label: 'Settings', icon: Settings },
  ];

  return (
    <aside className="w-64 h-full fluent-sidebar border-r border-fluent-border flex flex-col justify-between p-4">
      <div className="space-y-6">
        {/* App Title & Brand Header */}
        <div className="flex items-center gap-3 px-2 pt-2">
          <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-blue-600 to-indigo-700 flex items-center justify-center shadow-md shadow-blue-900/30">
            <Camera className="w-5 h-5 text-white" />
          </div>
          <div>
            <h1 className="text-base font-semibold text-slate-100 tracking-tight leading-none">LinkCam</h1>
            <p className="text-xs text-slate-400 mt-1">Windows Companion</p>
          </div>
        </div>

        {/* Navigation Tabs */}
        <nav className="space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onSelectTab(item.id)}
                className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                  isActive
                    ? 'bg-blue-600/20 text-blue-400 border border-blue-500/30'
                    : 'text-slate-400 hover:text-slate-200 hover:bg-white/[0.04]'
                }`}
              >
                <Icon className={`w-4 h-4 ${isActive ? 'text-blue-400' : 'text-slate-400'}`} />
                <span>{item.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      {/* Connected Device Badge Footer */}
      <div className="fluent-card p-3 rounded-xl space-y-2">
        <div className="flex items-center justify-between text-xs text-slate-400">
          <span className="flex items-center gap-1.5 font-medium text-slate-300">
            <Smartphone className="w-3.5 h-3.5 text-blue-400" />
            {isConnected ? deviceName : 'No Device Paired'}
          </span>
          <span
            className={`w-2 h-2 rounded-full ${
              isConnected ? 'bg-emerald-400 animate-pulse' : 'bg-slate-600'
            }`}
          />
        </div>

        {isConnected ? (
          <div className="flex items-center justify-between text-xs text-slate-400 pt-1 border-t border-white/[0.06]">
            <span className="flex items-center gap-1">
              {connectionMode === 'Usb' ? (
                <>
                  <Zap className="w-3 h-3 text-amber-400" /> USB 3.0
                </>
              ) : connectionMode === 'DirectHotspot' ? (
                <>
                  <Wifi className="w-3 h-3 text-blue-400" /> Hotspot Direct
                </>
              ) : (
                <>
                  <Wifi className="w-3 h-3 text-emerald-400" /> 5GHz Wi-Fi
                </>
              )}
            </span>
            <span className="font-mono text-slate-300">🔋 {batteryPct}%</span>
          </div>
        ) : (
          <button
            onClick={() => onSelectTab('pairing')}
            className="w-full text-xs text-blue-400 hover:text-blue-300 hover:underline pt-1 text-left"
          >
            Connect via QR or Hotspot →
          </button>
        )}
      </div>
    </aside>
  );
};
