# Cebelica CLI

- This is CLI for invoice issuing / accounting software named Čebelca
- CLI connects and interacts via GraphQL API interface named Gateway.
- Source code for cebelca-gateway is available at `../cebelca-gateway` folder.
- Dependencies for this project are managed via devenv (Nix). There is `devenv.nix` 
with full definition and dependencies.
- Goal and aim of this project is to build a user and machine friendly CLI interface that can be used for to conduct common invoicing and accounting operations. Issuing invoices, logging payments, stats, exports etc.

# Structure

- `src/` - is the Rust code of the project
- `bin/` - has helper bash scripts that can be used to help with development
- `graphql/` - has collection of GraphQL queries that are used to generate Rust code and types with the help of `graphql_client` library.
- `graphql/schema.graphql` file is GraphQL schema from `cebelca-gateway`. It is usually copied from gateway source code during the development and will be auto-updated via GitHub when this project eventually grows furhter. Try to keep an eye on changes over this file during the development. If there is significant change to it - the rest of the queries likely also need to be updated and project (re)built.

# Rust

- This project uses `anyhow` for better error ergonomics
- This project will be compiled and packaged for different platforms and distributions. Linux, Windows and Mac.

# Environment

- This CLI depends on two environment variables (passed via `.envrc` and devenv)
- `CEBELCA_TOKEN` is authentication token that can also be passed to CLI
- `CEBELCA_GATEWAY_URL` in development points to `http://localhost:5454/api/graphql`

