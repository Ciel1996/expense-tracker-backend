'use client';

import {useState} from "react";
import {useGetPots} from "@./expense-tracker-client";
import {Pot} from "./pot";
import {NewPotModal} from "./new-pot-modal";

export function PotsOverview() {
  const { data: pots } = useGetPots();
  const [open, setOpen] = useState(false);

  if (!pots) {
    return <div>Loading...</div>;
  }

  return(
    <div className="space-y-3">
      <div className="flex justify-end">
        <button
          onClick={() => setOpen(true)}
          className="px-4 py-2 rounded-md bg-blue-600 text-white hover:bg-blue-700"
        >
          New Pot
        </button>
      </div>

      <div className="columns-1 sm:columns-2 md:columns-[16rem] gap-1">
        {pots.map(e => (
          <div key={`pot-${e.id}`} className="break-inside-avoid mb-2">
            <Pot
              name={e.name}
              currency={e.default_currency.symbol}
              owner={e.users.find(u => u.uuid == e.owner_id)?.name ?? 'Unknown'}
              users={e.users.map(u => u.name)}
              href={`/pots/${e.id}`}
            />
          </div>
        ))}
      </div>

      <NewPotModal open={open} onClose={() => setOpen(false)} />
    </div>
  )
}
