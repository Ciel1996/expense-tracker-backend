# Working with migrations

## Create migrations

Run the following command to generate a new migration:
```bash
diesel migration generate {migration_name}
```

This creates a new folder withing the `migrations` folder.
You can then manually edit the generated up.sql/down.sql files.

## Apply migrations

To apply migrations, run the following command:
```bash
diesel migration run --database-url {database_url}
```

This will apply all pending migrations to the database and will update the schema.rs file.

## Rollback migrations

To rollback migrations, run the following command:
```bash
diesel migration revert --database-url {database_url}
```

This will rollback the last applied migration.

## Troubleshooting

If you encounter any issues with migrations, please refer to the [Diesel documentation](https://diesel.rs/guides/migrations/) for more information.
