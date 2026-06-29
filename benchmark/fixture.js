// Shared fixture: same logical data, rendered in 4 formats.
// Used by benchmark.js to compare parse throughput.

const fixtureObject = {
  app: {
    name: "Super App",
    version: "1.2.0",
    debug: false,
    max_users: 1000,
  },
  database: {
    host: "localhost",
    port: 5432,
    ssl: true,
    pool: { min: 5, max: 20 },
  },
  features: {
    auth: { enabled: true, provider: "google" },
    cache: { enabled: false, ttl: 300 },
  },
  servers: [
    { id: 1, host: "node1.example.com", region: "us-east" },
    { id: 2, host: "node2.example.com", region: "us-west" },
    { id: 3, host: "node3.example.com", region: "eu-central" },
  ],
  tags: ["api", "backend", "v1", "production"],
};

const json = JSON.stringify(fixtureObject, null, 2);

const yaml = `app:
  name: "Super App"
  version: "1.2.0"
  debug: false
  max_users: 1000
database:
  host: "localhost"
  port: 5432
  ssl: true
  pool:
    min: 5
    max: 20
features:
  auth:
    enabled: true
    provider: "google"
  cache:
    enabled: false
    ttl: 300
servers:
  - id: 1
    host: "node1.example.com"
    region: "us-east"
  - id: 2
    host: "node2.example.com"
    region: "us-west"
  - id: 3
    host: "node3.example.com"
    region: "eu-central"
tags:
  - "api"
  - "backend"
  - "v1"
  - "production"
`;

const toml = `tags = ["api", "backend", "v1", "production"]

[app]
name = "Super App"
version = "1.2.0"
debug = false
max_users = 1000

[database]
host = "localhost"
port = 5432
ssl = true
[database.pool]
min = 5
max = 20

[features.auth]
enabled = true
provider = "google"
[features.cache]
enabled = false
ttl = 300

[[servers]]
id = 1
host = "node1.example.com"
region = "us-east"

[[servers]]
id = 2
host = "node2.example.com"
region = "us-west"

[[servers]]
id = 3
host = "node3.example.com"
region = "eu-central"
`;

const yan = `app:
  name: "Super App"
  version: "1.2.0"
  debug: false
  max_users: 1000

database:
  host: "localhost"
  port: 5432
  ssl: true
  pool: {min: 5; max: 20}

features:
  auth: {enabled: true; provider: "google"}
  cache: {enabled: false; ttl: 300}

servers: { {id: 1; host: "node1.example.com"; region: "us-east"}; {id: 2; host: "node2.example.com"; region: "us-west"}; {id: 3; host: "node3.example.com"; region: "eu-central"} }

tags: api; backend; v1; production
`;

module.exports = { fixtureObject, json, yaml, toml, yan };
