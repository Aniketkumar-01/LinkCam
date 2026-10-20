import React, { useState, useEffect } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { Check, Copy, RefreshCw, ShieldCheck, Usb, Wifi, Zap, Smartphone, ArrowRight } from 'lucide-react';

interface PairingWizardProps {
  serverPort: number;
  availableIps: string[];
  sessionToken: string;
  sessionId?: string;
  verificationPin?: string;
  onRefreshPairing: () => void;
  onSimulateConnect?: () => void;
}

interface AdbDevice {
  serial: String;
  model: String;
  status: String;
  is_trusted: boolean;
}

export const PairingWizard: React.FC<PairingWizardProps> = ({
  serverPort,
  availableIps,
  sessionToken,
  sessionId,
  verificationPin = '582 391',
  onRefreshPairing,
  onSimulateConnect,
}) => {
  const [activeMode, setActiveMode] = useState<'hotspot' | 'wifi' | 'usb'>('hotspot');
  const [copied, setCopied] = useState(false);
  const [adbDevices, setAdbDevices] = useState<AdbDevice[]>([]);
  const [usbStatusMsg, setUsbStatusMsg] = useState<string | null>(null);
  const [isPushingToken, setIsPushingToken] = useState(false);

  const pairingData = JSON.stringify({
    port: serverPort,
    ips: availableIps,
    token: sessionToken,
    session_id: sessionId || 'default_session',
    pin: verificationPin,
  });

  const copyPairingCode = () => {
    navigator.clipboard.writeText(pairingData);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // Poll for connected ADB devices when in USB mode
  useEffect(() => {
    if (activeMode !== 'usb') return;

    const checkAdb = async () => {
      try {
        // @ts-ignore
        if (window.__TAURI__?.core?.invoke) {
          // @ts-ignore
          const devs = await window.__TAURI__.core.invoke('get_adb_devices');
          setAdbDevices(devs || []);
        }
      } catch {
        // Ignore in browser mock mode
      }
    };

    checkAdb();
    const interval = setInterval(checkAdb, 3000);
    return () => clearInterval(interval);
  }, [activeMode]);

  const handleTrustAndPair = async (serial: string) => {
    setIsPushingToken(true);
    setUsbStatusMsg(null);
    try {
      // @ts-ignore
      if (window.__TAURI__?.core?.invoke) {
        // @ts-ignore
        await window.__TAURI__.core.invoke('trust_adb_device', { serial });
        // @ts-ignore
        const res = await window.__TAURI__.core.invoke('pair_adb_device', { serial });
        setUsbStatusMsg(res || 'One-time secure USB token pushed to phone!');
      } else {
        setUsbStatusMsg(`Mock: Trusted & pushed USB token to ${serial}`);
      }
    } catch (e: any) {
      setUsbStatusMsg(`Error: ${e?.toString() || 'Failed to authorize device'}`);
    } finally {
      setIsPushingToken(false);
    }
  };

  const [pinCopied, setPinCopied] = useState(false);

  const copyPin = () => {
    navigator.clipboard.writeText(verificationPin.replace(/\s+/g, ''));
    setPinCopied(true);
    setTimeout(() => setPinCopied(false), 2000);
  };

  const handleManualUsbPair = async () => {
    setIsPushingToken(true);
    setUsbStatusMsg(null);
    try {
      // @ts-ignore
      if (window.__TAURI__?.core?.invoke) {
        // @ts-ignore
        const res = await window.__TAURI__.core.invoke('manual_usb_pair');
        setUsbStatusMsg(res || 'Port 8443 reversed & USB pairing token sent to phone!');
      } else {
        setUsbStatusMsg('Mock: Reversed port 8443 and pushed pairing token!');
      }
    } catch (e: any) {
      setUsbStatusMsg(`Error: ${e?.toString() || 'Failed to execute adb reverse'}`);
    } finally {
      setIsPushingToken(false);
    }
  };

  const checkAdbNow = async () => {
    try {
      // @ts-ignore
      if (window.__TAURI__?.core?.invoke) {
        // @ts-ignore
        const devs = await window.__TAURI__.core.invoke('get_adb_devices');
        setAdbDevices(devs || []);
      }
    } catch {
      // Ignore
    }
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
          <Zap className="w-4 h-4" />
          <span>Mobile Hotspot (Recommended)</span>
        </button>

        <button
          onClick={() => setActiveMode('wifi')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-medium transition-all ${
            activeMode === 'wifi'
              ? 'bg-blue-600 text-white shadow-md'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Wifi className="w-4 h-4" />
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
          <Usb className="w-4 h-4" />
          <span>USB Cable (Tethering / ADB)</span>
        </button>
      </div>

      {/* QR Code and Instructions Section */}
      {activeMode !== 'usb' ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-start">
          {/* Crisp QR Code Container */}
          <div className="fluent-card p-8 rounded-2xl flex flex-col items-center justify-center space-y-4">
            <div className="bg-white p-4 rounded-xl shadow-2xl ring-1 ring-black/10">
              <QRCodeSVG
                value={pairingData}
                size={220}
                level="L"
                includeMargin={true}
              />
            </div>

            <div className="flex items-center gap-2">
              <button
                onClick={onRefreshPairing}
                className="fluent-btn-subtle px-3 py-1.5 text-xs text-slate-300 flex items-center gap-1.5 hover:text-white"
              >
                <RefreshCw className="w-3.5 h-3.5 text-blue-400" />
                <span>Rotate Session Key</span>
              </button>

              <button
                onClick={copyPairingCode}
                className="fluent-btn-subtle px-3 py-1.5 text-xs text-slate-300 flex items-center gap-1.5 hover:text-white"
              >
                {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copied ? 'Copied' : 'Copy JSON'}</span>
              </button>
            </div>
          </div>

          {/* Step-by-Step Instructions */}
          <div className="fluent-card p-6 rounded-2xl space-y-4">
            <h3 className="text-sm font-semibold text-slate-100 flex items-center gap-2">
              <span>Pairing Instructions</span>
            </h3>

            {activeMode === 'hotspot' && (
              <div className="space-y-3 text-xs text-slate-300">
                <p className="text-slate-400">
                  Zero external router needed. Ideal for travel, coffee shops, or congested networks.
                </p>
                <ol className="space-y-2.5 list-decimal list-inside text-slate-300">
                  <li className="flex gap-2">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">1</span>
                    <span>Turn on <strong>Mobile Hotspot</strong> on your Android phone.</span>
                  </li>
                  <li className="flex gap-2">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">2</span>
                    <span>Connect your Windows PC Wi-Fi to your phone's hotspot.</span>
                  </li>
                  <li className="flex gap-2">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">3</span>
                    <span>Open LinkCam on Android and point camera at the QR code.</span>
                  </li>
                </ol>
              </div>
            )}

            {activeMode === 'wifi' && (
              <div className="space-y-3 text-xs text-slate-300">
                <p className="text-slate-400">
                  Connect both devices to the same local Wi-Fi router (5 GHz recommended for optimal latency).
                </p>
                <ol className="space-y-2.5 list-decimal list-inside text-slate-300">
                  <li className="flex gap-2">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">1</span>
                    <span>Ensure PC and Android phone are on the same Wi-Fi network.</span>
                  </li>
                  <li className="flex gap-2">
                    <span className="font-mono text-blue-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">2</span>
                    <span>Open LinkCam on your phone and point the camera at the QR code.</span>
                  </li>
                </ol>
              </div>
            )}

            {/* Quick Demo Simulator Button (Dev Mode Only - Tree-shaken in production builds) */}
            {import.meta.env.DEV && onSimulateConnect && (
              <div className="pt-4 border-t border-white/[0.08]">
                <button
                  onClick={onSimulateConnect}
                  className="fluent-btn-primary w-full py-2.5 px-4 text-xs font-semibold flex items-center justify-center gap-2 shadow-lg"
                >
                  <Zap className="w-4 h-4" />
                  <span>Simulate Phone Connected (Preview Mode)</span>
                </button>
              </div>
            )}
          </div>
        </div>
      ) : (
        /* USB Cable Mode Instructions & Device Manager */
        <div className="fluent-card p-6 rounded-2xl space-y-6">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-amber-500/20 text-amber-400 flex items-center justify-center">
              <Usb className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-slate-100">Wired USB Cable Tethering (15–30ms Ultra-Low Latency)</h3>
              <p className="text-xs text-slate-400">Highest bandwidth, zero Wi-Fi interference, and charges your phone simultaneously.</p>
            </div>
          </div>

          {/* Quick Manual PIN Pairing Card */}
          <div className="p-4 bg-blue-950/40 rounded-xl border border-blue-500/30 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
            <div>
              <div className="text-xs font-semibold text-blue-200">Desktop Pairing PIN:</div>
              <div className="text-[11px] text-slate-400">Enter this PIN on your phone under <strong>Manual / USB</strong> &rarr; <strong>Connect via USB</strong>.</div>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-base font-mono font-bold text-blue-300 bg-black/60 px-3 py-1.5 rounded-lg border border-blue-500/40 tracking-wider">
                {verificationPin}
              </span>
              <button
                onClick={copyPin}
                className="fluent-btn-subtle px-3 py-1.5 text-xs text-blue-300 flex items-center gap-1 hover:text-white border border-blue-500/20 rounded-lg"
              >
                {pinCopied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{pinCopied ? 'Copied' : 'Copy PIN'}</span>
              </button>
            </div>
          </div>

          {/* Connected ADB Device Auto-Detection */}
          <div className="p-4 bg-slate-950/70 rounded-xl border border-white/[0.08] space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-200 flex items-center gap-2">
                <Smartphone className="w-4 h-4 text-emerald-400" />
                <span>Detected USB Devices (ADB)</span>
              </span>
              <div className="flex items-center gap-2">
                <button
                  onClick={checkAdbNow}
                  className="fluent-btn-subtle px-2 py-1 text-[11px] text-slate-300 flex items-center gap-1 hover:text-white"
                  title="Scan for connected ADB devices now"
                >
                  <RefreshCw className="w-3 h-3 text-emerald-400" />
                  <span>Scan Now</span>
                </button>
                <span className="text-[11px] text-slate-400 hidden sm:inline">&bull; Auto-checking every 3s</span>
              </div>
            </div>

            {adbDevices.length > 0 ? (
              <div className="space-y-2">
                {adbDevices.map((dev, idx) => (
                  <div key={idx} className="flex items-center justify-between p-3 bg-black/40 rounded-lg border border-white/[0.06]">
                    <div>
                      <div className="text-xs font-semibold text-white">{String(dev.model)}</div>
                      <div className="text-[11px] font-mono text-slate-400">Serial: {String(dev.serial)} &bull; State: {String(dev.status)}</div>
                    </div>
                    <button
                      onClick={() => handleTrustAndPair(String(dev.serial))}
                      disabled={isPushingToken || String(dev.status) !== 'device'}
                      className="fluent-btn-primary px-3 py-1.5 text-xs font-medium flex items-center gap-1.5 disabled:opacity-50"
                    >
                      <span>{dev.is_trusted ? 'Push One-Time Token' : 'Trust & Push Token'}</span>
                      <ArrowRight className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="p-3 bg-black/30 rounded-lg border border-white/[0.04] space-y-3">
                <div className="text-xs text-slate-400">
                  No ADB device recognized automatically yet. Plug in your USB cable and ensure USB Debugging is allowed.
                </div>
                <div className="flex flex-wrap gap-2">
                  <button
                    onClick={handleManualUsbPair}
                    disabled={isPushingToken}
                    className="fluent-btn-primary px-3 py-1.5 text-xs font-medium flex items-center gap-1.5 disabled:opacity-50"
                  >
                    <Zap className="w-3.5 h-3.5 text-amber-300" />
                    <span>Push Token to Any Connected Phone</span>
                  </button>
                </div>
              </div>
            )}

            {usbStatusMsg && (
              <div className="text-xs font-medium text-amber-300 bg-amber-950/40 border border-amber-500/20 p-2.5 rounded-lg flex items-center gap-2">
                <span>{usbStatusMsg}</span>
              </div>
            )}
          </div>

          <ol className="space-y-3 text-xs text-slate-300">
            <li className="flex gap-3">
              <span className="font-mono text-amber-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">1</span>
              <span>Connect your phone to your PC with a high-speed USB-C cable (USB 3.0 recommended).</span>
            </li>
            <li className="flex gap-3">
              <span className="font-mono text-amber-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">2</span>
              <span>Enable <strong>USB Debugging</strong> in your phone's Developer Options and accept the <em>"Allow USB debugging from this computer?"</em> prompt on your phone screen.</span>
            </li>
            <li className="flex gap-3">
              <span className="font-mono text-amber-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">3</span>
              <span>
                If your phone appears in the box above, click <strong>Trust & Push Token</strong>. Otherwise, enter the <strong>Desktop PIN</strong> above on your phone, or manually reverse TCP port 8443:
                <div className="mt-2 space-y-2">
                  <div className="flex flex-wrap items-center gap-2">
                    <code className="bg-black/50 border border-white/[0.1] px-3 py-1.5 rounded-lg text-[11px] font-mono text-amber-300">
                      adb reverse tcp:8443 tcp:8443
                    </code>
                    <button
                      onClick={() => {
                        navigator.clipboard.writeText('adb reverse tcp:8443 tcp:8443');
                        setCopied(true);
                        setTimeout(() => setCopied(false), 2000);
                      }}
                      className="fluent-btn-subtle px-2.5 py-1.5 text-xs text-slate-300 flex items-center gap-1"
                    >
                      <Copy className="w-3.5 h-3.5 text-amber-400" />
                      <span>Copy Command</span>
                    </button>
                  </div>
                  <div className="flex flex-wrap items-center gap-2">
                    <code className="bg-black/50 border border-white/[0.1] px-3 py-1.5 rounded-lg text-[10px] font-mono text-slate-400">
                      &amp; "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe" reverse tcp:8443 tcp:8443
                    </code>
                    <button
                      onClick={() => {
                        navigator.clipboard.writeText('& "$env:LOCALAPPDATA\\Android\\Sdk\\platform-tools\\adb.exe" reverse tcp:8443 tcp:8443');
                        setCopied(true);
                        setTimeout(() => setCopied(false), 2000);
                      }}
                      className="fluent-btn-subtle px-2.5 py-1.5 text-xs text-blue-400 flex items-center gap-1 shrink-0"
                    >
                      <Copy className="w-3.5 h-3.5 text-blue-400" />
                      <span>Copy If ADB Not In PATH</span>
                    </button>
                  </div>
                </div>
                <p className="mt-1 text-[11px] text-slate-400">
                  Note: Port 8443 forwards the multiplexed TCP video and control stream. ADB reverse does not support UDP port 8444.
                </p>
              </span>
            </li>
            <li className="flex gap-3">
              <span className="font-mono text-amber-400 bg-white/[0.06] w-5 h-5 rounded-full flex items-center justify-center shrink-0">4</span>
              <span>The phone will automatically receive the one-time USB pairing key and establish the connection.</span>
            </li>
          </ol>
        </div>
      )}

      {/* Device Security & Pairing Policy */}
      <div className="fluent-card p-6 rounded-2xl space-y-5 border border-white/[0.08]">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-purple-600/20 text-purple-400 flex items-center justify-center">
              <ShieldCheck className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-slate-100">Pairing Security & Verification</h3>
              <p className="text-xs text-slate-400">Zero static loopback bypass. Devices require dynamic tokens or mutual SAS PIN.</p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-xs text-slate-400">Active Pairing PIN:</span>
            <span className="px-3 py-1 bg-slate-950 border border-blue-500/40 rounded-lg text-sm font-mono font-bold text-blue-400">
              {verificationPin}
            </span>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
          <div className="p-4 bg-slate-950/60 rounded-xl border border-white/[0.06] space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-200">Mutual SAS Verification PIN</span>
              <span className="text-[11px] px-2 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20 font-medium">Enforced</span>
            </div>
            <p className="text-[11px] text-slate-400">
              Protects against rogue devices on local networks by requiring matching 6-digit PIN confirmation.
            </p>
          </div>

          <div className="p-4 bg-slate-950/60 rounded-xl border border-white/[0.06] space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-200">Hardware Serial Binding</span>
              <span className="text-[11px] px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">Bound to Serial</span>
            </div>
            <p className="text-[11px] text-slate-400">
              USB connections are explicitly bound to ADB hardware serials with one-time session tokens.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
