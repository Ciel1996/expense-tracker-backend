module.exports = {
  'expense-tracker-client': {
    output: {
      mode: 'tags-split',
      target: 'src/endpoints/api.ts',
      schemas: 'src/model',
      client: 'react-query',
      httpClient: 'axios',
      override: {
        query: {
          version: 5,
        },
        mutator: {
          path: './src/custom-client.ts',
          name: 'useCustomClient',
        },
      },
    },
    input: {
      target: '../../openapi/expense_tracker_openapi.json',
    },
    hooks: {
      // Runs your local Prettier binary with your exact .prettierrc config
      afterAllFilesWrite: 'npx prettier --write ./src/api',
    },
  },
};
