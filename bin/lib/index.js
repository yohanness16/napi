"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getGitStatus = exports.detectClones = exports.rollbackRefactor = exports.applyRefactor = exports.previewDiff = exports.generatePlan = exports.scanRepository = exports.RefactoringEngine = void 0;
const binding_1 = require("./binding");
__exportStar(require("./types"), exports);
/**
 * High-performance repository-wide architectural refactoring engine
 */
class RefactoringEngine {
    /**
     * Scan repository to detect frameworks, boundary files, imports, and AST dependency graph
     */
    static scan(config) {
        return binding_1.native.scanRepository(config);
    }
    /**
     * Generate an architectural refactoring plan with naming normalization and import patching
     */
    static plan(config) {
        return binding_1.native.generatePlan(config);
    }
    /**
     * Generate a unified diff preview for a refactor plan without modifying disk files
     */
    static previewDiff(plan) {
        return binding_1.native.previewDiff(plan);
    }
    /**
     * Apply a refactoring plan atomically with git guardrails and rollback journal creation
     */
    static apply(plan, options) {
        return binding_1.native.applyRefactor(plan, options);
    }
    /**
     * Roll back a refactor transaction using the `.refactor-journal.json` file
     */
    static rollback(journalPath, rootPath) {
        return binding_1.native.rollbackRefactor(journalPath, rootPath);
    }
    /**
     * Detect duplicated code patterns and AST clones across the repository
     */
    static detectClones(config) {
        return binding_1.native.detectClones(config);
    }
    /**
     * Check the Git status of the repository for clean working tree validation
     */
    static getGitStatus(rootPath) {
        return binding_1.native.getGitStatus(rootPath);
    }
}
exports.RefactoringEngine = RefactoringEngine;
// Named function exports for flexible functional usage
exports.scanRepository = RefactoringEngine.scan;
exports.generatePlan = RefactoringEngine.plan;
exports.previewDiff = RefactoringEngine.previewDiff;
exports.applyRefactor = RefactoringEngine.apply;
exports.rollbackRefactor = RefactoringEngine.rollback;
exports.detectClones = RefactoringEngine.detectClones;
exports.getGitStatus = RefactoringEngine.getGitStatus;
exports.default = RefactoringEngine;
