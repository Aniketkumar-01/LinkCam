import React, { useEffect, useRef, useState } from 'react';
import { Grid, Smartphone, Video } from 'lucide-react';
import { CameraControls, CameraLensType } from './CameraControls';
import { StreamHUD } from './StreamHUD';
import { invokeTauri } from '../tauriApi';

interface LiveStudioProps {
  isConnected: boolean;
  fps: number;
  bitrateKbps: number;
  latencyMs: number;
  batteryPct: number;
  isCharging: boolean;
  connectionMode: 'Usb' | 'DirectHotspot' | 'WiFi';
  isVcamActive: boolean;
  hasAudio?: boolean;
  audioLevelRms?: number;
  serverPort?: number;
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
  hasAudio = false,
  audioLevelRms = 0,
  serverPort = 8443,
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
  const [statusMessage, setStatusMessage] = useState('Synchronizing hardware video pipeline');

  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  // Helper to send camera control to phone via Tauri backend
  const sendControl = (action: string, params: Record<string, unknown>) => {
    invokeTauri('send_camera_control', { action, params }).catch((err) => {
      console.warn(`[LinkCam Controls] Failed to dispatch ${action}:`, err);
    });
  };

  const handleLensChange = (newLens: CameraLensType) => {
    setLens(newLens);
    sendControl('set_lens', { lens: newLens });
  };

  const handleToggleTorch = () => {
    const nextTorch = !torch;
    setTorch(nextTorch);
    sendControl('set_torch', { enabled: nextTorch });
  };

  const zoomDebounceRef = useRef<number | null>(null);
  const handleZoomChange = (val: number) => {
    setZoom(val);
    if (zoomDebounceRef.current) {
      window.clearTimeout(zoomDebounceRef.current);
    }
    zoomDebounceRef.current = window.setTimeout(() => {
      sendControl('set_zoom', { ratio: val });
    }, 40);
  };

  const exposureDebounceRef = useRef<number | null>(null);
  const handleExposureChange = (val: number) => {
    setExposure(val);
    if (exposureDebounceRef.current) {
      window.clearTimeout(exposureDebounceRef.current);
    }
    exposureDebounceRef.current = window.setTimeout(() => {
      sendControl('set_exposure', { ev: val });
    }, 40);
  };

  const handleToggleMeetingMode = () => {
    const nextMode = !meetingMode;
    setMeetingMode(nextMode);
    sendControl('set_meeting_mode', { enabled: nextMode });
  };
  const decoderRef = useRef<any>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const hasReceivedVideoRef = useRef(false);
  const hasSeenKeyframeRef = useRef(false);

  useEffect(() => {
    if (!isConnected) {
      setHasReceivedVideo(false);
      hasReceivedVideoRef.current = false;
      hasSeenKeyframeRef.current = false;
      setStatusMessage('Connecting to live stream pipeline...');
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
    hasReceivedVideoRef.current = false;
    hasSeenKeyframeRef.current = false;

    // Initialize WebCodecs VideoDecoder if supported by the browser/WebView2
    const initDecoder = async () => {
      if (typeof window === 'undefined' || !('VideoDecoder' in window)) {
        console.warn('[LinkCam WebCodecs] VideoDecoder API not supported in this runtime');
        setStatusMessage('Hardware WebCodecs not supported in browser runtime');
        return;
      }

      try {
        if (decoderRef.current && decoderRef.current.state !== 'closed') {
          try {
            decoderRef.current.close();
          } catch (_) {}
        }

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
            if (!hasReceivedVideoRef.current) {
              hasReceivedVideoRef.current = true;
              setHasReceivedVideo(true);
            }
          },
          error: (err: any) => {
            console.warn('[LinkCam WebCodecs] Decoder error:', err);
            hasSeenKeyframeRef.current = false;
            // Attempt clean recreation on next keyframe
            setTimeout(() => {
              if (!isDisposed) initDecoder();
            }, 100);
          },
        });

        // W3C Annex B configuration with automatic fallback
        const configs = [
          { codec: 'avc1.42001f', avc: { format: 'annexb' }, optimizeForLatency: true },
          { codec: 'avc1.640028', avc: { format: 'annexb' }, optimizeForLatency: true },
          { codec: 'avc1.4d002a', avc: { format: 'annexb' }, optimizeForLatency: true },
        ];

        let chosenConfig = configs[0];
        if (typeof (window as any).VideoDecoder?.isConfigSupported === 'function') {
          for (const cfg of configs) {
            try {
              const res = await (window as any).VideoDecoder.isConfigSupported(cfg);
              if (res && res.supported) {
                chosenConfig = cfg;
                break;
              }
            } catch (_) {}
          }
        }

        dec.configure(chosenConfig);
        decoderRef.current = dec;
        console.log('[LinkCam WebCodecs] Decoder configured with Annex B:', chosenConfig);
      } catch (e) {
        console.warn('[LinkCam] WebCodecs initialization failed:', e);
        setStatusMessage('Hardware video decoder initialization failed');
      }
    };

