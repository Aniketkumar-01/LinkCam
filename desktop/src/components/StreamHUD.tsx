import React from 'react';
import { Activity, BatteryCharging, CheckCircle2, Gauge, Wifi, Zap } from 'lucide-react';

interface StreamHUDProps {
  fps: number;
  bitrateKbps: number;
  latencyMs: number;
  batteryPct: number;
  isCharging: boolean;
  connectionMode: 'Usb' | 'DirectHotspot' | 'WiFi';
  isVcamActive: boolean;
}

export const StreamHUD: React.FC<StreamHUDProps> = ({
  fps,
  bitrateKbps,
  latencyMs,
  batteryPct,
  isCharging,
  connectionMode,
  isVcamActive,
}) => {
  const bitrateMbps = (bitrateKbps / 1000).toFixed(1);

  return (
    <div className="absolute top-4 left-4 right-4 flex items-center justify-between pointer-events-none">
      {/* Left: Virtual Camera Status */}
      <div className="flex items-center gap-2 pointer-events-auto">
        <div className="fluent-acrylic px-3 py-1.5 rounded-full flex items-center gap-2 shadow-lg text-xs font-medium">
          <span className={`w-2 h-2 rounded-full ${isVcamActive ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'}`} />
          <span className="text-slate-200">
            {isVcamActive ? 'Windows Virtual Camera: Live' : 'Virtual Camera: Ready'}
          </span>
          <span className="text-[10px] bg-white/[0.08] text-slate-300 px-1.5 py-0.5 rounded">
            Teams • Zoom • OBS
          </span>
        </div>
      </div>

      {/* Right: Live Telemetry HUD */}
      <div className="fluent-acrylic px-3.5 py-1.5 rounded-full flex items-center gap-4 text-xs shadow-lg pointer-events-auto">
        {/* Connection mode */}
        <div className="flex items-center gap-1.5 text-slate-300 font-medium">
          {connectionMode === 'Usb' ? (
            <Zap className="w-3.5 h-3.5 text-amber-400" />
          ) : (
            <Wifi className="w-3.5 h-3.5 text-blue-400" />
          )}
          <span>{connectionMode === 'DirectHotspot' ? 'Hotspot Direct' : connectionMode}</span>
        </div>

        <div className="w-px h-3.5 bg-white/[0.12]" />

        {/* FPS */}
        <div className="flex items-center gap-1 text-slate-300">
          <Activity className="w-3.5 h-3.5 text-emerald-400" />
          <span className="font-mono tabular-nums text-slate-100">{fps.toFixed(1)}</span>
          <span className="text-[10px] text-slate-400">FPS</span>
        </div>

        <div className="w-px h-3.5 bg-white/[0.12]" />

        {/* Bitrate */}
        <div className="flex items-center gap-1 text-slate-300">
          <Gauge className="w-3.5 h-3.5 text-indigo-400" />
          <span className="font-mono tabular-nums text-slate-100">{bitrateMbps}</span>
          <span className="text-[10px] text-slate-400">Mbps</span>
        </div>

        <div className="w-px h-3.5 bg-white/[0.12]" />

        {/* Latency */}
        <div className="flex items-center gap-1 text-slate-300">
          <span className="font-mono tabular-nums text-slate-100">{latencyMs}</span>
          <span className="text-[10px] text-slate-400">ms</span>
        </div>

        <div className="w-px h-3.5 bg-white/[0.12]" />

        {/* Battery */}
        <div className="flex items-center gap-1 text-slate-300">
          {isCharging ? (
            <BatteryCharging className="w-3.5 h-3.5 text-emerald-400" />
          ) : (
            <span className="text-xs">🔋</span>
          )}
          <span className="font-mono tabular-nums text-slate-100">{batteryPct}%</span>
        </div>
      </div>
    </div>
  );
};
