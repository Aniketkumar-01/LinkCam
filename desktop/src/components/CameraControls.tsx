import React from 'react';
import { Camera, FileText, Flashlight, Moon, Sun, ZoomIn } from 'lucide-react';

export type CameraLensType = 'RearWide' | 'RearUltraWide' | 'RearTelephoto' | 'Front';

interface CameraControlsProps {
  currentLens: CameraLensType;
  onChangeLens: (lens: CameraLensType) => void;
  isTorchOn: boolean;
  onToggleTorch: () => void;
  zoomRatio: number;
  onChangeZoom: (val: number) => void;
  exposureEv: number;
  onChangeExposure: (val: number) => void;
  isMeetingMode: boolean;
  onToggleMeetingMode: () => void;
  onTriggerPhoto: () => void;
  onTriggerDocumentScan: () => void;
  resolution: string;
  onChangeResolution: (res: string) => void;
}

export const CameraControls: React.FC<CameraControlsProps> = ({
  currentLens,
  onChangeLens,
  isTorchOn,
  onToggleTorch,
  zoomRatio,
  onChangeZoom,
  exposureEv,
  onChangeExposure,
  isMeetingMode,
  onToggleMeetingMode,
  onTriggerPhoto,
  onTriggerDocumentScan,
  resolution,
  onChangeResolution,
}) => {
  return (
    <div className="fluent-acrylic rounded-2xl p-4 shadow-2xl border border-fluent-border space-y-4">
      {/* Top Row: Primary Actions & Lens Selector */}
      <div className="flex flex-wrap items-center justify-between gap-4">
        {/* Lens Switcher */}
        <div className="flex items-center gap-1.5 bg-black/30 p-1 rounded-xl border border-white/[0.06]">
          {(
            [
              { id: 'RearUltraWide', label: '0.6x Wide' },
              { id: 'RearWide', label: '1x Main' },
              { id: 'RearTelephoto', label: '3x Tele' },
              { id: 'Front', label: 'Front' },
            ] as const
          ).map((lens) => (
            <button
              key={lens.id}
              onClick={() => onChangeLens(lens.id)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
                currentLens === lens.id
                  ? 'bg-blue-600 text-white shadow-md'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-white/[0.04]'
              }`}
            >
              {lens.label}
            </button>
          ))}
        </div>

        {/* Resolution selector */}
        <div className="flex items-center gap-1.5 bg-black/30 p-1 rounded-xl border border-white/[0.06]">
          {['1080p60', '1080p30', '4K30'].map((res) => (
            <button
              key={res}
              onClick={() => onChangeResolution(res)}
              className={`px-2.5 py-1.5 rounded-lg text-xs font-mono font-medium transition-all ${
                resolution === res
                  ? 'bg-white/10 text-blue-400 border border-blue-500/30'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              {res}
            </button>
          ))}
        </div>

        {/* Instant Capture Triggers */}
        <div className="flex items-center gap-2">
          <button
            onClick={onTriggerPhoto}
            className="fluent-btn-subtle px-3 py-1.5 flex items-center gap-1.5 text-xs font-medium hover:bg-white/10"
            title="Snap instant high-resolution photo straight to Windows"
          >
            <Camera className="w-3.5 h-3.5 text-blue-400" />
            <span>Snap Photo</span>
          </button>

          <button
            onClick={onTriggerDocumentScan}
            className="fluent-btn-primary px-3.5 py-1.5 flex items-center gap-1.5 text-xs font-medium shadow-md"
            title="Scan document with auto-crop & deliver PDF to clipboard"
          >
            <FileText className="w-3.5 h-3.5 text-white" />
            <span>Scan Document</span>
          </button>
        </div>
      </div>

      {/* Bottom Row: Tactile Sliders & Quick Toggles */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-3 border-t border-white/[0.06] text-xs text-slate-300">
        {/* Zoom Slider */}
        <div className="flex items-center gap-3 bg-black/20 px-3 py-2 rounded-xl border border-white/[0.04]">
          <ZoomIn className="w-4 h-4 text-slate-400" />
          <span className="w-12 text-slate-400">Zoom</span>
          <input
            type="range"
            min="1.0"
            max="5.0"
            step="0.1"
            value={zoomRatio}
            onChange={(e) => onChangeZoom(parseFloat(e.target.value))}
            className="flex-1 accent-blue-500 h-1.5 bg-slate-700 rounded-lg cursor-pointer"
          />
          <span className="w-9 font-mono text-right text-slate-200">{zoomRatio.toFixed(1)}x</span>
        </div>

        {/* Exposure Compensation */}
        <div className="flex items-center gap-3 bg-black/20 px-3 py-2 rounded-xl border border-white/[0.04]">
          <Sun className="w-4 h-4 text-slate-400" />
          <span className="w-16 text-slate-400">Exposure</span>
          <input
            type="range"
            min="-2.0"
            max="2.0"
            step="0.2"
            value={exposureEv}
            onChange={(e) => onChangeExposure(parseFloat(e.target.value))}
            className="flex-1 accent-blue-500 h-1.5 bg-slate-700 rounded-lg cursor-pointer"
          />
          <span className="w-12 font-mono text-right text-slate-200">
            {exposureEv >= 0 ? `+${exposureEv.toFixed(1)}` : exposureEv.toFixed(1)}
          </span>
        </div>

        {/* Hardware Toggles: Torch & OLED Meeting Mode */}
        <div className="flex items-center justify-end gap-2">
          {/* Torch Toggle */}
          <button
            onClick={onToggleTorch}
            className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-xl border text-xs font-medium transition-colors ${
              isTorchOn
                ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 shadow-sm'
                : 'bg-black/20 border-white/[0.04] text-slate-300 hover:bg-white/[0.06]'
            }`}
          >
            <Flashlight className={`w-3.5 h-3.5 ${isTorchOn ? 'text-amber-300' : 'text-slate-400'}`} />
            <span>Torch {isTorchOn ? 'ON' : 'OFF'}</span>
          </button>

          {/* OLED Meeting Mode */}
          <button
            onClick={onToggleMeetingMode}
            className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-xl border text-xs font-medium transition-colors ${
              isMeetingMode
                ? 'bg-purple-600/20 text-purple-300 border-purple-500/40 shadow-sm'
                : 'bg-black/20 border-white/[0.04] text-slate-300 hover:bg-white/[0.06]'
            }`}
            title="Blanks phone display to eliminate thermal heating and save battery while streaming"
          >
            <Moon className={`w-3.5 h-3.5 ${isMeetingMode ? 'text-purple-300' : 'text-slate-400'}`} />
            <span>Meeting Mode</span>
          </button>
        </div>
      </div>
    </div>
  );
};
