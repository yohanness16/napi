# ⚡ Arch Refactor Engine

> Automated, repository-wide architectural refactoring engine powered by **Rust**, **Oxc AST**, and **NAPI-RS** for high-performance TypeScript & JavaScript codebase modernization.

[![CI](https://github.com/arch-refactor/engine/actions/workflows/ci.yml/badge.svg)](https://github.com/arch-refactor/engine/actions/workflows/ci.yml)
[![Release](https://github.com/arch-refactor/engine/actions/workflows/release.yml/badge.svg)](https://github.com/arch-refactor/engine/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![npm version](https://img.shields.io/npm/v/@arch-refactor/engine.svg)](https://www.npmjs.com/package/@arch-refactor/engine)

---

## 🚀 Key Features

* **Framework-Aware Boundaries**: Automatically respects Next.js App Router rules (`page.tsx`, `layout.tsx`, `route.ts`, route groups `(group)`, intercepting routes `(.)`, parallel slots `@slot`, `"use client"` / `"use server"` directives), Next.js Pages Router conventions, and Remix / React Router v7 route hierarchies.
* **Blazing Fast Oxc AST Engine**: Native Rust AST parser and resolver traversing thousands of files in milliseconds using multi-core parallelization via `rayon`.
* **Zero-Formatting-Drift Byte Patcher**: Calculates exact byte spans from Oxc AST to patch import specifiers directly at the byte level, preserving original indentation, whitespace, quotes style (`'` vs `"`), and comments.
* **AST Code Clone & Utility Deduplication**: Normalizes AST subtrees and computes 64-bit SipHash fingerprints to find duplicated functions across the repository and recommend shared extraction modules.
* **Atomic Transactions & 1-Click Rollback**: Records all file operations and byte diffs into a `.refactor-journal.json` ledger, enabling immediate, lossless rollback via `arch-refactor undo`.
* **Git Guardrails**: Validates clean working trees, tracks repository status, and previews unified color diffs before applying changes.

---

## 📦 Directory Structure

```
.
├── src/                     # Rust Core Refactoring Engine
│   ├── ast_graph.rs         # Module 2: AST Dependency Graph & Module Resolver
│   ├── clones.rs            # Module 5: Code Clone & Deduplication Detector
│   ├── lib.rs               # Module 7: NAPI-RS JavaScript/TypeScript Bridge
│   ├── patcher.rs           # Module 4: High-Precision AST Byte Patcher
│   ├── planner.rs           # Module 3: Architectural Planner & Naming Normalizer
│   ├── scanner.rs           # Module 1: Scanner & Framework Boundary Detector
│   ├── transaction.rs       # Module 6: Safety, Git Guardrails & Transaction Engine
│   └── types.rs             # Core Data Structures & Serde Models
├── lib/                     # TypeScript Programmatic SDK
│   ├── binding.ts           # Native Binary Dynamic Loader
│   ├── index.ts             # Programmatic SDK Entry Point
│   └── types.ts             # TypeScript Type Definitions
├── bin/                     # CLI Entry Point
│   ├── cli.ts               # CLI Implementation
│   └── cli.js               # Executable Binary (`arch-refactor`)
├── npm/                     # Platform-Specific Distribution Packages
│   ├── darwin-arm64/
│   ├── darwin-x64/
│   ├── linux-arm64-gnu/
│   ├── linux-x64-gnu/
│   ├── linux-x64-musl/
│   └── win32-x64-msvc/
├── test/                    # Comprehensive Integration Test Suite
├── Cargo.toml               # Rust Workspace Configuration
├── package.json             # Root Package & NAPI Target Configuration
└── tsconfig.json            # TypeScript Configuration
```

---

## 🛠️ CLI Usage

You can run `arch-refactor` directly via `npx` or install it globally:

```bash
# Scan repository architecture & circular dependencies
npx arch-refactor scan [path]

# Generate architectural migration plan
npx arch-refactor plan [path] --architecture=feature-based --naming=kebab-case

# Preview unified diffs before applying
npx arch-refactor diff [path]

# Execute refactoring plan atomically
npx arch-refactor apply [path] --force --yes

# Detect duplicated code patterns and duplicate utilities
npx arch-refactor detect-clones [path] --min-lines=3

# Undo the last refactoring operation
npx arch-refactor undo [path]
```

### Supported Architectural Layouts (`--architecture, -a`):
* `feature-based` (default): Modular domain slices (`src/features/<feature>/{components, hooks, services, types, utils}`) + `src/shared/`.
* `ddd`: Domain-Driven Design layout (`src/domain/`, `src/application/`, `src/infrastructure/`, `src/presentation/`, `src/shared/`).
* `layered`: Layer-based architecture (`src/components/`, `src/hooks/`, `src/services/`, `src/utils/`, `src/types/`).

### Supported Naming Conventions (`--naming, -n`):
* `kebab-case` (default): `user-profile-card.tsx`
* `pascal-case`: `UserProfileCard.tsx`
* `camel-case`: `userProfileCard.ts`
* `snake-case`: `user_profile_card.ts`
* `preserve`: Keep existing file stems

---

## 💻 Programmatic TypeScript SDK

You can integrate `arch-refactor` directly into your build tools or custom pipelines:

```typescript
import { RefactoringEngine } from '@arch-refactor/engine';

// 1. Scan the repository
const scanResult = RefactoringEngine.scan({
  rootPath: process.cwd(),
});
console.log(`Framework: ${scanResult.framework}`);
console.log(`Total files: ${scanResult.totalFiles}`);

// 2. Detect code clones & duplicate functions
const cloneResult = RefactoringEngine.detectClones({
  rootPath: process.cwd(),
  minLines: 4,
});
console.log(`Found ${cloneResult.totalClusters} duplicate logic clusters`);

// 3. Generate architectural migration plan
const plan = RefactoringEngine.plan({
  rootPath: process.cwd(),
  targetArchitecture: 'FeatureBased',
  namingConvention: 'KebabCase',
});

// 4. Preview diffs
const diff = RefactoringEngine.previewDiff(plan);
console.log(`Will modify ${diff.totalFilesChanged} files (+${diff.totalAdditions} -${diff.totalDeletions})`);

// 5. Apply the refactor atomically
const applyResult = RefactoringEngine.apply(plan, {
  dryRun: false,
  force: true,
});

if (applyResult.success) {
  console.log(`Refactored ${applyResult.filesMoved} files in tx: ${applyResult.transactionId}`);
}

// 6. Rollback if needed
RefactoringEngine.rollback();
```

---

## 🏗️ Cross-Platform Prebuilt Matrix

Prebuilt native binaries are distributed for all major operating systems and architectures:

| Platform | Architecture | Libc / Runtime | Package |
| :--- | :--- | :--- | :--- |
| **macOS** | Apple Silicon (`arm64`) | Darwin | `@arch-refactor/engine-darwin-arm64` |
| **macOS** | Intel (`x64`) | Darwin | `@arch-refactor/engine-darwin-x64` |
| **Linux** | x64 | `glibc` | `@arch-refactor/engine-linux-x64-gnu` |
| **Linux** | x64 | `musl` (Alpine) | `@arch-refactor/engine-linux-x64-musl` |
| **Linux** | arm64 | `glibc` | `@arch-refactor/engine-linux-arm64-gnu` |
| **Windows** | x64 | MSVC | `@arch-refactor/engine-win32-x64-msvc` |

---

## 🧪 Development & Testing

```bash
# Install dependencies
npm install

# Compile native Rust addon & TypeScript SDK
npm run build

# Run comprehensive integration test suite
npm run test
```

---

## 📄 License

MIT © [Arch Refactor Team](LICENSE)
