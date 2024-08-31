# subscription_app
WIP application that is supposed to be a subscription app in the future.
There's no core business logic yet, but I implemeted login / logout and authentication middleware already

# start up

1. create .env first. refer to .env.example file as an example
2. then run following commands

```Shell
docker compose up --build -d # this will create a new database
docker compose logs -f
```

# clean up when finished

```Shell
docker compose down
```

# migration related commands

- generate new table

```Shell
sea-orm-cli migrate generate create_user_table
```

- migrate

```Shell
sea-orm-cli migrate up -u postgres://postgres@localhost:5432/subscription_db
```

generate entity

```Shell
sea-orm-cli generate entity -u postgres://postgres@localhost:5432/subscription_db -o entity/src
```