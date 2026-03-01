import { useState, useEffect, useCallback } from 'react';
import { Trash2, Pause, Play, Loader2, Search, Bell, Filter } from 'lucide-react';
import { api } from './api/client';
import type { TaskStatus } from './types/api';
import { Sidebar } from './components/Sidebar';
import { ProgressBar } from './components/ProgressBar';
import { AddTaskModal } from './components/AddTaskModal';

export default function App() {
  // 狀態管理
  const [tasks, setTasks] = useState<TaskStatus[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [version, setVersion] = useState('0.3.8');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [initialLoading, setInitialLoading] = useState(true);

  // 1. 獲取任務列表
  const fetchTasks = useCallback(async () => {
    try {
      const { data } = await api.get<TaskStatus[]>('/task');
      setTasks(data);
      setInitialLoading(false);
    } catch (e) {
      console.error("Fetch error:", e);
    }
  }, []);

  // 2. 生命週期與輪詢
  useEffect(() => {
    // 獲取版本
    api.get('/system/version')
      .then(res => setVersion(res.data))
      .catch(() => setVersion('0.3.8'));

    fetchTasks();
    const interval = setInterval(fetchTasks, 3000);
    return () => clearInterval(interval);
  }, [fetchTasks]);

  // 3. 單選邏輯
  const toggleSelect = (id: number) => {
    const next = new Set(selectedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSelectedIds(next);
  };

  // 4. 批量操作邏輯
  const handleBulk = async (action: 'remove' | 'pause' | 'resume') => {
    try {
      await Promise.all(
        Array.from(selectedIds).map(id =>
          action === 'remove'
            ? api.delete(`/task/remove/${id}`)
            : api.post(`/task/${action}/${id}`)
        )
      );
      setSelectedIds(new Set());
      fetchTasks();
    } catch (e) {
      alert(`Operation ${action} failed`);
    }
  };

  return (
    <div className="flex min-h-screen bg-[#F8FAFC]">
      <Sidebar version={version} onAddTask={() => setIsModalOpen(true)} />

      <main className="flex-1 p-6 lg:p-10">
        <div className="max-w-7xl mx-auto">

          {/* Header */}
          <header className="flex justify-between items-center mb-10">
            <div className="space-y-1">
              <h2 className="text-3xl font-black text-slate-900 tracking-tight">
                Downloads <span className="text-blue-600">.</span>
              </h2>
              <div className="flex items-center gap-2 text-slate-400 text-sm font-medium">
                {!initialLoading && (
                  <>
                    <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                    {tasks.length} Active System Tasks
                  </>
                )}
              </div>
            </div>

            <div className="flex items-center gap-4">
              <div className="hidden md:flex items-center bg-white border border-slate-200 rounded-2xl px-4 py-2 shadow-sm focus-within:ring-2 focus-within:ring-blue-500/20 transition-all">
                <Search size={18} className="text-slate-400" />
                <input type="text" placeholder="Search tasks..." className="bg-transparent border-none outline-none ml-2 text-sm w-48" />
              </div>
              <button className="p-2.5 bg-white border border-slate-200 rounded-2xl text-slate-400 hover:text-blue-600 hover:shadow-md transition-all">
                <Bell size={20} />
              </button>
            </div>
          </header>

          {/* 批量操作浮動條 */}
          {selectedIds.size > 0 && (
            <div className="fixed bottom-8 left-1/2 -translate-x-1/2 flex items-center gap-4 bg-slate-900/90 backdrop-blur-xl px-6 py-4 rounded-3xl shadow-2xl z-40 border border-white/10 animate-in slide-in-from-bottom-8">
              <span className="text-white text-sm font-bold border-r border-white/20 pr-4">
                {selectedIds.size} Selected
              </span>
              <div className="flex gap-2">
                <button onClick={() => handleBulk('resume')} className="p-2 text-emerald-400 hover:bg-white/10 rounded-xl transition-colors"><Play size={20} fill="currentColor" /></button>
                <button onClick={() => handleBulk('pause')} className="p-2 text-amber-400 hover:bg-white/10 rounded-xl transition-colors"><Pause size={20} fill="currentColor" /></button>
                <button onClick={() => handleBulk('remove')} className="p-2 text-red-400 hover:bg-white/10 rounded-xl transition-colors"><Trash2 size={20} /></button>
              </div>
            </div>
          )}

          {/* 任務表格 */}
          <div className="bg-white rounded-[2.5rem] border border-slate-200 shadow-[0_8px_30px_rgb(0,0,0,0.04)] overflow-hidden">
            <div className="px-8 py-6 border-b border-slate-50 flex justify-between items-center bg-slate-50/30">
              <h3 className="font-bold text-slate-800 flex items-center gap-2 text-sm uppercase tracking-wider">
                <Filter size={16} /> Task Queue
              </h3>
            </div>

            <table className="w-full text-left">
              <thead>
                <tr className="text-slate-400 text-[11px] font-bold uppercase tracking-[0.15em]">
                  <th className="px-8 py-4 w-12">
                    <input
                      type="checkbox"
                      className="w-5 h-5 rounded-lg border-slate-300 text-blue-600 focus:ring-blue-500/20"
                      onChange={(e) => setSelectedIds(e.target.checked ? new Set(tasks.map(t => t.id)) : new Set())}
                      checked={selectedIds.size === tasks.length && tasks.length > 0}
                    />
                  </th>
                  <th className="px-6 py-4">Resource</th>
                  <th className="px-6 py-4">Status</th>
                  <th className="px-8 py-4 text-right">Action</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50">
                {initialLoading ? (
                  <tr>
                    <td colSpan={4} className="px-8 py-20 text-center">
                      <div className="flex flex-col items-center gap-3">
                        <Loader2 className="animate-spin text-blue-600" size={32} />
                        <span className="text-slate-400 font-medium text-sm">Synchronizing tasks...</span>
                      </div>
                    </td>
                  </tr>
                ) : tasks.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-8 py-20 text-center text-slate-400 font-medium italic">
                      Queue is empty. Start a new download!
                    </td>
                  </tr>
                ) : tasks.map(task => (
                  <tr key={task.id} className={`group transition-all duration-300 ${selectedIds.has(task.id) ? 'bg-blue-50/50' : 'hover:bg-slate-50/50'}`}>
                    <td className="px-8 py-7">
                      <input
                        type="checkbox"
                        className="w-5 h-5 rounded-lg border-slate-300 text-blue-600"
                        checked={selectedIds.has(task.id)}
                        onChange={() => toggleSelect(task.id)}
                      />
                    </td>
                    <td className="px-6 py-7">
                      <div className="flex flex-col gap-1">
                        <span className="font-bold text-slate-800 text-base group-hover:text-blue-600 transition-colors">
                          {task.file_path.split('/').pop()}
                        </span>
                        <span className="text-[10px] text-slate-400 font-mono opacity-0 group-hover:opacity-100 transition-opacity">
                          {task.file_path}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-7 min-w-[280px]">
                      <ProgressBar chunks={task.chunks} contentLength={task.content_length} state={task.state} />
                    </td>
                    <td className="px-8 py-7 text-right">
                      <button
                        onClick={() => api.post(`/task/${task.state === 'Paused' ? 'resume' : 'pause'}/${task.id}`)}
                        className="p-3 bg-slate-50 text-slate-400 hover:text-blue-600 hover:bg-white hover:shadow-md rounded-2xl transition-all"
                      >
                        {task.state === 'Paused' ? <Play size={18} fill="currentColor" /> : <Pause size={18} fill="currentColor" />}
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </main>

      <AddTaskModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSuccess={fetchTasks}
      />
    </div>
  );
}
