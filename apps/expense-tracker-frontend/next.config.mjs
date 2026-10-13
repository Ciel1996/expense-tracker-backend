//@ts-check

import { composePlugins, withNx } from '@nx/next';

/**
 * @type {import('@nx/next/plugins/with-nx').WithNxOptions}
 **/
const nextConfig = {
  nx: {},
  output: 'standalone',
  transpilePackages: ['@./expense-tracker-client'],
  experimental: {
    useTypeScriptCli: true,
  },
};

const plugins = [
  withNx,
];

export default composePlugins(...plugins)(nextConfig);
