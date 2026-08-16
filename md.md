# **Building an Automated Repository-Wide Architectural Refactoring Engine with Rust and NAPI-RS for NPM**

# **1\. Architecture & Project Layout**

The engine utilizes a hybrid monorepo structure to manage both high-performance Rust internals and the Node.js consumer interface.

## **Directory Tree**

* `src/`: Primary Rust source code for the refactoring engine.  
* `lib/`: TypeScript source for the programmatic SDK and NAPI-RS wrappers.  
* `bin/`: CLI entry points and command logic.  
* `npm/`: Platform-specific distribution packages (e.g., `@engine/darwin-arm64`).  
* `Cargo.toml`: Rust workspace and dependency configuration.  
* `package.json`: Root configuration defining the CLI and platform-specific `optionalDependencies`.

## **Core Configurations**

The `package.json` at the root acts as the central hub, managing binaries via `optionalDependencies` to ensure users only download the native build required for their architecture.

The `Cargo.toml` includes specific crates required for high-performance AST manipulation and repository analysis:

* **NAPI-RS**: For high-speed JavaScript/TypeScript bindings.  
* **Oxc**: Providing the parser, allocator, span, and resolver for TypeScript/JavaScript source code.  
* **Gix**: For deep Git integration and repository state management.  
* **Rayon**: To parallelize repository scanning and AST traversal.  
* **Similar**: For generating unified diff previews before committing changes.

# **2\. Step-by-Step Rust Core Implementation**

## **Module 1: Scanner & Framework Boundary Detector**

This module performs the initial pass of the repository. It is designed to be framework-aware, detecting protected structures such as Next.js App Router rules and Remix protected routes. It respects framework boundaries while utilizing `tsconfig` paths to identify the scope of the project.

## **Module 2: AST Dependency Graph & Module Resolver**

Leveraging the **Oxc** suite, this module parses the source code into an Abstract Syntax Tree. It handles complex module resolution, including native Node.js modules and custom path aliases, to build a comprehensive map of how files interact across the project.

## **Module 3: Architectural Planner & Naming Normalizer**

The planner determines the new project structure based on Domain-Driven Design (DDD) or feature-based modular layouts. It includes a naming engine to enforce consistency across the codebase, transforming identifiers and file names into kebab-case or PascalCase as required by the target architecture.

## **Module 4: High-Precision AST Byte Patcher**

Unlike traditional find-and-replace, the byte patcher uses Oxc spans to calculate precise locations for modification. It updates relative import paths and source code references directly at the byte level, ensuring that comments, whitespace, and original formatting remain intact.

## **Module 5: Code Clone & Utility Deduplication Detector**

To improve code quality during refactoring, this module normalizes AST subtrees and uses SipHash to identify duplicate logic clusters. It detects redundant utility functions and suggests candidates for deduplication into shared modules.

## **Module 6: Safety, Git Guardrails & Transaction Engine**

Before any write operation, the engine verifies the Git state for cleanliness. It generates a unified diff preview for user review. Every refactoring action is recorded in a .refactor-journal.json file, which enables a robust rollback system to restore the previous state if errors are detected.

## **Module 7: NAPI-RS Bridge**

Located in `src/lib.rs`, this module defines the interface between Rust and Node.js. It exports the core engine functions as JavaScript/TypeScript bindings, allowing the CLI and SDK to invoke the Rust logic with minimal overhead.

# **3\. Node.js CLI & Programmatic API**

## **CLI Entry Point (bin/cli.ts)**

The CLI provides a user-friendly interface for the engine. It supports the following commands:

* `scan`: Analyze the current architecture and identify potential refactoring targets.  
* `apply`: Execute the architectural transformation based on the plan.  
* `detect-clones`: Identify duplicated code patterns across the repository.  
* `undo`: Roll back the last refactor operation using the journal.

## **Programmatic SDK**

The package includes an `index.d.ts` file providing full TypeScript definitions for the native bindings. Developers can integrate the engine directly into their own tooling by importing the programmatic SDK, allowing for custom refactoring workflows.

# **4\. Cross-Platform Compilation & Automated NPM Publishing CI/CD**

## **GitHub Actions Workflow Matrix**

The engine is compiled for multiple platforms to ensure broad compatibility. The build matrix covers:

* **macOS**: x64 and arm64 (Apple Silicon).  
* **Linux**: x64 (gnu and musl) and arm64 (gnu).  
* **Windows**: x64 msvc.

## **Automated Release Pipeline**

Upon a successful build of the native `.node` binaries, the CI/CD pipeline triggers an automated release to the npm registry. This process includes **npm provenance**, providing a verifiable link between the published package and the source code repository for enhanced security.