"use client";

import React, {useState} from "react";
import { ExpenseDTO } from "@./expense-tracker-client";
import { Collapse } from "../collapse";
import { UserLite } from "./types";
import { ExpenseSplitDetails } from "./expense-split-details";

export { type UserLite };

interface ExpensesTableProps {
  expenses: ExpenseDTO[];
  users: UserLite[];
  potId: number;
  isArchived?: boolean;
}

export function ExpensesTable({ expenses, users, potId, isArchived }: ExpensesTableProps) {
  const [expandedId, setExpandedId] = useState<number | null>(null);

  const toggle = (id: number) => {
    setExpandedId((prev) => (prev === id ? null : id));
  };

  return (
    <div className="overflow-x-auto rounded-sm border border-gray-200 dark:border-gray-800">
      <table className="min-w-full table-auto text-left border-collapse">
        <thead className="bg-gray-50 text-gray-600 dark:bg-gray-800 dark:text-gray-300">
          <tr>
            <th className="px-4 py-2">Description</th>
            <th className="px-4 py-2">Amount</th>
            <th className="px-4 py-2">Owner</th>
            <th className="px-4 py-2">Paid</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
          {expenses.map((e) => {
            const paid = e.splits?.filter((s) => s.is_paid).length ?? 0;
            const total = e.splits?.length ?? 0;
            const currencySymbol = e.currency?.symbol ?? "";
            const amount = `${currencySymbol}${e.sum.toFixed(2)}`;
            const totalAmount = `${currencySymbol}${e.total_amount.toFixed(2)}`;
            const ownerName = users.find(u => u.uuid == e.owner_id)?.name ?? e.owner_id;

            const isOpen = expandedId === e.id;

            return (
              <React.Fragment key={e.id}>
                <tr
                  onClick={() => toggle(e.id)}
                  aria-expanded={isOpen}
                  className="cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800"
                >
                  <td className="px-4 py-2">{e.description}</td>

                  {e.sum >= 0 ? (
                    <td className="px-4 py-2 whitespace-nowrap">{amount} ({totalAmount})</td>
                    ) : (
                      <td className="px-4 py-2 whitespace-nowrap text-red-600">{amount} ({totalAmount})</td>
                    )
                  }
                  <td className="px-4 py-2 text-sm text-gray-600 dark:text-gray-300">{ownerName}</td>
                  <td className="px-4 py-2 text-sm text-gray-600 dark:text-gray-300">{paid}/{total}</td>
                </tr>

                {/* Animated details row */}
                <tr className="bg-white dark:bg-gray-900">
                  <td colSpan={4} className="p-0">
                    <Collapse isOpen={isOpen} className="border-t border-gray-100 dark:border-gray-800">
                      <ExpenseSplitDetails
                        splits={e.splits ?? []}
                        users={users}
                        currencySymbol={currencySymbol}
                        expenseId={e.id}
                        expenseOwnerId={e.owner_id}
                        potId={potId}
                        isArchived={isArchived}
                      />
                    </Collapse>
                  </td>
                </tr>
              </React.Fragment>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
