import React from 'react';
import { Plus, LayoutDashboard, Settings, HardDrive, Info } from 'lucide-react';

interface Props {
  onAddTask: () => void;
  version: string;
}

export const Sidebar: React.FC<Props> = ({ onAddTask, version }) => (
  <aside className="w-64 bg-slate-900 text-slate-300 flex flex-col h-screen sticky top-0 shrink-0">
    <div className="p-6 flex-1">
      <div className="flex items-center gap-3 mb-8 text-white">
        <div className="bg-blue-600 p-2 rounded-lg">
          <HardDrive size={24} />
        </div>
        <h1 className="text-xl font-bold tracking-tight text-white">Caracal</h1>
      </div>

      <button
        onClick={onAddTask}
        className="w-full flex items-center justify-center gap-2 bg-blue-600 hover:bg-blue-500 text-white py-2.5 rounded-xl transition-all font-semibold shadow-lg shadow-blue-900/40 active:scale-95"
      >
        <Plus size={18} /> Add Task
      </button>

      <nav className="mt-10 space-y-1">
        <div className="flex items-center gap-3 px-4 py-3 bg-slate-800 text-white rounded-xl cursor-pointer">
          <LayoutDashboard size={18} /> Dashboard
        </div>
        <div className="flex items-center gap-3 px-4 py-3 hover:bg-slate-800/50 hover:text-white rounded-xl transition-colors cursor-pointer text-slate-400">
          <Settings size={18} /> Settings
        </div>
      </nav>
    </div>

    <div className="p-6 border-t border-slate-800">
      <div className="flex items-center gap-2 text-[10px] font-mono text-slate-500 tracking-widest uppercase">
        <Info size={12} /> v{version}
      </div>
    </div>
  </aside>
);
