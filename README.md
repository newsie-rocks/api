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

### Postgres

```sh
brew services start postgresql@15
```

### Qdrant

```sh
docker run -p 6333:6333 -p 6334:6334 \
    -e QDRANT__SERVICE__GRPC_PORT="6334" \
    qdrant/qdrant
```

### Tracing

TBD
