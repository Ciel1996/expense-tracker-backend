"use client";

import React, {useState} from "react";
import { ExpenseDTO, SplitDTO, usePayExpense, getGetPotExpensesQueryKey, useCurrentUser } from "@./expense-tracker-client";
import { Collapse } from "../collapse";
import { useQueryClient } from "@tanstack/react-query";

export interface UserLite { uuid: string; name: string }

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
    <div className="overflow-x-auto rounded border border-gray-200 dark:border-gray-800">
      <table className="min-w-full table-auto text-left">
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
                    <td className="px-4 py-2 whitespace-nowrap">{amount}</td>
                    ) : (
                      <td className="px-4 py-2 whitespace-nowrap text-red-600">{amount}</td>
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

function ExpenseSplitDetails({ splits, users, currencySymbol, expenseId, expenseOwnerId, potId, isArchived }: { splits: SplitDTO[]; users: UserLite[]; currencySymbol: string; expenseId: number; expenseOwnerId: string; potId: number; isArchived?: boolean }) {
  const { data: currentUser } = useCurrentUser();
  const payExpense = usePayExpense();
  const queryClient = useQueryClient();

  const canMark = (splitUserId: string) => {
    if (!currentUser) return false;
    return currentUser.uuid === splitUserId || currentUser.uuid === expenseOwnerId;
  };

  const onMarkPaid = async (e: React.MouseEvent, amount: number) => {
    e.stopPropagation();
    try {
      await payExpense.mutateAsync({ expenseId, data: { sum_paid: amount } });
      await queryClient.invalidateQueries({ queryKey: getGetPotExpensesQueryKey(potId) });
    } catch (err) {
      console.error('Failed to mark split as paid', err);
      // Non-intrusive: can add toast later
    }
  };

  if (!splits || splits.length === 0) {
    return (
      <div className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">No splits available for this expense.</div>
    );
  }

  return (
    <div className="px-4 py-3">
      <div className="overflow-x-auto rounded-md border border-gray-100 dark:border-gray-800 overflow-hidden">
        <table className="min-w-full table-auto text-left">
          <thead className="bg-gray-50 text-gray-600 dark:bg-gray-800 dark:text-gray-300">
            <tr>
              <th className="px-3 py-2 text-sm first:rounded-tl-md">Participant</th>
              <th className="px-3 py-2 text-sm">Amount</th>
              <th className="px-3 py-2 text-sm last:rounded-tr-md">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
            {splits.map((s, idx) => {
              const userName = users.find(u => u.uuid == s.user_id)?.name ?? s.user_id;
              const splitAmount = `${currencySymbol}${s.amount.toFixed(2)}`;
              const allowed = !s.is_paid && canMark(s.user_id);
              return (
                <tr key={`split-${idx}`} className="bg-white dark:bg-gray-900">
                  <td className="px-3 py-2 text-sm text-gray-700 dark:text-gray-200">{userName}</td>
                  <td className="px-3 py-2 text-sm text-gray-700 dark:text-gray-200 whitespace-nowrap">{splitAmount}</td>
                  <td className="px-3 py-2">
                    {s.is_paid ? (
                      <span className="inline-flex items-center rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300 px-2 py-0.5 text-xs font-medium">Paid</span>
                    ) : (allowed && !isArchived) ? (
                      <button
                        onClick={(evt) => onMarkPaid(evt, s.amount)}
                        disabled={payExpense.isPending}
                        className="inline-flex items-center rounded-md bg-blue-600 text-white px-2 py-1 text-xs hover:bg-blue-700 disabled:opacity-50"
                        title="Mark this split as paid"
                      >
                        {payExpense.isPending ? 'Marking…' : 'Mark paid'}
                      </button>
                    ) : (
                      <span className="inline-flex items-center rounded-full bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300 px-2 py-0.5 text-xs font-medium">Open</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
