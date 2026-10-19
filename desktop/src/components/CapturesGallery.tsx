import React from 'react';
import { Copy, FileText, FolderOpen, Image as ImageIcon, Trash2 } from 'lucide-react';
import { DeliveredCapture } from './QuickCaptureTray';

interface CapturesGalleryProps {
  captures: DeliveredCapture[];
  onOpenFolder: (path: string) => void;
  onCopyPath: (path: string) => void;
}

export const CapturesGallery: React.FC<CapturesGalleryProps> = ({
  captures,
  onOpenFolder,
  onCopyPath,
}) => {
  return (
    <div className="flex-1 h-full p-8 overflow-y-auto max-w-5xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-slate-100 tracking-tight">Instant Captures Gallery</h2>
          <p className="text-sm text-slate-400 mt-1">
            Photos and auto-cropped document scans instantly delivered from your Android phone.
          </p>
        </div>

        <button
          onClick={() => onOpenFolder('')}
          className="fluent-btn-subtle px-3.5 py-2 text-xs flex items-center gap-2"
        >
          <FolderOpen className="w-4 h-4 text-blue-400" />
          <span>Open LinkCam Folder</span>
        </button>
      </div>

      {captures.length === 0 ? (
        <div className="fluent-card p-12 rounded-2xl text-center space-y-3">
          <div className="w-12 h-12 rounded-xl bg-white/[0.04] border border-white/[0.06] flex items-center justify-center mx-auto text-slate-400">
            <FileText className="w-6 h-6" />
          </div>
          <h3 className="text-sm font-semibold text-slate-200">No Captures Yet</h3>
          <p className="text-xs text-slate-400 max-w-sm mx-auto">
            Use the "Scan Document" or "Snap Photo" button in Live Studio, or open the LinkCam mobile app to send your first capture.
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {captures.map((capture) => (
            <div
              key={capture.id}
              className="fluent-card p-4 rounded-xl space-y-3 hover:border-blue-500/30 transition-all group"
            >
              <div className="aspect-[4/3] rounded-lg bg-black/40 border border-white/[0.06] flex flex-col items-center justify-center relative overflow-hidden group-hover:bg-black/50 transition-colors">
                {capture.isDocument ? (
                  <FileText className="w-12 h-12 text-blue-400" />
                ) : (
                  <ImageIcon className="w-12 h-12 text-indigo-400" />
                )}
                <span className="text-[10px] uppercase font-mono text-slate-400 mt-2">
                  {capture.isDocument ? 'PDF Document' : 'JPEG Photo'}
                </span>

                <div className="absolute top-2 right-2 bg-black/60 px-2 py-0.5 rounded text-[10px] text-slate-300 font-mono">
                  {(capture.fileSize / 1024).toFixed(0)} KB
                </div>
              </div>

              <div>
                <h4 className="text-xs font-semibold text-slate-200 truncate">{capture.fileName}</h4>
                <p className="text-[10px] text-slate-400 font-mono mt-0.5">{capture.timestamp}</p>
              </div>

              <div className="flex items-center gap-2 pt-2 border-t border-white/[0.06]">
                <button
                  onClick={() => onCopyPath(capture.filePath)}
                  className="flex-1 fluent-btn-subtle py-1.5 px-2 text-xs flex items-center justify-center gap-1.5"
                >
                  <Copy className="w-3 h-3 text-slate-400" />
                  <span>Copy</span>
                </button>

                <button
                  onClick={() => onOpenFolder(capture.filePath)}
                  className="fluent-btn-primary py-1.5 px-3 text-xs flex items-center justify-center gap-1"
                >
                  <FolderOpen className="w-3 h-3 text-white" />
                  <span>Open</span>
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
