import React, { useState } from 'react';
import { Grid, Smartphone } from 'lucide-react';
import { CameraControls, CameraLensType } from './CameraControls';
import { StreamHUD } from './StreamHUD';

interface LiveStudioProps {
  isConnected: boolean;
  fps: number;
  bitrateKbps: number;
  latencyMs: number;
  batteryPct: number;
  isCharging: boolean;
  connectionMode: 'Usb' | 'DirectHotspot' | 'WiFi';
  isVcamActive: boolean;
  onTriggerPhoto: () => void;
  onTriggerDocumentScan: () => void;
}

export const LiveStudio: React.FC<LiveStudioProps> = ({
  isConnected,
  fps,
  bitrateKbps,
  latencyMs,
  batteryPct,
  isCharging,
  connectionMode,
  isVcamActive,
  onTriggerPhoto,
  onTriggerDocumentScan,
}) => {
  const [lens, setLens] = useState<CameraLensType>('RearWide');
  const [torch, setTorch] = useState(false);
  const [zoom, setZoom] = useState(1.0);
  const [exposure, setExposure] = useState(0.0);
  const [meetingMode, setMeetingMode] = useState(false);
  const [resolution, setResolution] = useState('1080p60');
  const [showGrid, setShowGrid] = useState(false);

  return (
    <div className="flex-1 h-full flex flex-col p-6 gap-4 overflow-hidden">
      {/* Viewfinder Canvas Container */}
      <div className="relative flex-1 w-full bg-slate-950 rounded-2xl overflow-hidden border border-white/[0.08] shadow-2xl flex items-center justify-center">
        {isConnected ? (
          /* Live Stream Viewfinder View */
          <div className="relative w-full h-full flex items-center justify-center bg-black">
            {/* Visual simulation of phone camera viewfinder */}
            <div className="relative aspect-video w-full h-full max-h-full flex items-center justify-center overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-t from-slate-950/80 via-transparent to-slate-950/40 pointer-events-none z-10" />

              {/* Viewfinder Video Stream Canvas */}
              <div className="w-full h-full bg-gradient-to-tr from-slate-900 via-indigo-950/40 to-slate-900 flex items-center justify-center">
                {/* Center focus reticle */}
                <div className="w-16 h-16 border border-white/20 rounded-lg flex items-center justify-center">
                  <div className="w-2 h-2 bg-blue-400 rounded-full animate-ping" />
                </div>
              </div>

              {/* Composition 3x3 Grid Overlay */}
              {showGrid && (
                <div className="absolute inset-0 grid grid-cols-3 grid-rows-3 pointer-events-none z-10">
                  <div className="border-r border-b border-white/10" />
                  <div className="border-r border-b border-white/10" />
                  <div className="border-b border-white/10" />
                  <div className="border-r border-b border-white/10" />
                  <div className="border-r border-b border-white/10" />
                  <div className="border-b border-white/10" />
                  <div className="border-r border-white/10" />
                  <div className="border-r border-white/10" />
                  <div />
                </div>
              )}
            </div>

            {/* Stream HUD overlay */}
            <StreamHUD
              fps={fps}
              bitrateKbps={bitrateKbps}
              latencyMs={latencyMs}
              batteryPct={batteryPct}
              isCharging={isCharging}
              connectionMode={connectionMode}
              isVcamActive={isVcamActive}
            />

            {/* Viewfinder Tool Overlay (Bottom Right of Canvas) */}
            <div className="absolute bottom-4 right-4 z-20 flex items-center gap-2">
              <button
                onClick={() => setShowGrid(!showGrid)}
                className={`p-2 rounded-lg fluent-acrylic text-xs transition-colors ${
                  showGrid ? 'text-blue-400 border-blue-500/40' : 'text-slate-400 hover:text-slate-200'
                }`}
                title="Toggle Rule-of-Thirds Grid"
              >
                <Grid className="w-4 h-4" />
              </button>
            </div>
          </div>
        ) : (
          /* Disconnected State / Waiting for Phone */
          <div className="text-center max-w-md p-8 space-y-4">
            <div className="w-16 h-16 rounded-2xl bg-blue-600/10 border border-blue-500/20 text-blue-400 flex items-center justify-center mx-auto shadow-inner">
              <Smartphone className="w-8 h-8" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-slate-100">Ready to Connect</h2>
              <p className="text-sm text-slate-400 mt-1">
                Open the LinkCam app on your Android phone and scan the QR code, or connect directly via USB cable / Mobile Hotspot.
              </p>
            </div>
            <div className="pt-2 flex justify-center gap-3">
              <div className="flex items-center gap-2 text-xs text-slate-400 bg-white/[0.04] px-3 py-1.5 rounded-lg border border-white/[0.06]">
                <span className="w-2 h-2 rounded-full bg-blue-400 animate-pulse" />
                <span>Listening on port 8443</span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Tactile Camera Controls Dock */}
      {isConnected && (
        <CameraControls
          currentLens={lens}
          onChangeLens={setLens}
          isTorchOn={torch}
          onToggleTorch={() => setTorch(!torch)}
          zoomRatio={zoom}
          onChangeZoom={setZoom}
          exposureEv={exposure}
          onChangeExposure={setExposure}
          isMeetingMode={meetingMode}
          onToggleMeetingMode={() => setMeetingMode(!meetingMode)}
          onTriggerPhoto={onTriggerPhoto}
          onTriggerDocumentScan={onTriggerDocumentScan}
          resolution={resolution}
          onChangeResolution={setResolution}
        />
      )}
    </div>
  );
};
