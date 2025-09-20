import type { GetServerSideProps, NextPage } from "next";
import { useGetPotExpenses, useGetUsers } from "@./expense-tracker-client";
import React, { useState } from "react";
import { ExpensesTable } from "../../components/expenses/expenses-table";

type Props = { id: number };

const PotDetails: NextPage<Props> = ({ id }) => {
  const { data: expenses, isLoading, isError } = useGetPotExpenses(id);
  const { data: users, isLoading: isLoadingUsers, isError: isErrorUsers} = useGetUsers();

  if (isLoading || isLoadingUsers) {
    return <div className="p-4 text-sm text-gray-500">Loading expenses…</div>;
  }

  if (isError || isErrorUsers) {
    return (
      <div className="p-4 text-sm text-red-600">Failed to load expenses. Please try again.</div>
    );
  }

  if (!expenses || expenses.length === 0) {
    return <div className="p-4 text-sm text-gray-500">Nothing to display</div>;
  }

  if (!users || users.length === 0) {
    return <div className="p-4 text-sm text-gray-500">Could not connect to user database</div>;
  }

  return (
    <div className="p-4">
      <h1 className="mb-4 text-xl font-semibold">Pot {id} — Expenses</h1>
      <ExpensesTable expenses={expenses} users={users}/>
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
