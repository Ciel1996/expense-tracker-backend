'use client';

import {useGetPots} from "@./expense-tracker-client";
import {Pot} from "./pot";

export function PotsOverview() {
  const { data: pots } = useGetPots();

  if (!pots) {
    return <div>Loading...</div>;
  }

  return(
    <div className="columns-1 sm:columns-2 md:columns-3 lg:columns-4 xl:columns-5 2xl:columns-6 gap-3">
      {pots.map(e => (
        <div key={`pot-${e.id}`} className="break-inside-avoid mb-3">
          <Pot
            name={e.name}
            currency={e.default_currency.symbol}
            owner={e.users.find(u => u.uuid == e.owner_id)?.name ?? 'Unknown'}
            users={e.users.map(u => u.name)}
          />
        </div>
      ))}
    </div>
  )
}
