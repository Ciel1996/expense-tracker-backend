module.exports = {
  "expense-tracker-client": {
    output: {
      mode: 'tags-split',
      target: 'src/endpoints/api.ts',
      schemas: 'src/model',
      client: 'react-query',
      prettier: true,
      override: {
        mutator: {
          path: './src/custom-client.ts',
          name: 'useCustomClient',
        }
      }
    },
    input: {
      target: '../../openapi/expense_tracker_openapi.json',
    },
  },
};
