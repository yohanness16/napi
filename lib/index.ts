import { native } from './binding';
import {
  ApplyOptions,
  ApplyResult,
  CloneDetectionConfig,
  CloneDetectionResult,
  DiffPreviewResult,
  GitStatusResult,
  PlanConfig,
  RefactorPlan,
  RepositoryScanResult,
  RollbackResult,
  ScanConfig,
} from './types';

export * from './types';

/**
 * High-performance repository-wide architectural refactoring engine
 */
export class RefactoringEngine {
  /**
   * Scan repository to detect frameworks, boundary files, imports, and AST dependency graph
   */
  public static scan(config: ScanConfig): RepositoryScanResult {
    return native.scanRepository(config);
  }

  /**
   * Generate an architectural refactoring plan with naming normalization and import patching
   */
  public static plan(config: PlanConfig): RefactorPlan {
    return native.generatePlan(config);
  }

  /**
   * Generate a unified diff preview for a refactor plan without modifying disk files
   */
  public static previewDiff(plan: RefactorPlan): DiffPreviewResult {
    return native.previewDiff(plan);
  }

  /**
   * Apply a refactoring plan atomically with git guardrails and rollback journal creation
   */
  public static apply(plan: RefactorPlan, options?: ApplyOptions): ApplyResult {
    return native.applyRefactor(plan, options);
  }

  /**
   * Roll back a refactor transaction using the `.refactor-journal.json` file
   */
  public static rollback(journalPath?: string, rootPath?: string): RollbackResult {
    return native.rollbackRefactor(journalPath, rootPath);
  }

  /**
   * Detect duplicated code patterns and AST clones across the repository
   */
  public static detectClones(config: CloneDetectionConfig): CloneDetectionResult {
    return native.detectClones(config);
  }

  /**
   * Check the Git status of the repository for clean working tree validation
   */
  public static getGitStatus(rootPath: string): GitStatusResult {
    return native.getGitStatus(rootPath);
  }
}

// Named function exports for flexible functional usage
export const scanRepository = RefactoringEngine.scan;
export const generatePlan = RefactoringEngine.plan;
export const previewDiff = RefactoringEngine.previewDiff;
export const applyRefactor = RefactoringEngine.apply;
export const rollbackRefactor = RefactoringEngine.rollback;
export const detectClones = RefactoringEngine.detectClones;
export const getGitStatus = RefactoringEngine.getGitStatus;

export default RefactoringEngine;
