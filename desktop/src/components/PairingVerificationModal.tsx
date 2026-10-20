import React, { useState } from 'react';
import { Smartphone, ShieldCheck, Check, X, Wifi } from 'lucide-react';

export interface PendingPairingRequest {
  id: string;
  deviceName: string;
  ipAddress: string;
  connectionMode: 'DirectHotspot' | 'WiFi' | 'USB';
  verificationPin: string;
  requestedAt: string;
}

interface PairingVerificationModalProps {
  request: PendingPairingRequest | null;
  onAccept: (request: PendingPairingRequest, trustDevice: boolean) => void;
  onDecline: (request: PendingPairingRequest) => void;
}

export const PairingVerificationModal: React.FC<PairingVerificationModalProps> = ({
  request,
  onAccept,
  onDecline,
}) => {
  const [trustDevice, setTrustDevice] = useState(true);

  if (!request) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-md animate-fade-in">
      <div className="w-full max-w-md bg-slate-900 border border-white/[0.12] rounded-3xl shadow-2xl overflow-hidden p-6 space-y-6">
        {/* Header with security icon */}
        <div className="flex items-start gap-4">
          <div className="w-12 h-12 rounded-2xl bg-blue-600/20 border border-blue-500/30 text-blue-400 flex items-center justify-center shrink-0">
            <ShieldCheck className="w-6 h-6" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white tracking-tight">Pairing Authorization</h3>
            <p className="text-xs text-slate-400 mt-0.5">
              An incoming phone is requesting to connect to your PC.
            </p>
          </div>
        </div>

        {/* Device Information Card */}
        <div className="bg-slate-950/60 border border-white/[0.06] rounded-2xl p-4 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-9 h-9 rounded-xl bg-white/[0.06] text-slate-200 flex items-center justify-center">
                <Smartphone className="w-5 h-5 text-blue-400" />
              </div>
              <div>
                <div className="text-sm font-medium text-white">{request.deviceName}</div>
                <div className="text-xs text-slate-400 font-mono">{request.ipAddress}</div>
              </div>
            </div>

            <span className="text-[11px] font-medium px-2.5 py-1 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1">
              <Wifi className="w-3 h-3" />
              {request.connectionMode}
            </span>
          </div>

          {/* Verification Code Box */}
          <div className="pt-2 border-t border-white/[0.06] flex flex-col items-center justify-center">
            <span className="text-[11px] text-slate-400 uppercase tracking-wider font-medium">
              Match 6-Digit Code on Phone
            </span>
            <div className="mt-1 px-4 py-2 bg-blue-950/40 border border-blue-500/40 rounded-xl text-2xl font-mono font-bold tracking-widest text-blue-300">
              {request.verificationPin}
            </div>
            <p className="text-[11px] text-slate-500 mt-1">
              Verify that the code above matches the number shown on your phone's screen.
            </p>
          </div>
        </div>

        {/* Trust Option Checkbox */}
        <label className="flex items-start gap-3 cursor-pointer group text-xs text-slate-300">
          <input
            type="checkbox"
            checked={trustDevice}
            onChange={(e) => setTrustDevice(e.target.checked)}
            className="w-4 h-4 mt-0.5 rounded accent-blue-600 bg-slate-800 border-white/20 cursor-pointer"
          />
          <div>
            <span className="font-medium group-hover:text-white transition-colors">
              Always trust this device
            </span>
            <p className="text-[11px] text-slate-500 mt-0.5">
              Future connections from this phone will pair automatically without prompting.
            </p>
          </div>
        </label>

        {/* Action Buttons */}
        <div className="flex gap-3 pt-2">
          <button
            onClick={() => onDecline(request)}
            className="flex-1 px-4 py-3 rounded-xl border border-red-500/30 bg-red-950/30 text-red-300 hover:bg-red-900/40 hover:text-white transition-all text-xs font-semibold flex items-center justify-center gap-2"
          >
            <X className="w-4 h-4" />
            Decline
          </button>

          <button
            onClick={() => onAccept(request, trustDevice)}
            className="flex-[1.4] px-4 py-3 rounded-xl bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-600/30 transition-all text-xs font-semibold flex items-center justify-center gap-2"
          >
            <Check className="w-4 h-4" />
            Pair & Allow
          </button>
        </div>
      </div>
    </div>
  );
};
