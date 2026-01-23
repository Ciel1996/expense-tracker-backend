import type { GetServerSideProps, NextPage } from "next";
import { useGetPotExpenses, useGetUsers, useDeletePot, useGetPots } from "@./expense-tracker-client";
import React, { useState } from "react";
import { useRouter } from "next/router";
import { ExpensesTable } from "../../components/expenses/expenses-table";
import { NewExpenseModal } from "../../components/expenses/new-expense-modal";

type Props = { id: number };

const PotDetails: NextPage<Props> = ({ id }) => {
  const router = useRouter();
  const [isNewExpenseOpen, setNewExpenseOpen] = useState(false);
  const { data: expenses, isLoading, isError } = useGetPotExpenses(id);
  const { data: users, isLoading: isLoadingUsers, isError: isErrorUsers } = useGetUsers();
  const { data: pots, isLoading: isLoadingPots, isError: isErrorPots } = useGetPots();
  const pot = (pots ?? []).find((p) => p.id === id) ?? null;
  const { mutate: deletePot, isPending: isDeleting, error: deleteError } = useDeletePot({
    mutation: {
      onSuccess: async () => {
        // Navigate back to overview
        await router.push("/");
      },
    },
  });

  if (isLoading || isLoadingUsers || isLoadingPots) {
    return <div className="p-4 text-sm text-gray-500">Loading expenses…</div>;
  }

  if (isError || isErrorUsers || isErrorPots) {
    return (
      <div className="p-4 text-sm text-red-600">Failed to load expenses. Please try again.</div>
    );
  }

  // Compute derived state
  const hasExpenses = !!expenses && expenses.length > 0;
  const totalBalance = hasExpenses ? expenses.reduce((acc, e) => acc + (e.sum ?? 0), 0) : 0;
  const canDelete = !hasExpenses || totalBalance === 0;

  const handleDelete = () => {
    if (!canDelete || isDeleting) return;
    const ok = window.confirm("Are you sure you want to delete this pot? This action cannot be undone.");
    if (!ok) return;
    deletePot({ potId: id });
  };

  const deleteSection = (
    <div className="mt-4">
      <button
        onClick={handleDelete}
        disabled={!canDelete || isDeleting}
        className={`px-4 py-2 rounded-md text-white ${canDelete ? "bg-red-600 hover:bg-red-700" : "bg-gray-300 cursor-not-allowed"}`}
        title={canDelete ? "Delete Pot" : "Cannot delete: outstanding balance exists"}
      >
        {isDeleting ? "Deleting…" : "Delete Pot"}
      </button>
    </div>
  );

  if (!users || users.length === 0) {
    return <div className="p-4 text-sm text-gray-500">Could not connect to user database</div>;
  }

  return (
    <div className="p-4">
      <div className="flex items-center justify-between mb-2">
        <h1 className="text-xl font-semibold">Pot {id} — Expenses</h1>
        <button
          onClick={() => setNewExpenseOpen(true)}
          disabled={!pot}
          className="px-4 py-2 rounded-md bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
          title={!pot ? "Loading pot…" : "Add a new expense"}
        >
          Add Expense
        </button>
      </div>

      {/* Content */}
      {hasExpenses ? (
        <div className="mt-4">
          <ExpensesTable expenses={expenses} users={users} potId={id} />
        </div>
      ) : (
        <div className="mt-4 text-sm text-gray-500">Nothing to display</div>
      )}

      {/* Show delete button if allowed */}
      {canDelete && deleteSection}

      {/* Modal */}
      <NewExpenseModal open={isNewExpenseOpen} onClose={() => setNewExpenseOpen(false)} pot={pot} potId={id} />
    </div>
  );
};

export const getServerSideProps: GetServerSideProps<Props> = async (ctx) => {
  const rawId = ctx.params?.id;

  // Validate and parse id
  if (Array.isArray(rawId)) {
    return { notFound: true };
  }

  const idNum = Number(rawId);

  if (!Number.isFinite(idNum)) {
    return { notFound: true };
  }

  return {
    props: { id: idNum },
  };
};

export default PotDetails;
