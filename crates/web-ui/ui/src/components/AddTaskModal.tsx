import React, { useState } from 'react';
import { Loader2, X } from 'lucide-react';
import { api } from '../api/client';
import type { CreateTaskRequest } from '../types/api';

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export const AddTaskModal: React.FC<Props> = ({ isOpen, onClose, onSuccess }) => {
  const [url, setUrl] = useState('');
  const [loading, setLoading] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      const payload: CreateTaskRequest = {
        uri: url,
        priority: 'Normal',
        creation_timestamp: new Date().toISOString(),
      };
      await api.post('/task', payload);
      setUrl('');
      onSuccess();
      onClose();
    } catch (err) {
      alert('Failed to create task');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-2xl shadow-2xl w-full max-w-md animate-fade-in overflow-hidden">
        <div className="flex justify-between items-center p-6 border-b border-slate-100">
          <h3 className="text-xl font-bold text-slate-800">New Download</h3>
          <button onClick={onClose} className="text-slate-400 hover:text-slate-600"><X size={20} /></button>
        </div>
        <form onSubmit={handleSubmit} className="p-6 space-y-5">
          <div>
            <label className="block text-sm font-semibold text-slate-700 mb-2">Resource URL</label>
            <input
              required
              autoFocus
              type="url"
              className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-blue-500 outline-none transition-all"
              placeholder="https://example.com/data.zip"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
            />
          </div>
          <div className="flex gap-3 pt-2">
            <button type="button" onClick={onClose} className="flex-1 py-3 font-bold text-slate-500 hover:bg-slate-50 rounded-xl transition-colors">Cancel</button>
            <button
              type="submit"
              disabled={loading}
              className="flex-1 py-3 bg-blue-600 text-white rounded-xl font-bold shadow-lg shadow-blue-200 disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {loading && <Loader2 size={18} className="animate-spin" />}
              Download
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
