import React from 'react';
import type { ProgressChunk, TaskState } from '../types/api';

interface Props {
  chunks: ProgressChunk[];
  contentLength: number;
  state: TaskState;
}

export const ProgressBar: React.FC<Props> = ({ chunks, contentLength, state }) => {
  const totalReceived = chunks.reduce((acc, chunk) => acc + chunk.received, 0);
  const percentage = contentLength > 0 ? Math.min(Math.round((totalReceived / contentLength) * 100), 100) : 0;

  const colors: Record<TaskState, string> = {
    Downloading: 'bg-blue-500',
    Paused: 'bg-amber-500',
    Completed: 'bg-emerald-500',
    Failed: 'bg-red-500',
    Pending: 'bg-slate-400',
    Canceled: 'bg-slate-300'
  };

  return (
    <div className="w-full">
      <div className="flex justify-between mb-1 text-[10px] font-bold text-slate-500 uppercase">
        <span>{percentage}%</span>
        <span>{state}</span>
      </div>
      <div className="w-full bg-slate-200 rounded-full h-1.5 overflow-hidden">
        <div
          className={`h-full transition-all duration-700 ease-out ${colors[state]}`}
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
};
