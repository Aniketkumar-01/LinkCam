import React, { useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { Check, Copy, RefreshCw, Smartphone, Usb, Wifi, Zap } from 'lucide-react';

interface PairingWizardProps {
  serverPort: number;
  availableIps: string[];
  sessionToken: string;
  onRefreshPairing: () => void;
  onSimulateConnect: () => void;
}

export const PairingWizard: React.FC<PairingWizardProps> = ({
  serverPort,
  availableIps,
  sessionToken,
  onRefreshPairing,
  onSimulateConnect,
}) => {
  const [activeMode, setActiveMode] = useState<'hotspot' | 'wifi' | 'usb'>('hotspot');
  const [copied, setCopied] = useState(false);

  const pairingData = JSON.stringify({
    port: serverPort,
    ips: availableIps,
    token: sessionToken,
    proto: 1,
  });

  const copyPairingCode = () => {
    navigator.clipboard.writeText(pairingData);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex-1 h-full p-8 overflow-y-auto max-w-4xl mx-auto space-y-8">
      {/* Header */}
      <div>
        <h2 className="text-xl font-semibold text-slate-100 tracking-tight">Connect Your Device</h2>
        <p className="text-sm text-slate-400 mt-1">
          Pair your Android phone to use its camera as a high-fidelity Windows webcam or instant document scanner.
        </p>
      </div>

      {/* Mode Selection Tabs */}
      <div className="flex gap-2 p-1 bg-black/30 rounded-xl border border-white/[0.06] w-fit">
        <button
          onClick={() => setActiveMode('hotspot')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-medium transition-all ${
            activeMode === 'hotspot'
              ? 'bg-blue-600 text-white shadow-md'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Wifi className="w-3.5 h-3.5" />
          <span>Direct Hotspot (Zero Router)</span>
        </button>

        <button
          onClick={() => setActiveMode('wifi')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-medium transition-all ${
            activeMode === 'wifi'
              ? 'bg-blue-600 text-white shadow-md'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Wifi className="w-3.5 h-3.5" />
          <span>Local Wi-Fi Network</span>
        </button>

        <button
          onClick={() => setActiveMode('usb')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-medium transition-all ${
            activeMode === 'usb'
              ? 'bg-blue-600 text-white shadow-md'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Usb className="w-3.5 h-3.5" />
          <span>USB Cable (Ultra Low Latency)</span>
        </button>
      </div>

      {/* Content for Hotspot / Wi-Fi Modes (QR Code pairing) */}
      {activeMode !== 'usb' ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-start">
          {/* QR Code Container */}
          <div className="fluent-card p-6 rounded-2xl flex flex-col items-center justify-center space-y-4 shadow-xl">
            <div className="p-4 bg-white rounded-xl shadow-md">
              <QRCodeSVG
                value={pairingData}
                size={220}
                level="M"
                includeMargin={false}
              />
            </div>

            <div className="flex items-center gap-2 w-full pt-2">
              <button
                onClick={onRefreshPairing}
                className="flex-1 fluent-btn-subtle py-2 px-3 text-xs flex items-center justify-center gap-1.5"
                title="Generate new temporary pairing token"
              >
                <RefreshCw className="w-3.5 h-3.5 text-slate-400" />
                <span>Refresh QR</span>
              </button>

              <button
                onClick={copyPairingCode}
                className="fluent-btn-subtle py-2 px-3 text-xs flex items-center justify-center gap-1.5"
                title="Copy raw connection payload"
              >
                {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copied ? 'Copied' : 'Copy'}</span>
              </button>
            </div>
          </div>

          {/* Step-by-Step Instructions */}
          <div className="space-y-6">
            {activeMode === 'hotspot' ? (
              <div className="space-y-4">
                <div className="flex items-center gap-2 text-blue-400 font-medium text-sm">
                  <span className="w-6 h-6 rounded-full bg-blue-600/20 flex items-center justify-center text-xs">★</span>
                  <span>Direct Mobile Hotspot Guide (No Wi-Fi router needed)</span>
                </div>

                <ol className="space-y-3 text-xs text-slate-300">
                  <li className="flex gap-3">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">1</span>
                    <span>Turn on <strong>Mobile Hotspot</strong> on your Android phone (Settings &gt; Network &gt; Hotspot).</span>
                  </li>
                  <li className="flex gap-3">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">2</span>
                    <span>On this PC, click your Wi-Fi icon in the taskbar and connect to your phone's hotspot network.</span>
                  </li>
                  <li className="flex gap-3">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">3</span>
                    <span>Open the LinkCam app on your phone and tap <strong>Scan QR</strong> to point at the code on the left.</span>
                  </li>
                </ol>

                <div className="p-3.5 bg-blue-950/30 border border-blue-500/20 rounded-xl text-xs text-blue-200">
                  💡 <strong>Direct Connection:</strong> Your phone acts as the private router. Video frames stream directly between your phone and laptop with zero internet usage.
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                <div className="flex items-center gap-2 text-slate-200 font-medium text-sm">
                  <span>Local Wi-Fi Network Guide</span>
                </div>

                <ol className="space-y-3 text-xs text-slate-300">
                  <li className="flex gap-3">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">1</span>
                    <span>Ensure both your phone and this Windows PC are connected to the same Wi-Fi network (5GHz recommended).</span>
                  </li>
                  <li className="flex gap-3">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">2</span>
                    <span>Open LinkCam on your phone and point the camera at the QR code.</span>
                  </li>
                </ol>
              </div>
            )}

            {/* Quick Demo Simulator Button */}
            <div className="pt-4 border-t border-white/[0.08]">
              <button
                onClick={onSimulateConnect}
                className="fluent-btn-primary w-full py-2.5 px-4 text-xs font-semibold flex items-center justify-center gap-2 shadow-lg"
              >
                <Zap className="w-4 h-4" />
                <span>Simulate Phone Connected (Preview Mode)</span>
              </button>
            </div>
          </div>
        </div>
      ) : (
        /* USB Wired Instructions */
        <div className="fluent-card p-6 rounded-2xl space-y-6">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-amber-500/20 border border-amber-500/30 flex items-center justify-center text-amber-400">
              <Usb className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-slate-100">Wired USB Plug-and-Play</h3>
              <p className="text-xs text-slate-400">Highest bandwidth, lowest latency (&lt;40ms), and charges your phone simultaneously.</p>
            </div>
          </div>

          <ol className="space-y-3 text-xs text-slate-300">
            <li className="flex gap-3">
              <span className="font-mono text-amber-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">1</span>
              <span>Connect your phone to your PC with a high-speed USB-C cable (USB 3.0 recommended).</span>
            </li>
            <li className="flex gap-3">
              <span className="font-mono text-amber-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">2</span>
              <span>Enable <strong>USB Debugging</strong> or USB Tethering in your phone's Developer Options.</span>
            </li>
            <li className="flex gap-3">
              <span className="font-mono text-amber-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">3</span>
              <span>LinkCam will automatically detect the device and establish the video pipe.</span>
            </li>
          </ol>

          <div className="pt-2">
            <button
              onClick={onSimulateConnect}
              className="fluent-btn-primary py-2.5 px-6 text-xs font-semibold flex items-center gap-2"
            >
              <span>Connect via USB</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
