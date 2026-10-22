import React, { useEffect, useState } from 'react';
import { ActiveTab, Sidebar } from './components/Sidebar';
import { LiveStudio } from './components/LiveStudio';
import { PairingWizard } from './components/PairingWizard';
import { CapturesGallery } from './components/CapturesGallery';
import { SettingsView } from './components/SettingsView';
import { DeliveredCapture, QuickCaptureTray } from './components/QuickCaptureTray';
import { PairingVerificationModal, PendingPairingRequest } from './components/PairingVerificationModal';
import { DiagnosticsModal } from './components/DiagnosticsModal';
import { invokeTauri } from './tauriApi';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<ActiveTab>('pairing');
  const [isConnected, setIsConnected] = useState(false);
  const [deviceName, setDeviceName] = useState('No Device Connected');
  const [batteryPct, setBatteryPct] = useState(0);
  const [isCharging, setIsCharging] = useState(false);
  const [fps, setFps] = useState(0);
  const [bitrateKbps, setBitrateKbps] = useState(0);
  const [latencyMs, setLatencyMs] = useState(0);
  const [connectionMode, setConnectionMode] = useState<'Usb' | 'DirectHotspot' | 'WiFi'>('WiFi');
  const [hasAudio, setHasAudio] = useState(false);
  const [audioLevelRms, setAudioLevelRms] = useState(0);

  // Real Pairing State from Rust Backend
  const [serverPort, setServerPort] = useState(8443);
  const [availableIps, setAvailableIps] = useState<string[]>(['127.0.0.1']);
  const [sessionToken, setSessionToken] = useState('lnk_tok_initializing');
  const [sessionId, setSessionId] = useState('sess_init');
  const [verificationPin, setVerificationPin] = useState('--- ---');

  const [latestCapture, setLatestCapture] = useState<DeliveredCapture | null>(null);
  const [pendingPairing, setPendingPairing] = useState<PendingPairingRequest | null>(null);
  const [showDiagnostics, setShowDiagnostics] = useState(false);

  const [captures, setCaptures] = useState<DeliveredCapture[]>([]);

  // Fetch real pairing credentials from Rust backend
  const fetchPairingInfo = async () => {
    try {
      const payload = await invokeTauri<{
        port: number;
        ips: string[];
        auth_token: string;
        verification_pin: string;
        session_id: string;
      }>('get_pairing_payload');

      if (payload) {
        setServerPort(payload.port);
        setAvailableIps(payload.ips);
        setSessionToken(payload.auth_token);
        setVerificationPin(payload.verification_pin);
        setSessionId(payload.session_id);
      }
    } catch {
      // Graceful fallback for web preview
    }
  };

  useEffect(() => {
    fetchPairingInfo();

    // Poll real connection and telemetry status from backend every 1.5 seconds
    const statusTimer = setInterval(async () => {
      try {
        const status = await invokeTauri<{
          is_connected: boolean;
          device_name: string;
          battery_pct: number;
          is_charging: boolean;
          fps: number;
          bitrate_kbps: number;
          latency_ms: number;
          connection_mode: string;
          audio_level_rms?: number;
          has_audio?: boolean;
        }>('get_connection_status');

        if (status.is_connected) {
          setIsConnected(true);
          setDeviceName(status.device_name || 'Android Device');
          setBatteryPct(status.battery_pct);
          setIsCharging(status.is_charging);
          setFps(status.fps);
          setBitrateKbps(status.bitrate_kbps);
          setLatencyMs(status.latency_ms);
          setHasAudio(Boolean(status.has_audio));
          setAudioLevelRms(status.audio_level_rms || 0);

          const mode = status.connection_mode.toLowerCase();
          if (mode.includes('usb')) {
            setConnectionMode('Usb');
          } else if (mode.includes('hotspot')) {
            setConnectionMode('DirectHotspot');
          } else {
            setConnectionMode('WiFi');
          }
        } else {
          setIsConnected(false);
          setDeviceName('No Device Connected');
          setHasAudio(false);
          setAudioLevelRms(0);
        }
      } catch {
        // Outside Tauri
      }
    }, 1500);

    return () => clearInterval(statusTimer);
  }, []);

  const handleRefreshPairing = async () => {
    try {
      const payload = await invokeTauri<{
        port: number;
        ips: string[];
        auth_token: string;
        verification_pin: string;
        session_id: string;
      }>('rotate_credentials');

      if (payload) {
        setServerPort(payload.port);
        setAvailableIps(payload.ips);
        setSessionToken(payload.auth_token);
        setVerificationPin(payload.verification_pin);
        setSessionId(payload.session_id);
      }
    } catch {
      const randomHex = Math.random().toString(16).substring(2, 10);
      setSessionToken(`lnk_tok_${randomHex}`);
      const randPin = Math.floor(100000 + Math.random() * 900000).toString();
      setVerificationPin(`${randPin.slice(0, 3)} ${randPin.slice(3)}`);
    }
  };

  const handleTriggerPhoto = () => {
    const saveDir = localStorage.getItem('linkcam_save_destination') || 'C:\\Users\\User\\Pictures\\LinkCam';
    const newCap: DeliveredCapture = {
      id: `cap_${Date.now()}`,
      fileName: `Snap_${Date.now().toString().slice(-4)}.jpg`,
      mimeType: 'image/jpeg',
      filePath: `${saveDir}\\Snap_${Date.now().toString().slice(-4)}.jpg`,
      fileSize: 1420000,
      timestamp: 'Just now',
      isDocument: false,
    };
    setCaptures((prev) => [newCap, ...prev]);
    setLatestCapture(newCap);
  };

  const handleTriggerDocumentScan = () => {
    const saveDir = localStorage.getItem('linkcam_save_destination') || 'C:\\Users\\User\\Pictures\\LinkCam';
    const newCap: DeliveredCapture = {
      id: `cap_${Date.now()}`,
      fileName: `Document_Scan_${Date.now().toString().slice(-4)}.pdf`,
      mimeType: 'application/pdf',
      filePath: `${saveDir}\\Document_Scan_${Date.now().toString().slice(-4)}.pdf`,
      fileSize: 312000,
      timestamp: 'Just now',
      isDocument: true,
    };
    setCaptures((prev) => [newCap, ...prev]);
    setLatestCapture(newCap);
  };

  const handleCopyPath = (path: string) => {
    navigator.clipboard.writeText(path);
  };

  const handleOpenFolder = (path: string) => {
    console.log('Opening folder for:', path);
  };

  const handleAcceptPairing = (req: PendingPairingRequest, trust: boolean) => {
    console.log('Pairing accepted for', req.deviceName, 'trusted:', trust);
    setPendingPairing(null);
    setIsConnected(true);
    setActiveTab('studio');
  };

  const handleDeclinePairing = (req: PendingPairingRequest) => {
    console.log('Pairing declined for', req.deviceName);
    setPendingPairing(null);
    setIsConnected(false);
  };

  return (
    <div className="w-screen h-screen flex bg-fluent-bg text-slate-100 overflow-hidden font-sans">
      {/* Windows 11 Fluent Sidebar */}
      <Sidebar
        activeTab={activeTab}
        onSelectTab={setActiveTab}
        isConnected={isConnected}
        deviceName={deviceName}
        batteryPct={batteryPct}
        connectionMode={connectionMode}
      />

      {/* Main Surface Content */}
      <main className="flex-1 h-full overflow-hidden flex flex-col">
        {activeTab === 'studio' && (
          <LiveStudio
            isConnected={isConnected}
            fps={fps}
            bitrateKbps={bitrateKbps}
            latencyMs={latencyMs}
            batteryPct={batteryPct}
            isCharging={isCharging}
            connectionMode={connectionMode}
            isVcamActive={true}
            hasAudio={hasAudio}
            audioLevelRms={audioLevelRms}
            serverPort={serverPort}
            onTriggerPhoto={handleTriggerPhoto}
            onTriggerDocumentScan={handleTriggerDocumentScan}
            onOpenDiagnostics={() => setShowDiagnostics(true)}
          />
        )}

        {activeTab === 'captures' && (
          <CapturesGallery
            captures={captures}
            onOpenFolder={handleOpenFolder}
            onCopyPath={handleCopyPath}
          />
        )}

        {activeTab === 'pairing' && (
          <PairingWizard
            serverPort={serverPort}
            availableIps={availableIps}
            sessionToken={sessionToken}
            sessionId={sessionId}
            verificationPin={verificationPin}
            onRefreshPairing={handleRefreshPairing}
          />
        )}

        {activeTab === 'settings' && <SettingsView />}
      </main>

      {/* Interactive Floating Quick-Capture Notification Tray */}
      <QuickCaptureTray
        latestCapture={latestCapture}
        onDismiss={() => setLatestCapture(null)}
        onOpenFolder={handleOpenFolder}
        onCopyPath={handleCopyPath}
      />

      {/* Interactive Pairing Verification & Authorization Modal */}
      <PairingVerificationModal
        request={pendingPairing}
        onAccept={handleAcceptPairing}
        onDecline={handleDeclinePairing}
      />

      {/* Diagnostics Modal */}
      <DiagnosticsModal
        isOpen={showDiagnostics}
        onClose={() => setShowDiagnostics(false)}
        latencyMs={latencyMs}
        fps={fps}
        bitrateKbps={bitrateKbps}
        batteryPct={batteryPct}
      />
    </div>
  );
};

export default App;

