# Arch Refactor Engine

Automated, repository-wide architectural refactoring engine implemented in Rust, Oxc AST, and NAPI-RS for high-performance TypeScript and JavaScript codebase modernization.

---

## Table of Contents

- [Overview](#overview)
- [Key Capabilities](#key-capabilities)
  - [1. Framework Boundary Protection](#1-framework-boundary-protection)
  - [2. AST Dependency Graph and Cycle Detection](#2-ast-dependency-graph-and-cycle-detection)
  - [3. Zero-Formatting-Drift Byte Patcher](#3-zero-formatting-drift-byte-patcher)
  - [4. AST Code Clone and Redundancy Detection](#4-ast-code-clone-and-redundancy-detection)
  - [5. Atomic Transactions and Lossless Rollback](#5-atomic-transactions-and-lossless-rollback)
- [Installation and Execution](#installation-and-execution)
  - [Using via NPX (No Installation)](#using-via-npx-no-installation)
  - [Global Installation](#global-installation)
  - [Local Project Dependency](#local-project-dependency)
- [Command Line Interface (CLI) Reference](#command-line-interface-cli-reference)
  - [1. Scan Architecture (`scan`)](#1-scan-architecture-scan)
  - [2. Detect Duplicate Code (`detect-clones`)](#2-detect-duplicate-code-detect-clones)
  - [3. Generate Architectural Plan (`plan`)](#3-generate-architectural-plan-plan)
  - [4. Preview Unified Diffs (`diff`)](#4-preview-unified-diffs-diff)
  - [5. Apply Transformation (`apply`)](#5-apply-transformation-apply)
  - [6. Undo / Rollback (`undo`)](#6-undo--rollback-undo)
- [Architectural Layout Targets](#architectural-layout-targets)
  - [Feature-Based Architecture (`feature-based`)](#feature-based-architecture-feature-based)
  - [Domain-Driven Design Architecture (`ddd`)](#domain-driven-design-architecture-ddd)
  - [Layered Architecture (`layered`)](#layered-architecture-layered)
- [Naming Conventions](#naming-conventions)
- [Supported Frameworks and Routing Conventions](#supported-frameworks-and-routing-conventions)
- [Programmatic TypeScript / JavaScript SDK](#programmatic-typescript--javascript-sdk)
- [Prebuilt Cross-Platform Binary Support](#prebuilt-cross-platform-binary-support)
- [Building from Source and Testing](#building-from-source-and-testing)
- [License](#license)

---

## Overview

Modernizing large JavaScript and TypeScript codebases often requires shifting directory layouts, consolidating duplicate utility functions, resolving circular dependencies, and updating import statements across hundreds or thousands of files.

Arch Refactor Engine executes these operations natively using an AST engine built in Rust on top of Oxc, exposed to Node.js environments through NAPI-RS. It completes repository-wide scans, duplicate code detection, AST byte patch generation, and transaction commits in milliseconds while guaranteeing zero formatting drift and automated rollback capabilities.

---

## Key Capabilities

### 1. Framework Boundary Protection
The engine analyzes repository layout and respects framework-specific routing conventions. Framework-managed entry points, route handlers, layouts, and server/client directives are locked and preserved from destructive reorganizations.

### 2. AST Dependency Graph and Cycle Detection
Using Oxc's parser and allocator, the engine constructs a full directed graph of static imports, dynamic `import()` calls, CommonJS `require()` statements, and re-exports. It resolves path aliases defined in `tsconfig.json` / `jsconfig.json` and detects circular dependency cycles using depth-first search (DFS).

### 3. Zero-Formatting-Drift Byte Patcher
Import specifiers are modified at the exact byte spans extracted from AST nodes. Replacements are applied in reverse offset order to prevent coordinate shift. Indentation, line endings, comments, and quotation marks (single vs. double quotes) are preserved exactly as written.

### 4. AST Code Clone and Redundancy Detection
The clone detector normalizes AST subtrees by stripping trivia and anonymizing identifiers into canonical tokens (`$VAR`). It computes 64-bit SipHash fingerprints to group identical and near-identical functions into duplicate clusters and generates utility extraction recommendations (`src/shared/utils/<name>.ts`).

### 5. Atomic Transactions and Lossless Rollback
Before applying changes, the engine inspects Git cleanliness and creates a full snapshot journal stored at `.refactor-journal.json`. If a transformation needs to be undone, running `undo` immediately reverts all modified files to their exact prior byte state and removes created directories.

---

## Installation and Execution

### Using via NPX (No Installation)
Execute the CLI directly on any target directory without global installation:

```bash
npx arch-refactor-engine scan /path/to/project
```

### Global Installation
Install the binary globally for continuous CLI usage across repositories:

```bash
npm install -g arch-refactor-engine
```

After installation, the `arch-refactor` command is available system-wide:

```bash
arch-refactor --help
```

### Local Project Dependency
Install as a devDependency in existing projects:

```bash
npm install --save-dev arch-refactor-engine
```

---

## Command Line Interface (CLI) Reference

### 1. Scan Architecture (`scan`)
Scans the repository, identifies frameworks, maps tsconfig path aliases, detects circular dependencies, and reports framework-protected files.

```bash
arch-refactor scan [target_path]
```

Example:
```bash
arch-refactor scan ./src
```

### 2. Detect Duplicate Code (`detect-clones`)
Scans functions across all source files, detects duplicated blocks, calculates potential lines saved, and suggests centralized destination targets.

```bash
arch-refactor detect-clones [target_path] [--min-lines=<number>]
```

Options:
- `--min-lines`: Minimum number of source lines for a function to be evaluated (Default: 3).

Example:
```bash
arch-refactor detect-clones . --min-lines=5
```

### 3. Generate Architectural Plan (`plan`)
Generates a complete migration plan detailing file relocations, renamings, and import rewire operations without modifying files on disk.

```bash
arch-refactor plan [target_path] [options]
```

Options:
- `-a, --architecture`: Target layout (`feature-based`, `ddd`, `layered`). Default: `feature-based`.
- `-n, --naming`: Naming convention (`kebab-case`, `pascal-case`, `camel-case`, `snake-case`, `preserve`). Default: `kebab-case`.

Example:
```bash
arch-refactor plan . --architecture=feature-based --naming=kebab-case
```

### 4. Preview Unified Diffs (`diff`)
Generates unified diff hunks showing the exact byte changes that will be applied to import paths across all affected files.

```bash
arch-refactor diff [target_path] [options]
```

Example:
```bash
arch-refactor diff . --architecture=feature-based
```

### 5. Apply Transformation (`apply`)
Executes the architectural refactor atomically, moving files, patching import specifiers, and writing the transaction journal.

```bash
arch-refactor apply [target_path] [options]
```

Options:
- `--dry-run`: Simulates the refactoring transaction without writing to disk.
- `-f, --force`: Overrides Git working tree cleanliness check.
- `-y, --yes`: Skips interactive confirmation prompt.
- `-a, --architecture`: Target architecture layout.
- `-n, --naming`: Target file naming style.

Example (Dry Run):
```bash
arch-refactor apply . --dry-run
```

Example (Execution):
```bash
arch-refactor apply . -a feature-based -n kebab-case --yes
```

### 6. Undo / Rollback (`undo`)
Rolls back the previous refactor transaction recorded in `.refactor-journal.json`, restoring original file contents and deleting newly generated directories.

```bash
arch-refactor undo [target_path]
```

Example:
```bash
arch-refactor undo .
```

---

## Architectural Layout Targets

### Feature-Based Architecture (`feature-based`)
Partitions domain logic into dedicated feature modules under `src/features/<feature>/`, separating shared infrastructure into `src/shared/`:

```
src/
├── features/
│   ├── auth/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── services/
│   │   └── types/
│   └── billing/
│       ├── components/
│       ├── hooks/
│       ├── services/
│       └── types/
└── shared/
    ├── components/
    ├── hooks/
    └── utils/
```

### Domain-Driven Design Architecture (`ddd`)
Organizes modules by strategic DDD layers:

```
src/
├── domain/
│   ├── models/
│   └── repositories/
├── application/
│   ├── use-cases/
│   └── services/
├── infrastructure/
│   ├── api/
│   └── persistence/
├── presentation/
│   ├── components/
│   └── hooks/
└── shared/
    └── utils/
```

### Layered Architecture (`layered`)
Groups files by technical role across the codebase:

```
src/
├── components/
├── hooks/
├── services/
├── utils/
└── types/
```

---

## Naming Conventions

The engine supports automated file stem normalization during refactoring:

| Convention | Option Value | Input Example | Output Example |
| :--- | :--- | :--- | :--- |
| Kebab Case | `kebab-case` | `UserProfileCard.tsx` | `user-profile-card.tsx` |
| Pascal Case | `pascal-case` | `user_profile_card.tsx` | `UserProfileCard.tsx` |
| Camel Case | `camel-case` | `UserProfileCard.ts` | `userProfileCard.ts` |
| Snake Case | `snake-case` | `userProfileCard.ts` | `user_profile_card.ts` |
| Preserve | `preserve` | `UserProfileCard.tsx` | `UserProfileCard.tsx` |

---

## Supported Frameworks and Routing Conventions

The engine recognizes and protects the following conventions:

- Next.js App Router: `app/**/page.{js,jsx,ts,tsx}`, `app/**/layout.{js,jsx,ts,tsx}`, `app/**/loading.{js,jsx,ts,tsx}`, `app/**/error.{js,jsx,ts,tsx}`, `app/**/not-found.{js,jsx,ts,tsx}`, `app/**/route.{js,ts}`, route groups `(group)`, parallel routes `@slot`, intercepting routes `(.)`, and directive headers (`"use client"`, `"use server"`).
- Next.js Pages Router: `pages/**`, `pages/_app.{js,jsx,ts,tsx}`, `pages/_document.{js,jsx,ts,tsx}`, `pages/api/**`.
- Remix / React Router v7: `app/routes/**`, `app/root.{js,jsx,ts,tsx}`.
- Vite: Standard `src/` modular Single Page Applications.
- NestJS / Express: Controller, service, and module hierarchies.

---

## Programmatic TypeScript / JavaScript SDK

The engine can be invoked directly inside Node.js tools, migration scripts, or CI pipelines:

```typescript
import { RefactoringEngine } from 'arch-refactor-engine';

// 1. Scan repository
const scanResult = RefactoringEngine.scan({
  rootPath: process.cwd(),
});
console.log(`Detected Framework: ${scanResult.framework}`);
console.log(`Total Source Files: ${scanResult.totalFiles}`);

// 2. Identify code clone clusters
const cloneResult = RefactoringEngine.detectClones({
  rootPath: process.cwd(),
  minLines: 4,
});
console.log(`Duplicate Clusters: ${cloneResult.totalClusters}`);
console.log(`Estimated Lines Saved: ${cloneResult.totalLinesSaved}`);

// 3. Generate architectural plan
const plan = RefactoringEngine.plan({
  rootPath: process.cwd(),
  targetArchitecture: 'FeatureBased',
  namingConvention: 'KebabCase',
});

// 4. Preview diff
const diffPreview = RefactoringEngine.previewDiff(plan);
console.log(`Files Changed: ${diffPreview.totalFilesChanged}`);

// 5. Apply refactoring
const applyResult = RefactoringEngine.apply(plan, {
  dryRun: false,
  force: true,
});

if (applyResult.success) {
  console.log(`Successfully refactored ${applyResult.filesMoved} files in transaction: ${applyResult.transactionId}`);
}

// 6. Rollback if necessary
// RefactoringEngine.rollback(undefined, process.cwd());
```

---

## Prebuilt Cross-Platform Binary Support

Native binaries are built and distributed across major architectures:

- Linux x64 (glibc & musl)
- Linux arm64 (glibc)
- macOS Apple Silicon (arm64)
- macOS Intel (x64)
- Windows x64 (MSVC)

---

## Building from Source and Testing

### Prerequisites
- Node.js >= 20.0.0
- Rust stable toolchain (`cargo`, `rustc`)

### Build Steps
```bash
# Clone the repository
git clone https://github.com/yohanness16/napi.git
cd napi

# Install dependencies
npm install

# Compile native Rust extension and TypeScript definitions
npm run build
```

### Running Test Suite
```bash
# Run unit and integration tests
npm test
```

---

## License

MIT License. Copyright (c) Arch Refactor Team.
