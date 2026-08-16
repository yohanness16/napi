import { ApplyOptions, ApplyResult, CloneDetectionConfig, CloneDetectionResult, DiffPreviewResult, GitStatusResult, PlanConfig, RefactorPlan, RepositoryScanResult, RollbackResult, ScanConfig } from './types';
export * from './types';
/**
 * High-performance repository-wide architectural refactoring engine
 */
export declare class RefactoringEngine {
    /**
     * Scan repository to detect frameworks, boundary files, imports, and AST dependency graph
     */
    static scan(config: ScanConfig): RepositoryScanResult;
    /**
     * Generate an architectural refactoring plan with naming normalization and import patching
     */
    static plan(config: PlanConfig): RefactorPlan;
    /**
     * Generate a unified diff preview for a refactor plan without modifying disk files
     */
    static previewDiff(plan: RefactorPlan): DiffPreviewResult;
    /**
     * Apply a refactoring plan atomically with git guardrails and rollback journal creation
     */
    static apply(plan: RefactorPlan, options?: ApplyOptions): ApplyResult;
    /**
     * Roll back a refactor transaction using the `.refactor-journal.json` file
     */
    static rollback(journalPath?: string, rootPath?: string): RollbackResult;
    /**
     * Detect duplicated code patterns and AST clones across the repository
     */
    static detectClones(config: CloneDetectionConfig): CloneDetectionResult;
    /**
     * Check the Git status of the repository for clean working tree validation
     */
    static getGitStatus(rootPath: string): GitStatusResult;
}
export declare const scanRepository: typeof RefactoringEngine.scan;
export declare const generatePlan: typeof RefactoringEngine.plan;
export declare const previewDiff: typeof RefactoringEngine.previewDiff;
export declare const applyRefactor: typeof RefactoringEngine.apply;
export declare const rollbackRefactor: typeof RefactoringEngine.rollback;
export declare const detectClones: typeof RefactoringEngine.detectClones;
export declare const getGitStatus: typeof RefactoringEngine.getGitStatus;
export default RefactoringEngine;
//# sourceMappingURL=index.d.ts.map