    initDecoder();

    // Connect to Companion local WebSocket server for zero-latency frame feed
    const connectWs = () => {
      if (isDisposed) return;
      try {
        const wsUrl = `ws://127.0.0.1:${serverPort}`;
        const ws = new WebSocket(wsUrl);
        ws.binaryType = 'arraybuffer';
        wsRef.current = ws;

        ws.onopen = () => {
          console.log(`[LinkCam UI] Live preview WebSocket stream connected to ${wsUrl}`);
          setStatusMessage('Waiting for camera keyframe...');
        };

        ws.onmessage = (event) => {
          if (isDisposed) return;
          const data = new Uint8Array(event.data as ArrayBuffer);
          const HEADER_SIZE = 24; // 24-byte LinkCam Wire PacketHeader
          if (data.length <= HEADER_SIZE) return;

          const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
          const magic = view.getUint32(0, false);
          if (magic !== 0x4c4e4b43) return; // 'LNKC'

          const packetType = view.getUint8(5);
          if (packetType !== 0x01) return; // PacketType::VideoFrame

          const flags = view.getUint8(6);
          const isKeyframeHeader = (flags & 0x01) !== 0; // FLAG_KEYFRAME
          const timestamp = Number(view.getBigUint64(12, false));

          const payload = data.subarray(HEADER_SIZE);

          // Robust keyframe detection: check header flag OR Annex B NAL unit types (SPS = 7, IDR = 5)
          const hasSpsOrIdr = payload.length > 4 && (
            (payload[0] === 0 && payload[1] === 0 && payload[2] === 0 && payload[3] === 1 && ((payload[4] & 0x1f) === 7 || (payload[4] & 0x1f) === 5)) ||
            (payload[0] === 0 && payload[1] === 0 && payload[2] === 1 && ((payload[3] & 0x1f) === 7 || (payload[3] & 0x1f) === 5))
          );
          const isKeyframe = isKeyframeHeader || hasSpsOrIdr;

          // WebCodecs requires that the first decoded chunk after configure/reset MUST be a keyframe.
          if (!hasSeenKeyframeRef.current) {
            if (!isKeyframe) {
              return; // Wait for initial IDR keyframe containing SPS/PPS before feeding decoder
            }
            hasSeenKeyframeRef.current = true;
          }

          let decoder = decoderRef.current;
          if (decoder && decoder.state === 'configured') {
            try {
              const chunk = new (window as any).EncodedVideoChunk({
                type: isKeyframe ? 'key' : 'delta',
                timestamp: timestamp > 0 ? timestamp * 1000 : performance.now() * 1000,
                data: payload,
              });
              decoder.decode(chunk);
            } catch (decErr) {
              console.warn('[LinkCam WebCodecs] Decode error, resetting for next keyframe:', decErr);
              hasSeenKeyframeRef.current = false;
            }
          } else if (!decoder || decoder.state === 'closed') {
            hasSeenKeyframeRef.current = false;
            initDecoder();
          }
        };

        ws.onerror = () => {
          // Retry automatically after backoff
        };

        ws.onclose = () => {
          if (!isDisposed && isConnected) {
            setTimeout(connectWs, 1500);
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
  }, [isConnected, serverPort]);

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
                    <p className="text-xs text-slate-400 mt-0.5">{statusMessage}</p>
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
              hasAudio={hasAudio}
              audioLevelRms={audioLevelRms}
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
          onChangeLens={handleLensChange}
          isTorchOn={torch}
          onToggleTorch={handleToggleTorch}
          zoomRatio={zoom}
          onChangeZoom={handleZoomChange}
          exposureEv={exposure}
          onChangeExposure={handleExposureChange}
          isMeetingMode={meetingMode}
          onToggleMeetingMode={handleToggleMeetingMode}
          onTriggerPhoto={onTriggerPhoto}
          onTriggerDocumentScan={onTriggerDocumentScan}
          resolution={resolution}
          onChangeResolution={setResolution}
        />
      )}
    </div>
  );
};
