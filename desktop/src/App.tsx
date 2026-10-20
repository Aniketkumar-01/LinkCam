import React, { useState } from 'react';
import { ActiveTab, Sidebar } from './components/Sidebar';
import { LiveStudio } from './components/LiveStudio';
import { PairingWizard } from './components/PairingWizard';
import { CapturesGallery } from './components/CapturesGallery';
import { SettingsView } from './components/SettingsView';
import { DeliveredCapture, QuickCaptureTray } from './components/QuickCaptureTray';
import { PairingVerificationModal, PendingPairingRequest } from './components/PairingVerificationModal';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<ActiveTab>('studio');
  const [isConnected, setIsConnected] = useState(true);
  const [sessionToken, setSessionToken] = useState('lnk_tok_7f9a2b1c4e6d8035');
  const [latestCapture, setLatestCapture] = useState<DeliveredCapture | null>(null);
  const [pendingPairing, setPendingPairing] = useState<PendingPairingRequest | null>(null);

  const [captures, setCaptures] = useState<DeliveredCapture[]>([
    {
      id: 'cap_1',
      fileName: 'Signed_Contract_Page1.pdf',
      mimeType: 'application/pdf',
      filePath: 'C:\\Users\\User\\Pictures\\LinkCam\\20261019_171500_Signed_Contract_Page1.pdf',
      fileSize: 245000,
      timestamp: 'Today at 5:15 PM',
      isDocument: true,
    },
    {
      id: 'cap_2',
      fileName: 'Whiteboard_Brainstorm.jpg',
      mimeType: 'image/jpeg',
      filePath: 'C:\\Users\\User\\Pictures\\LinkCam\\20261019_170530_Whiteboard_Brainstorm.jpg',
      fileSize: 1820000,
      timestamp: 'Today at 5:05 PM',
      isDocument: false,
    },
  ]);

  // Triggering Instant Capture (simulated from desktop or mobile trigger)
  const handleTriggerPhoto = () => {
    const newCap: DeliveredCapture = {
      id: `cap_${Date.now()}`,
      fileName: `Snap_${Date.now().toString().slice(-4)}.jpg`,
      mimeType: 'image/jpeg',
      filePath: `C:\\Users\\User\\Pictures\\LinkCam\\Snap_${Date.now().toString().slice(-4)}.jpg`,
      fileSize: 1420000,
      timestamp: 'Just now',
      isDocument: false,
    };
    setCaptures((prev) => [newCap, ...prev]);
    setLatestCapture(newCap);
  };

  const handleTriggerDocumentScan = () => {
    const newCap: DeliveredCapture = {
      id: `cap_${Date.now()}`,
      fileName: `Document_Scan_${Date.now().toString().slice(-4)}.pdf`,
      mimeType: 'application/pdf',
      filePath: `C:\\Users\\User\\Pictures\\LinkCam\\Document_Scan_${Date.now().toString().slice(-4)}.pdf`,
      fileSize: 312000,
      timestamp: 'Just now',
      isDocument: true,
    };
    setCaptures((prev) => [newCap, ...prev]);
    setLatestCapture(newCap);
  };

  const handleRefreshPairing = () => {
    const randomHex = Math.random().toString(16).substring(2, 10);
    setSessionToken(`lnk_tok_${randomHex}`);
  };

  const handleCopyPath = (path: string) => {
    navigator.clipboard.writeText(path);
  };

  const handleOpenFolder = (path: string) => {
    console.log('Opening folder for:', path);
  };

  const handleTriggerPairingPrompt = () => {
    setPendingPairing({
      id: `req_${Date.now()}`,
      deviceName: 'Pixel 8 Pro (ARM64)',
      ipAddress: '192.168.43.1',
      connectionMode: 'DirectHotspot',
      verificationPin: '582 391',
      requestedAt: 'Just now',
    });
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
        deviceName="Pixel 8 Pro (ARM64)"
        batteryPct={86}
        connectionMode="DirectHotspot"
      />

      {/* Main Surface Content */}
      <main className="flex-1 h-full overflow-hidden flex flex-col">
        {activeTab === 'studio' && (
          <LiveStudio
            isConnected={isConnected}
            fps={60.0}
            bitrateKbps={4850}
            latencyMs={38}
            batteryPct={86}
            isCharging={true}
            connectionMode="DirectHotspot"
            isVcamActive={true}
            onTriggerPhoto={handleTriggerPhoto}
            onTriggerDocumentScan={handleTriggerDocumentScan}
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
            serverPort={8443}
            availableIps={['192.168.43.1', '192.168.1.120', '127.0.0.1']}
            sessionToken={sessionToken}
            onRefreshPairing={handleRefreshPairing}
            onSimulateConnect={handleTriggerPairingPrompt}
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
    </div>
  );
};

export default App;
