"use client";

import React from "react";
import {
  SplitDTO,
  usePayExpense,
  getGetPotExpensesQueryKey,
  useCurrentUser,
} from "@./expense-tracker-client";
import { useQueryClient } from "@tanstack/react-query";
import { UserLite } from "./types";

export interface ExpenseSplitDetailsProps {
  splits: SplitDTO[];
  users: UserLite[];
  currencySymbol: string;
  expenseId: number;
  expenseOwnerId: string;
  potId: number;
  isArchived?: boolean;
}

export function ExpenseSplitDetails({
  splits,
  users,
  currencySymbol,
  expenseId,
  expenseOwnerId,
  potId,
  isArchived,
}: ExpenseSplitDetailsProps) {
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
      await payExpense.mutateAsync({ expenseId });
      await queryClient.invalidateQueries({ queryKey: getGetPotExpensesQueryKey(potId) });
    } catch (err) {
      console.error("Failed to mark split as paid", err);
      // Non-intrusive: can add toast later
    }
  };

  if (!splits || splits.length === 0) {
    return (
      <div className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
        No splits available for this expense.
      </div>
    );
  }

  return (
    <div className="px-4 py-3">
      <div className="overflow-x-auto rounded-md border border-gray-100 dark:border-gray-800 overflow-hidden">
        <table className="min-w-full table-auto text-left border-collapse">
          <thead className="bg-gray-50 text-gray-600 dark:bg-gray-800 dark:text-gray-300">
            <tr>
              <th className="px-3 py-2 text-sm first:rounded-tl-md">Participant</th>
              <th className="px-3 py-2 text-sm">Amount</th>
              <th className="px-3 py-2 text-sm last:rounded-tr-md">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
            {splits.map((s, idx) => {
              const userName = users.find((u) => u.uuid == s.user_id)?.name ?? s.user_id;
              const splitAmount = `${currencySymbol}${s.amount.toFixed(2)}`;
              const allowed = !s.is_paid && canMark(s.user_id);
              return (
                <tr key={`split-${idx}`} className="bg-white dark:bg-gray-900">
                  <td className="px-3 py-2 text-sm text-gray-700 dark:text-gray-200">{userName}</td>
                  <td className="px-3 py-2 text-sm text-gray-700 dark:text-gray-200 whitespace-nowrap">
                    {splitAmount}
                  </td>
                  <td className="px-3 py-2">
                    {s.is_paid ? (
                      <span className="inline-flex items-center rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300 px-2 py-0.5 text-xs font-medium">
                        Paid
                      </span>
                    ) : allowed && !isArchived ? (
                      <button
                        onClick={(evt) => onMarkPaid(evt, s.amount)}
                        disabled={payExpense.isPending}
                        className="inline-flex items-center rounded-md bg-blue-600 text-white px-2 py-1 text-xs hover:bg-blue-700 disabled:opacity-50"
                        title="Mark this split as paid"
                      >
                        {payExpense.isPending ? "Marking…" : "Mark paid"}
                      </button>
                    ) : (
                      <span className="inline-flex items-center rounded-full bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300 px-2 py-0.5 text-xs font-medium">
                        Open
                      </span>
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
