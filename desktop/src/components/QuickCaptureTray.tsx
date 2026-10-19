import React from 'react';
import { Check, Copy, FileText, FolderOpen, Image as ImageIcon, X } from 'lucide-react';

export interface DeliveredCapture {
  id: string;
  fileName: string;
  mimeType: string;
  filePath: string;
  fileSize: number;
  timestamp: string;
  isDocument: boolean;
}

interface QuickCaptureTrayProps {
  latestCapture: DeliveredCapture | null;
  onDismiss: () => void;
  onOpenFolder: (path: string) => void;
  onCopyPath: (path: string) => void;
}

export const QuickCaptureTray: React.FC<QuickCaptureTrayProps> = ({
  latestCapture,
  onDismiss,
  onOpenFolder,
  onCopyPath,
}) => {
  if (!latestCapture) return null;

  return (
    <div className="fixed bottom-6 right-6 z-50 animate-in slide-in-from-bottom-5 duration-200">
      <div className="fluent-acrylic w-96 p-4 rounded-2xl shadow-2xl border border-blue-500/30 bg-slate-900/90 backdrop-blur-2xl space-y-3">
        {/* Banner Header */}
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-2 font-medium text-blue-400">
            <span className="w-2 h-2 rounded-full bg-blue-400 animate-ping" />
            <span>Instant Capture Delivered</span>
          </div>
          <button
            onClick={onDismiss}
            className="p-1 rounded-md text-slate-400 hover:text-slate-200 hover:bg-white/[0.06]"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Thumbnail & File Details */}
        <div className="flex items-center gap-3 bg-black/30 p-2.5 rounded-xl border border-white/[0.06]">
          <div className="w-12 h-14 rounded-lg bg-blue-950/60 border border-blue-500/20 flex flex-col items-center justify-center shrink-0">
            {latestCapture.isDocument ? (
              <FileText className="w-6 h-6 text-blue-400" />
            ) : (
              <ImageIcon className="w-6 h-6 text-indigo-400" />
            )}
            <span className="text-[9px] uppercase font-mono text-slate-400 mt-1">
              {latestCapture.isDocument ? 'PDF' : 'JPG'}
            </span>
          </div>

          <div className="flex-1 min-w-0">
            <h4 className="text-xs font-semibold text-slate-100 truncate">{latestCapture.fileName}</h4>
            <p className="text-[11px] text-slate-400 mt-0.5">
              {(latestCapture.fileSize / 1024).toFixed(1)} KB • {latestCapture.isDocument ? 'Auto-cropped document' : 'Full-res photo'}
            </p>
            <div className="flex items-center gap-1 mt-1 text-[10px] text-emerald-400 font-medium">
              <Check className="w-3 h-3" />
              <span>Copied to Windows Clipboard</span>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="grid grid-cols-2 gap-2 pt-1">
          <button
            onClick={() => onCopyPath(latestCapture.filePath)}
            className="fluent-btn-subtle py-2 px-3 text-xs flex items-center justify-center gap-1.5 hover:bg-white/10"
          >
            <Copy className="w-3.5 h-3.5 text-slate-400" />
            <span>Copy Path</span>
          </button>

          <button
            onClick={() => onOpenFolder(latestCapture.filePath)}
            className="fluent-btn-primary py-2 px-3 text-xs flex items-center justify-center gap-1.5"
          >
            <FolderOpen className="w-3.5 h-3.5 text-white" />
            <span>Show in Folder</span>
          </button>
        </div>
      </div>
    </div>
  );
};
