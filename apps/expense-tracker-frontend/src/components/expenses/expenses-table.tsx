"use client";

import React, {useMemo, useState} from "react";
import { ExpenseDTO } from "@./expense-tracker-client";
import { Collapse } from "../collapse";
import { UserLite } from "./types";
import { ExpenseSplitDetails } from "./expense-split-details";

export { type UserLite };

type SortColumn = "date" | "description" | "amount";
type SortDirection = "ascending" | "descending";

interface ExpensesTableProps {
  expenses: ExpenseDTO[];
  users: UserLite[];
  potId: number;
  isArchived?: boolean;
}

export function ExpensesTable({ expenses, users, potId, isArchived }: ExpensesTableProps) {
  const [expandedId, setExpandedId] = useState<number | null>(null);
  const [sort, setSort] = useState<{ column: SortColumn; direction: SortDirection } | null>(null);

  const sortedExpenses = useMemo(() => {
    if (!sort) return expenses;

    return [...expenses].sort((a, b) => {
      let comparison: number;
      switch (sort.column) {
        case "date":
          comparison = new Date(a.date).getTime() - new Date(b.date).getTime();
          break;
        case "description":
          comparison = a.description.localeCompare(b.description);
          break;
        case "amount":
          comparison = a.sum - b.sum;
          break;
      }
      return sort.direction === "ascending" ? comparison : -comparison;
    });
  }, [expenses, sort]);

  const sortBy = (column: SortColumn) => {
    setSort((current) => ({
      column,
      direction: current?.column === column && current.direction === "ascending"
        ? "descending"
        : "ascending",
    }));
  };

  const sortIndicator = (column: SortColumn) =>
    sort?.column === column ? (sort.direction === "ascending" ? " \u2191" : " \u2193") : "";

  const toggle = (id: number) => {
    setExpandedId((prev) => (prev === id ? null : id));
  };

  return (
    <div className="overflow-x-auto rounded-sm border border-gray-200 dark:border-gray-800">
      <table className="min-w-full table-auto text-left border-collapse">
        <thead className="bg-gray-50 text-gray-600 dark:bg-gray-800 dark:text-gray-300">
          <tr>
            <th className="px-4 py-2 whitespace-nowrap w-px" aria-sort={sort?.column === "date" ? sort.direction : undefined}>
              <button type="button" className="cursor-pointer hover:underline focus-visible:underline" onClick={() => sortBy("date")}>Date and Time<span aria-hidden="true">{sortIndicator("date")}</span></button>
            </th>
            <th className="px-4 py-2" aria-sort={sort?.column === "description" ? sort.direction : undefined}>
              <button type="button" className="cursor-pointer hover:underline focus-visible:underline" onClick={() => sortBy("description")}>Description<span aria-hidden="true">{sortIndicator("description")}</span></button>
            </th>
            <th className="px-4 py-2" aria-sort={sort?.column === "amount" ? sort.direction : undefined}>
              <button type="button" className="cursor-pointer hover:underline focus-visible:underline" onClick={() => sortBy("amount")}>Amount<span aria-hidden="true">{sortIndicator("amount")}</span></button>
            </th>
            <th className="px-4 py-2">Owner</th>
            <th className="px-4 py-2">Paid</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
          {sortedExpenses.map((e) => {
            const paid = e.splits?.filter((s) => s.is_paid).length ?? 0;
            const total = e.splits?.length ?? 0;
            const currencySymbol = e.currency?.symbol ?? "";
            const amount = `${currencySymbol}${e.sum.toFixed(2)}`;
            const totalAmount = `${currencySymbol}${e.total_amount.toFixed(2)}`;
            const ownerName = users.find(u => u.uuid == e.owner_id)?.name ?? e.owner_id;
            const dateTime = new Date(e.date);

            const isOpen = expandedId === e.id;

            return (
              <React.Fragment key={e.id}>
                <tr
                  onClick={() => toggle(e.id)}
                  aria-expanded={isOpen}
                  className="cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800"
                >
                  <td className="px-4 py-2 whitespace-nowrap">{dateTime.toLocaleString()}</td>
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
                  <td colSpan={5} className="p-0">
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
