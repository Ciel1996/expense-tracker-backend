# What is Expense-Tracker?
Expense-Tracker is a simple application that allows the users to track their household expenses.
A user can create pots, invite other users, and add expenses to be split among them.

## Wait, why is the repository called expense-tracker-backend, there is a frontend in here too?
Initially I wanted to create a Rust backend and then write a native iOS app (which I called expense-tracker) in SwiftUI. But soon after I realized that the backend would need a web UI so that users can self-host it and access it via their browser on any device they want.
This doesn't mean that the iOS app won't be available sometime in the future. But for now, I would like to focus on the backend and the web frontend.

Additionally I wanted to experiment with a tool which was new to me: Nx. So I decided to turn my backend repo into an Nx workspace.

# How to contribute
First: Thank you for your interest in contributing! I'm eager to see what we can learn from each other!

1. Clone the repository
2. if you are on macOS, I'd recommend using homebrew to install rustup and node: 
```bash
brew install rustup
brew install node
```

On Linux, I'd recommend using the official installers.

3. If you want to work on the backend, you will also need to install diesel_cli using this command:
```bash
cargo install diesel_cli --no-default-features --features postgres
```

Additionally you will need to install libpq on your system. The Dockerfile shows a way to do this on Debian-based systems.

On macOS you can use homebrew:
```bash
brew install libpq
```

(Honestly I haven't developed on Linux since early 2025, so I might have forgotten some details, please let me know if you run into any issues, and we'll figure out how to solve them 😊)

