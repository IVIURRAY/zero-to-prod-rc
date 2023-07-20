# Zero 2 Prod: Book

# Running the app
### locally
```shell
cargo watch -x check -x test -x run
```

Formatting Rust code
```shell
cargo fmt
```

Running tests
```shell
cargo test
```

Running Clippy
```shell
cargo clippy
```

# Migrating the db
### locally 

```shell
SKIP_DOCKER=true ./scripts/init_db.sh
```

### production
```shell
DATABASE_URL=YOUR-DIGITAL-OCEAN-DB-CONNECTION-STRING sqlx migrate run
```

# Digital Ocean

Making any changes to `spec.yaml` need to be applied to Digital Ocean.

> Remember to apply the changes to DigitalOcean every time we touch spec.yaml: grab your app identifier via doctl apps list --format ID and then run doctl apps update $APP_ID --spec spec.yaml.