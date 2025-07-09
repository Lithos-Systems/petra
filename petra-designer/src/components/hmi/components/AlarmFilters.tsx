// Filter controls for alarms
import React from 'react';

interface AlarmFiltersProps {
  filter: string;
  setFilter: (f: string) => void;
}

export default function AlarmFilters({ filter, setFilter }: AlarmFiltersProps) {
  return (
    <div className="alarm-filters">
      <select value={filter} onChange={e => setFilter(e.target.value)}>
        <option value="all">All</option>
        <option value="active">Active</option>
        <option value="unack">Unack</option>
      </select>
    </div>
  );
}
