'use client';

import {FC} from "react";
import {Card} from "./card";

type PotProps = {
  name: string;
  currency: string;
  owner: string;
  users: string[];
};

export const Pot: FC<PotProps> = ({name, currency, owner, users}) => {
  return (
    <Card hover={true} padding="md" className="w-full min-w-[16rem] min-h-[120px]">
      <div className="space-y-3">
        <h2 className="font-bold text-base text-gray-900 dark:text-white truncate">
          {name}
        </h2>

        <div className="space-y-2 text-sm text-gray-600 dark:text-gray-300">
          <div className="flex justify-between items-center">
            <span className="font-medium text-gray-700 dark:text-gray-200">Owner:</span>
            <span className="truncate ml-2">{owner}</span>
          </div>

          <div className="flex justify-between items-center">
            <span className="font-medium text-gray-700 dark:text-gray-200">Currency:</span>
            <span className="font-mono text-green-600 dark:text-green-400">{currency}</span>
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
  )
};
