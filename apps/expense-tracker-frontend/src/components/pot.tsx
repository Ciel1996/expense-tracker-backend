'use client';

import {FC} from "react";
import Link from "next/link";
import {Card} from "./card";

type PotProps = {
  name: string;
  currency: string;
  owner: string;
  users: string[];
  balance: number;
  href: string;
  isArchived?: boolean;
};

export const Pot: FC<PotProps> = ({name, currency, owner, users, balance, href, isArchived}) => {
  return (
    <Link href={href} className="block focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-lg">
      <Card hover={true} padding="md" className={`w-full min-w-[16rem] min-h-[120px] cursor-pointer ${isArchived ? 'opacity-75 bg-gray-50 dark:bg-gray-800/50' : ''}`}>
        <div className="space-y-3">
          <div className="flex justify-between items-start">
            <h2 className="font-bold text-base text-gray-900 dark:text-white truncate">
              {name}
            </h2>
            {isArchived && (
              <span className="bg-gray-200 text-gray-700 text-[10px] font-bold px-1.5 py-0.5 rounded uppercase tracking-wider">
                Archived
              </span>
            )}
          </div>

          <div className="space-y-2 text-sm text-gray-600 dark:text-gray-300">
            <div className="flex justify-between items-center">
              <span className="font-medium text-gray-700 dark:text-gray-200">Owner:</span>
              <span className="truncate ml-2">{owner}</span>
            </div>

            <div className="flex justify-between items-center">
              <span className="font-medium text-gray-700 dark:text-gray-200">Currency:</span>
              <span className="font-mono text-green-600 dark:text-green-400">{currency}</span>
            </div>

            <div className="flex justify-between items-center">
              <span className="font-medium text-gray-700 dark:text-gray-200">Balance:</span>
              {balance >= 0 ? (
                <span className="font-mono text-gray-700 dark:text-gray-200">{balance.toFixed(2)}</span>
              ) : (
                <span className="font-mono text-red-600 dark:text-red-400">{balance.toFixed(2)}</span>
              )}
            </div>

            <div className="space-y-1">
              <span className="font-medium text-gray-700 dark:text-gray-200 block">Users:</span>
              <div className="text-xs">
                {users.length > 3 ? (
                  <span>{users.slice(0, 3).join(', ')} +{users.length - 3} more</span>
                ) : (
                  <span>{users.join(', ')}</span>
                )}
              </div>
            </div>
          </div>
        </div>
      </Card>
    </Link>
  )
};
