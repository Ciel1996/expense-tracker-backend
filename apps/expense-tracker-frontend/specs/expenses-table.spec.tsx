import React from "react";
import { fireEvent, render, screen, within } from "@testing-library/react";
import type { ExpenseDTO } from "@./expense-tracker-client";
import { ExpensesTable } from "../src/components/expenses/expenses-table";

jest.mock("../src/components/expenses/expense-split-details", () => ({
  ExpenseSplitDetails: ({ expenseId }: { expenseId: number }) => <div>Details for {expenseId}</div>,
}));

const expenses: ExpenseDTO[] = [
  {
    id: 1, date: "2025-02-01T10:00:00Z", description: "Banana", sum: -5, total_amount: 100,
    currency: { id: 1, name: "Euro", symbol: "€" }, owner_id: "owner", pot_id: 1, splits: [],
  },
  {
    id: 2, date: "2025-01-01T10:00:00Z", description: "Apple", sum: 20, total_amount: 1,
    currency: { id: 1, name: "Euro", symbol: "€" }, owner_id: "owner", pot_id: 1, splits: [],
  },
  {
    id: 3, date: "2025-03-01T10:00:00Z", description: "Cherry", sum: 3, total_amount: 50,
    currency: { id: 1, name: "Euro", symbol: "€" }, owner_id: "owner", pot_id: 1, splits: [],
  },
];

const rowNames = () =>
  screen.getAllByRole("row").filter((row) => row.hasAttribute("aria-expanded"))
    .map((row) => within(row).getAllByRole("cell")[1].textContent);

describe("ExpensesTable sorting", () => {
  it("preserves the input order until a column is selected", () => {
    render(<ExpensesTable expenses={expenses} users={[]} potId={1} />);
    expect(rowNames()).toEqual(["Banana", "Apple", "Cherry"]);
    expect(screen.getAllByRole("columnheader").every((header) => !header.hasAttribute("aria-sort"))).toBe(true);
  });

  it.each([
    ["Date and Time", ["Apple", "Banana", "Cherry"], ["Cherry", "Banana", "Apple"]],
    ["Description", ["Apple", "Banana", "Cherry"], ["Cherry", "Banana", "Apple"]],
    ["Amount", ["Banana", "Cherry", "Apple"], ["Apple", "Cherry", "Banana"]],
  ])("sorts %s ascending and descending without mutating the input", (column, ascending, descending) => {
    const input = [...expenses];
    render(<ExpensesTable expenses={input} users={[]} potId={1} />);
    const button = screen.getByRole("button", { name: column });
    const header = button.closest("th");

    fireEvent.click(button);
    expect(rowNames()).toEqual(ascending);
    expect(header?.getAttribute("aria-sort")).toBe("ascending");

    fireEvent.click(button);
    expect(rowNames()).toEqual(descending);
    expect(header?.getAttribute("aria-sort")).toBe("descending");
    expect(input).toEqual(expenses);
  });

  it("starts a newly selected column ascending and updates the sort indicator", () => {
    render(<ExpensesTable expenses={expenses} users={[]} potId={1} />);
    const date = screen.getByRole("button", { name: "Date and Time" });
    const amount = screen.getByRole("button", { name: "Amount" });
    fireEvent.click(date);
    fireEvent.click(date);
    fireEvent.click(amount);
    expect(date.closest("th")?.hasAttribute("aria-sort")).toBe(false);
    expect(amount.closest("th")?.getAttribute("aria-sort")).toBe("ascending");
    expect(amount.textContent).toContain("↑");
    expect(rowNames()).toEqual(["Banana", "Cherry", "Apple"]);
  });

  it("keeps ties in input order and reapplies sorting when expenses change", () => {
    const tied = [{ ...expenses[0], sum: 3 }, expenses[2], expenses[1]];
    const { rerender } = render(<ExpensesTable expenses={tied} users={[]} potId={1} />);
    fireEvent.click(screen.getByRole("button", { name: "Amount" }));
    expect(rowNames()).toEqual(["Banana", "Cherry", "Apple"]);
    fireEvent.click(screen.getByRole("button", { name: "Amount" }));
    expect(rowNames()).toEqual(["Apple", "Banana", "Cherry"]);
    rerender(<ExpensesTable expenses={[expenses[1], expenses[0]]} users={[]} potId={1} />);
    expect(rowNames()).toEqual(["Apple", "Banana"]);
  });

  it("keeps the expanded expense open when rows are reordered", () => {
    render(<ExpensesTable expenses={expenses} users={[]} potId={1} />);
    const banana = screen.getByText("Banana").closest("tr");
    expect(banana).not.toBeNull();
    fireEvent.click(banana!);
    expect(banana?.getAttribute("aria-expanded")).toBe("true");
    fireEvent.click(screen.getByRole("button", { name: "Description" }));
    expect(screen.getByText("Banana").closest("tr")?.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText("Details for 1").closest("[aria-hidden]")?.getAttribute("aria-hidden")).toBe("false");
  });
});
