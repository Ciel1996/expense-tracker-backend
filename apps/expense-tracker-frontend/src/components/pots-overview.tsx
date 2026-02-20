'use client';

import { useMemo, useState } from "react";
import { useGetPots } from "@./expense-tracker-client";
import { Pot } from "./pot";
import { NewPotModal } from "./new-pot-modal";

export function PotsOverview() {
  const { data: pots } = useGetPots();
  const [open, setOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<'active' | 'archived'>('active');

  const filteredAndSortedPots = useMemo(() => {
    if (!pots) return [];

    return [...pots]
      .filter((p) => (activeTab === 'active' ? !p.archived : p.archived))
      .sort((a, b) => {
        // Sort by creation date (descending - newest first)
        const dateA = new Date(a.created_at).getTime();
        const dateB = new Date(b.created_at).getTime();
        return dateB - dateA;
      });
  }, [pots, activeTab]);

  if (!pots) {
    return <div>Loading...</div>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between border-b border-gray-200 dark:border-gray-700">
        <div className="flex gap-4">
          <button
            onClick={() => setActiveTab('active')}
            className={`px-1 py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'active'
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Active
          </button>
          <button
            onClick={() => setActiveTab('archived')}
            className={`px-1 py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'archived'
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Archived
          </button>
        </div>
        {activeTab === 'active' && (
          <button
            onClick={() => setOpen(true)}
            className="px-4 py-2 text-sm rounded-md bg-blue-600 text-white hover:bg-blue-700"
          >
            New Pot
          </button>
        )}
      </div>

      {filteredAndSortedPots.length > 0 ? (
        <div className="columns-1 sm:columns-2 md:columns-[16rem] gap-1">
          {filteredAndSortedPots.map((e) => (
            <div key={`pot-${e.id}`} className="break-inside-avoid mb-2">
              <Pot
                name={e.name}
                currency={e.default_currency.symbol}
                owner={e.users.find((u) => u.uuid == e.owner_id)?.name ?? 'Unknown'}
                users={e.users.map((u) => u.name)}
                balance={e.net_balance}
                href={`/pots/${e.id}`}
                isArchived={e.archived}
              />
            </div>
          ))}
        </div>
      ) : (
        <div className="py-12 text-center text-gray-500">
          No {activeTab} pots found.
        </div>
      )}

      <NewPotModal open={open} onClose={() => setOpen(false)} />
    </div>
  );
}
