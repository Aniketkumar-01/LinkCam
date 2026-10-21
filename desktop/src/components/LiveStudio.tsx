import React, { useEffect, useRef, useState } from 'react';
import { Grid, Smartphone, Video } from 'lucide-react';
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
  onOpenDiagnostics?: () => void;
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
  onOpenDiagnostics,
}) => {
  const [lens, setLens] = useState<CameraLensType>('RearWide');
  const [torch, setTorch] = useState(false);
  const [zoom, setZoom] = useState(1.0);
  const [exposure, setExposure] = useState(0.0);
  const [meetingMode, setMeetingMode] = useState(false);
  const [resolution, setResolution] = useState('1080p60');
  const [showGrid, setShowGrid] = useState(false);
  const [hasReceivedVideo, setHasReceivedVideo] = useState(false);

  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const decoderRef = useRef<any>(null);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (!isConnected) {
      setHasReceivedVideo(false);
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
      if (decoderRef.current) {
        try {
          decoderRef.current.close();
        } catch (_) {}
        decoderRef.current = null;
      }
      return;
    }

    let isDisposed = false;

    // Initialize WebCodecs VideoDecoder if supported by the browser/WebView2
    const initDecoder = () => {
      if (typeof window !== 'undefined' && 'VideoDecoder' in window) {
        try {
          const dec = new (window as any).VideoDecoder({
            output: (videoFrame: any) => {
              if (isDisposed) {
                videoFrame.close();
                return;
              }
              const canvas = canvasRef.current;
              if (canvas) {
                if (canvas.width !== videoFrame.displayWidth || canvas.height !== videoFrame.displayHeight) {
                  canvas.width = videoFrame.displayWidth;
                  canvas.height = videoFrame.displayHeight;
                }
                const ctx = canvas.getContext('2d');
                if (ctx) {
                  ctx.drawImage(videoFrame, 0, 0);
                }
              }
              videoFrame.close();
              setHasReceivedVideo(true);
            },
            error: (err: any) => {
              console.warn('[LinkCam WebCodecs] Decoder error:', err);
            },
          });

          dec.configure({
            codec: 'avc1.640028', // H.264 High Profile Level 4.0 (1080p60)
            optimizeForLatency: true,
          });

          decoderRef.current = dec;
        } catch (e) {
          console.warn('[LinkCam] WebCodecs initialization failed:', e);
        }
      }
    };

    initDecoder();

    // Connect to Companion local WebSocket server for zero-latency frame feed
    const connectWs = () => {
      if (isDisposed) return;
      try {
        const ws = new WebSocket('ws://127.0.0.1:8443');
        ws.binaryType = 'arraybuffer';
        wsRef.current = ws;

        ws.onopen = () => {
          console.log('[LinkCam UI] Live preview WebSocket stream connected');
        };

        ws.onmessage = (event) => {
          if (isDisposed) return;
          const data = new Uint8Array(event.data as ArrayBuffer);
          if (data.length <= 32) return; // Must contain 32-byte LinkCam PacketHeader

          const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
          const magic = view.getUint32(0, false);
          if (magic !== 0x4c4e4b43) return; // 'LNKC'

          const packetType = view.getUint8(5);
          if (packetType !== 0x01) return; // PacketType::VideoFrame

          const flags = view.getUint8(6);
          const isKeyframe = (flags & 0x01) !== 0; // FLAG_KEYFRAME
          const timestamp = Number(view.getBigUint64(12, false));

          const payload = data.subarray(32);

          const decoder = decoderRef.current;
          if (decoder && decoder.state === 'configured') {
            try {
              const chunk = new (window as any).EncodedVideoChunk({
                type: isKeyframe ? 'key' : 'delta',
                timestamp: timestamp * 1000,
                data: payload,
              });
              decoder.decode(chunk);
            } catch (decErr) {
              // If decode errors on delta before first keyframe, wait for next keyframe
            }
          }
        };

        ws.onerror = () => {
          // Retry automatically after backoff
        };

        ws.onclose = () => {
          if (!isDisposed && isConnected) {
            setTimeout(connectWs, 2000);
          }
        };
      } catch (err) {
        console.warn('[LinkCam] WebSocket preview error:', err);
      }
    };

    connectWs();

    return () => {
      isDisposed = true;
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
      if (decoderRef.current) {
        try {
          decoderRef.current.close();
        } catch (_) {}
        decoderRef.current = null;
      }
    };
  }, [isConnected]);

  return (
    <div className="flex-1 h-full flex flex-col p-6 gap-4 overflow-hidden">
      {/* Viewfinder Canvas Container */}
      <div className="relative flex-1 w-full bg-slate-950 rounded-2xl overflow-hidden border border-white/[0.08] shadow-2xl flex items-center justify-center">
        {isConnected ? (
          /* Live Stream Viewfinder View */
          <div className="relative w-full h-full flex items-center justify-center bg-black">
            <div className="relative aspect-video w-full h-full max-h-full flex items-center justify-center overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-t from-slate-950/60 via-transparent to-slate-950/30 pointer-events-none z-10" />

              {/* Real Hardware-Decoded Live Video Canvas */}
              <canvas
                ref={canvasRef}
                className={`w-full h-full object-contain transition-opacity duration-300 ${
                  hasReceivedVideo ? 'opacity-100' : 'opacity-0'
                }`}
              />

              {/* Standby Placeholder when waiting for first video frame */}
              {!hasReceivedVideo && (
                <div className="absolute inset-0 bg-gradient-to-tr from-slate-900 via-indigo-950/40 to-slate-900 flex flex-col items-center justify-center gap-3">
                  <div className="w-16 h-16 border border-blue-400/30 rounded-xl flex items-center justify-center bg-blue-500/10 backdrop-blur-sm">
                    <Video className="w-7 h-7 text-blue-400 animate-pulse" />
                  </div>
                  <div className="text-center">
                    <p className="text-sm font-medium text-slate-200">Starting Live Stream...</p>
                    <p className="text-xs text-slate-400 mt-0.5">Synchronizing hardware video pipeline</p>
                  </div>
                </div>
              )}

              {/* Center Focus Reticle Overlay */}
              <div className="absolute pointer-events-none z-10 w-16 h-16 border border-white/25 rounded-lg flex items-center justify-center shadow-lg">
                <div className="w-2 h-2 bg-blue-400 rounded-full animate-ping opacity-75" />
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
                  <div className="border-r border-b border-white/10" />
                  <div className="border-r border-b border-white/10" />
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
              onOpenDiagnostics={onOpenDiagnostics}
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
