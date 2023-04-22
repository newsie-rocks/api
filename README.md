# api

## Overview

REST API webservice

The service manages:

- authentication
- user feeds
- call of ML models

## OpenAPI doc

```sh
cargo run  --bin docgen -F docgen > doc/openapi/specs.yaml
redocly preview-docs doc/openapi/specs.yaml
```

## Local dev

### PostgresQL

```sh
brew services start postgresql@15
```

### Tracing

TBD