4. Run `npm install` to install the dependencies (crucial, otherwise you can't use Nx)
5. Modify docker compose to setup keycloak and postgreSQL (and the backend, if you want to work on the frontend 😊
6. Adjust the settings.toml file in the config directory to your needs, currently I'm using my NAS's URL as my keycloak server, but you can use localhost:8080/realms/local as the issuer address. In that case, the client would also be `expense-tracker`
7. If you want to run the frontend, make a copy of .env.local.example and rename it to .env.local. Then adjust the variables to your needs. 
8. If you want to run the backend use `cargo run` from the repo root (recommended), alternatively you can use `npx nx run expense_tracker`, but this creates a production build. 
9. If you want to run the frontend use `npx nx dev expense-tracker-frontend` from the repo root. 
10. If you made changes to the backend's API, e.g. adding a new endpoint, modifying or removing an existing one, you can use `npx nx swagger expense_tracker`, this will automatically update the OpenApi Spec and generate a new expense-tracker-client which can then be used by the frontend.

## Getting an Access Token for the API (using docker compose)
You can run the shell script `localAccessToken.sh` to get an access token for the API.
It will automatically copy the token to your clipboard.
You may need to allow the script to run by running `chmod +x localAccessToken.sh`

#

<a alt="Nx logo" href="https://nx.dev" target="_blank" rel="noreferrer"><img src="https://raw.githubusercontent.com/nrwl/nx/master/images/nx-logo.png" width="45"></a>

✨ Your new, shiny [Nx workspace](https://nx.dev) is almost ready ✨.

Run `npx nx graph` to visually explore what got created. Now, let's get you up to speed!

## Finish your CI setup

[Click here to finish setting up your workspace!](https://cloud.nx.app/connect/PPehlZtI1M)


## Run tasks

To run tasks with Nx use:

```sh
npx nx <target> <project-name>
```

For example:

```sh
npx nx build myproject
```

These targets are either [inferred automatically](https://nx.dev/concepts/inferred-tasks?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) or defined in the `project.json` or `package.json` files.

[More about running tasks in the docs &raquo;](https://nx.dev/features/run-tasks?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)

## Add new projects

While you could add new projects to your workspace manually, you might want to leverage [Nx plugins](https://nx.dev/concepts/nx-plugins?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) and their [code generation](https://nx.dev/features/generate-code?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) feature.

To install a new plugin you can use the `nx add` command. Here's an example of adding the React plugin:
```sh
npx nx add @nx/react
```

Use the plugin's generator to create new projects. For example, to create a new React app or library:

```sh
# Generate an app
npx nx g @nx/react:app demo

# Generate a library
npx nx g @nx/react:lib some-lib
```

You can use `npx nx list` to get a list of installed plugins. Then, run `npx nx list <plugin-name>` to learn about more specific capabilities of a particular plugin. Alternatively, [install Nx Console](https://nx.dev/getting-started/editor-setup?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) to browse plugins and generators in your IDE.

[Learn more about Nx plugins &raquo;](https://nx.dev/concepts/nx-plugins?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects) | [Browse the plugin registry &raquo;](https://nx.dev/plugin-registry?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)


[Learn more about Nx on CI](https://nx.dev/ci/intro/ci-with-nx#ready-get-started-with-your-provider?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)

## Install Nx Console

Nx Console is an editor extension that enriches your developer experience. It lets you run tasks, generate code, and improves code autocompletion in your IDE. It is available for VSCode and IntelliJ.

[Install Nx Console &raquo;](https://nx.dev/getting-started/editor-setup?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)

## Useful links

Learn more:

- [Learn about Nx on CI](https://nx.dev/ci/intro/ci-with-nx?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)
- [Releasing Packages with Nx release](https://nx.dev/features/manage-releases?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)
- [What are Nx plugins?](https://nx.dev/concepts/nx-plugins?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)

And join the Nx community:
- [Discord](https://go.nx.dev/community)
- [Follow us on X](https://twitter.com/nxdevtools) or [LinkedIn](https://www.linkedin.com/company/nrwl)
- [Our Youtube channel](https://www.youtube.com/@nxdevtools)
- [Our blog](https://nx.dev/blog?utm_source=nx_project&utm_medium=readme&utm_campaign=nx_projects)


---

# Expense Tracker - Docker & Nx Production Builds

This repository contains a Rust backend (Axum) and a Next.js frontend, managed with Nx. This section explains how to dockerize both applications and ensure Nx produces optimized production builds.

## What was added
- Dockerfile for backend: apps/expense_tracker/Dockerfile (multi-stage, release build, small runtime)
- Dockerfile for frontend: apps/expense-tracker-frontend/Dockerfile (Next.js standalone output, minimal runtime)
- docker-compose.yaml now includes services for db, keycloak, api, and web
- Example configs:
  - config/settings.toml.example → copy to config/settings.toml
  - apps/expense-tracker-frontend/.env.local.example → copy to apps/expense-tracker-frontend/.env.local
- Nx production builds:
  - Frontend project.json now has proper build/start targets and a production configuration
  - CI workflows use --configuration=production

To create the frontend's docker container locally run:
`docker build -t frontend -f apps/expense-tracker-frontend/Dockerfile .`

## Why these choices
- Multi-stage Docker images: keep final images small and reduce attack surface; build deps aren’t shipped.
- Next.js standalone output: runs with a minimal Node image without dev dependencies, speeding starts and reducing size.
- Explicit production builds in Nx: optimized JS/CSS, Rust release mode with LTO for smaller, faster binaries.
- docker-compose: local, reproducible stack including Postgres and Keycloak.

## Quick start (local)
1. Adjust config files:
   - config/settings_compose.toml
   - for Frontend, either adjust Docker Compose file environment variables or .env.local and mount .env.local into your container
2. Start everything (this might take a while):
   - docker compose up -d --build
3. Open services:
   - Frontend: http://localhost:3000 (go here after Keycloak is ready, you might need to restart docker compose, if nothing can be found. Click "Sign in to keycloak twice")
   - API: http://localhost:3001
   - Keycloak: http://localhost:8080
4. Go to http://localhost:8080/admin and login with admin/admin to view the Keycloak configuration.

## Build images manually
- Backend:
  - docker build -f apps/expense_tracker/Dockerfile -t expense-tracker/api:latest .
- Frontend (adjust API URL as needed):
  - docker build -f apps/expense-tracker-frontend/Dockerfile --build-arg NEXT_PUBLIC_API_URL=http://localhost:3001 -t expense-tracker/web:latest .

## Nx optimized builds
- Frontend: npx nx build expense-tracker-frontend --configuration=production
- Backend (Rust): npx nx build expense_tracker --configuration=production
- All: npx nx run-many -t build --configuration=production --parallel

Notes:
- Backend release builds use LTO (see Cargo.toml [profile.release]).
- next.config.mjs sets output: 'standalone' for smaller runtime images.

## Configuration details
- Backend reads config from config/settings.toml (mounted into the container). The example selects:
  - OIDC issuer (Keycloak dev realm)
  - audience string
  - port and database connection string (uses service name db)
- Frontend needs NEXT_PUBLIC_API_URL to call the API from the browser. Compose and Dockerfile set it to http://localhost:3001 by default.

## CI
- .github/workflows/ci.yml and cicd.yml run production builds via Nx.
- cicd.yml contains example steps to build Docker images on a self-hosted runner; optionally run docker compose up -d --build to deploy.

## Troubleshooting
- If Keycloak TLS causes JWKS fetch issues in dev, set EXPENSE_TRACKER_IGNORE_TLS=1 for the api service in compose (not recommended in prod).
- Ensure config/settings.toml exists before starting the api container.
- If ports collide, change host mappings in docker-compose.yaml.
