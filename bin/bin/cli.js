#!/usr/bin/env node
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
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const readline = __importStar(require("readline"));
const index_1 = require("../lib/index");
// ANSI Color helper functions
const colors = {
    reset: (text) => `\x1b[0m${text}\x1b[0m`,
    bold: (text) => `\x1b[1m${text}\x1b[0m`,
    dim: (text) => `\x1b[2m${text}\x1b[0m`,
    cyan: (text) => `\x1b[36m${text}\x1b[0m`,
    green: (text) => `\x1b[32m${text}\x1b[0m`,
    yellow: (text) => `\x1b[33m${text}\x1b[0m`,
    red: (text) => `\x1b[31m${text}\x1b[0m`,
    magenta: (text) => `\x1b[35m${text}\x1b[0m`,
    blue: (text) => `\x1b[34m${text}\x1b[0m`,
    gray: (text) => `\x1b[90m${text}\x1b[0m`,
};
function printBanner() {
    console.log(colors.cyan(colors.bold(`
┌─────────────────────────────────────────────────────────────┐
│  ⚡ ARCH-REFACTOR: Rust & Oxc Architectural Refactor Engine  │
└─────────────────────────────────────────────────────────────┘`)));
}
function printUsage() {
    printBanner();
    console.log(`
${colors.bold('USAGE:')}
  ${colors.green('arch-refactor')} ${colors.cyan('<command>')} [options] [target_path]

${colors.bold('COMMANDS:')}
  ${colors.cyan('scan')} [path]             Analyze repository architecture, frameworks & AST graph
  ${colors.cyan('plan')} [path]             Generate an architectural migration plan & import map
  ${colors.cyan('diff')} [path]             Preview unified diffs for a proposed refactor
  ${colors.cyan('apply')} [path]            Execute refactoring plan atomically with rollback journal
  ${colors.cyan('detect-clones')} [path]    Detect duplicated logic clusters & extractable utilities
  ${colors.cyan('undo')} [path]             Roll back the last refactor using .refactor-journal.json

${colors.bold('OPTIONS:')}
  ${colors.yellow('--architecture, -a')}    Target layout: ${colors.green('feature-based')} (default), ${colors.green('ddd')}, ${colors.green('layered')}
  ${colors.yellow('--naming, -n')}          Naming style: ${colors.green('kebab-case')} (default), ${colors.green('pascal-case')}, ${colors.green('camel-case')}, ${colors.green('snake-case')}, ${colors.green('preserve')}
  ${colors.yellow('--dry-run')}             Simulate transformation without modifying files on disk
  ${colors.yellow('--force, -f')}           Bypass Git working tree cleanliness check
  ${colors.yellow('--yes, -y')}             Skip interactive confirmation prompt
  ${colors.yellow('--min-lines')}           Minimum line threshold for code clone detection (default: 3)
  ${colors.yellow('--help, -h')}            Show this help message
  ${colors.yellow('--version, -v')}         Print version
`);
}
function parseArgs(args) {
    const flags = {};
    const positional = [];
    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg.startsWith('--')) {
            const parts = arg.slice(2).split('=');
            const key = parts[0];
            if (parts.length > 1) {
                flags[key] = parts[1];
            }
            else if (i + 1 < args.length && !args[i + 1].startsWith('-')) {
                flags[key] = args[i + 1];
                i++;
            }
            else {
                flags[key] = true;
            }
        }
        else if (arg.startsWith('-')) {
            const char = arg.slice(1);
            if (char === 'a')
                flags['architecture'] = args[++i];
            else if (char === 'n')
                flags['naming'] = args[++i];
            else if (char === 'f')
                flags['force'] = true;
            else if (char === 'y')
                flags['yes'] = true;
            else if (char === 'h')
                flags['help'] = true;
            else if (char === 'v')
                flags['version'] = true;
            else
                flags[char] = true;
        }
        else {
            positional.push(arg);
        }
    }
    return { flags, positional };
}
function resolveArchitectureTarget(arch) {
    if (typeof arch !== 'string')
        return 'FeatureBased';
    const val = arch.toLowerCase().replace(/[-_]/g, '');
    if (val.includes('ddd') || val.includes('domain'))
        return 'DomainDrivenDesign';
    if (val.includes('layer'))
        return 'Layered';
    return 'FeatureBased';
}
function resolveNamingConvention(naming) {
    if (typeof naming !== 'string')
        return 'KebabCase';
    const val = naming.toLowerCase().replace(/[-_]/g, '');
    if (val.includes('pascal'))
        return 'PascalCase';
    if (val.includes('camel'))
        return 'CamelCase';
    if (val.includes('snake'))
        return 'SnakeCase';
    if (val.includes('preserve'))
        return 'Preserve';
    return 'KebabCase';
}
async function promptConfirmation(question) {
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
    });
    return new Promise((resolve) => {
        rl.question(`${question} (y/N): `, (answer) => {
            rl.close();
            resolve(answer.trim().toLowerCase() === 'y' || answer.trim().toLowerCase() === 'yes');
        });
    });
}
async function handleScan(targetPath) {
    printBanner();
    console.log(colors.bold(`\n🔍 Scanning repository at: ${colors.cyan(targetPath)}...`));
    const startTime = Date.now();
    const config = {
        rootPath: targetPath,
    };
    const result = index_1.RefactoringEngine.scan(config);
    const duration = Date.now() - startTime;
    console.log(`\n${colors.green('✔')} Scan completed in ${colors.bold(`${duration}ms`)}\n`);
    console.log(`${colors.bold('Framework Detected:')}   ${colors.magenta(result.framework)} - ${result.frameworkDescription}`);
    console.log(`${colors.bold('Total Files:')}          ${colors.cyan(result.totalFiles.toString())}`);
    console.log(`${colors.bold('Total Source LOC:')}     ${colors.cyan(result.totalLines.toString())}`);
    console.log(`${colors.bold('Dependency Edges:')}     ${colors.cyan(result.dependencyGraph.totalEdges.toString())}`);
    // TSConfig info
    const aliasKeys = Object.keys(result.tsconfig.paths);
    if (aliasKeys.length > 0) {
        console.log(`\n${colors.bold('TSConfig Path Aliases:')}`);
        for (const key of aliasKeys) {
            console.log(`  • ${colors.yellow(key)} ➔ ${result.tsconfig.paths[key].join(', ')}`);
        }
    }
    // Circular dependencies
    if (result.dependencyGraph.circularCycles.length > 0) {
        console.log(`\n${colors.red(colors.bold(`⚠️  Circular Dependencies Detected (${result.dependencyGraph.circularCycles.length} cycles):`))}`);
        result.dependencyGraph.circularCycles.forEach((cycle, idx) => {
            console.log(`  ${colors.yellow(`Cycle #${idx + 1}`)} (${cycle.cycleLength} files):`);
            cycle.files.forEach((f, fIdx) => {
                const arrow = fIdx === cycle.files.length - 1 ? '↺' : '↓';
                console.log(`    ${arrow} ${colors.dim(path.relative(targetPath, f))}`);
            });
        });
    }
    else {
        console.log(`\n${colors.green('✔')} ${colors.bold('Clean AST Graph:')} No circular dependencies detected.`);
    }
    // Protected framework boundaries
    const protectedFiles = result.files.filter((f) => f.frameworkBoundary.isProtectedRoute);
    if (protectedFiles.length > 0) {
        console.log(`\n${colors.bold('Framework Boundary Protected Routes:')} ${colors.yellow(`(${protectedFiles.length} files)`)}`);
        protectedFiles.slice(0, 10).forEach((f) => {
            console.log(`  🔒 ${colors.cyan(f.relativePath)} ${colors.gray(`[${f.frameworkBoundary.boundaryType}]`)}`);
        });
        if (protectedFiles.length > 10) {
            console.log(`  ${colors.dim(`... and ${protectedFiles.length - 10} more protected routes`)}`);
        }
    }
}
async function handlePlan(targetPath, flags) {
    printBanner();
    const targetArch = resolveArchitectureTarget(flags['architecture']);
    const namingConv = resolveNamingConvention(flags['naming']);
    console.log(colors.bold(`\n📐 Generating refactor plan for: ${colors.cyan(targetPath)}`));
    console.log(`${colors.bold('Target Architecture:')} ${colors.magenta(targetArch)}`);
    console.log(`${colors.bold('Naming Convention:')}   ${colors.magenta(namingConv)}`);
    const planConfig = {
        rootPath: targetPath,
        targetArchitecture: targetArch,
        namingConvention: namingConv,
    };
    const plan = index_1.RefactoringEngine.plan(planConfig);
    console.log(`\n${colors.green('✔')} Plan generated successfully:\n`);
    console.log(`  • Files to move / rename:    ${colors.bold(colors.yellow(plan.summary.totalFilesMoved.toString()))}`);
    console.log(`  • Import specifiers to patch: ${colors.bold(colors.cyan(plan.summary.totalImportsPatched.toString()))}`);
    console.log(`  • Framework protected files:  ${colors.bold(colors.green(plan.summary.totalProtectedFiles.toString()))}`);
    if (plan.fileMoves.length > 0) {
        console.log(`\n${colors.bold('Planned File Movements (Sample):')}`);
        plan.fileMoves.slice(0, 12).forEach((m) => {
            console.log(`  ${colors.red(m.originalRelativePath)} ➔ ${colors.green(m.newRelativePath)}`);
        });
        if (plan.fileMoves.length > 12) {
            console.log(`  ${colors.dim(`... and ${plan.fileMoves.length - 12} more movements`)}`);
        }
    }
    console.log(`\n${colors.dim('Run `arch-refactor diff` to inspect unified diffs, or `arch-refactor apply` to execute.')}`);
}
async function handleDiff(targetPath, flags) {
    const targetArch = resolveArchitectureTarget(flags['architecture']);
    const namingConv = resolveNamingConvention(flags['naming']);
    const planConfig = {
        rootPath: targetPath,
        targetArchitecture: targetArch,
        namingConvention: namingConv,
    };
    const plan = index_1.RefactoringEngine.plan(planConfig);
    const diffResult = index_1.RefactoringEngine.previewDiff(plan);
    printBanner();
    console.log(colors.bold(`\n📑 Unified Diff Preview (${diffResult.totalFilesChanged} files changed, +${diffResult.totalAdditions} -${diffResult.totalDeletions}):\n`));
    for (const diff of diffResult.fileDiffs) {
        console.log(colors.bold(colors.cyan(`=== File: ${diff.relativePath} ${diff.isMoved ? `(Moved to ${diff.newPath})` : ''} ===`)));
        for (const hunk of diff.hunks) {
            console.log(colors.magenta(hunk.header));
            for (const line of hunk.lines) {
                if (line.startsWith('+')) {
                    console.log(colors.green(line));
                }
                else if (line.startsWith('-')) {
                    console.log(colors.red(line));
                }
                else {
                    console.log(colors.gray(line));
                }
            }
        }
        console.log('');
    }
}
async function handleApply(targetPath, flags) {
    printBanner();
    const targetArch = resolveArchitectureTarget(flags['architecture']);
    const namingConv = resolveNamingConvention(flags['naming']);
    const dryRun = Boolean(flags['dry-run']);
    const force = Boolean(flags['force']);
    const autoYes = Boolean(flags['yes']);
    // Git status guardrails check
    const gitStatus = index_1.RefactoringEngine.getGitStatus(targetPath);
    if (gitStatus.isGitRepo && !gitStatus.isClean && !force) {
        console.log(colors.red(`\n❌ Git Guardrail Alert: Working tree has ${gitStatus.modifiedFiles.length} uncommitted modifications.`));
        console.log(`   Modified files: ${gitStatus.modifiedFiles.slice(0, 5).join(', ')}${gitStatus.modifiedFiles.length > 5 ? '...' : ''}`);
        console.log(`   Please commit or stash your changes before refactoring, or pass ${colors.yellow('--force')} to override.\n`);
        process.exit(1);
    }
    const planConfig = {
        rootPath: targetPath,
        targetArchitecture: targetArch,
        namingConvention: namingConv,
    };
    const plan = index_1.RefactoringEngine.plan(planConfig);
    console.log(`\n${colors.bold('Refactoring Plan Summary:')}`);
    console.log(`  • Target Architecture: ${colors.magenta(targetArch)}`);
    console.log(`  • Naming Convention:   ${colors.magenta(namingConv)}`);
    console.log(`  • Files to Move:       ${colors.yellow(plan.summary.totalFilesMoved.toString())}`);
    console.log(`  • Imports to Rewire:   ${colors.cyan(plan.summary.totalImportsPatched.toString())}`);
    if (plan.summary.totalFilesMoved === 0 && plan.summary.totalImportsPatched === 0) {
        console.log(`\n${colors.green('✔ Repository is already fully compliant with the target architecture!')}`);
        return;
    }
    if (!dryRun && !autoYes) {
        const confirmed = await promptConfirmation('\nDo you want to apply these architectural refactoring changes now?');
        if (!confirmed) {
            console.log(colors.yellow('\nRefactoring cancelled.'));
            return;
        }
    }
    const applyOpts = {
        dryRun,
        force,
        skipGitCheck: force,
    };
    const applyResult = index_1.RefactoringEngine.apply(plan, applyOpts);
    if (applyResult.success) {
        console.log(`\n${colors.green(colors.bold('✔ Transformation Applied Successfully!'))}`);
        console.log(`  • Transaction ID: ${colors.bold(applyResult.transactionId)}`);
        console.log(`  • Files Moved:    ${applyResult.filesMoved}`);
        console.log(`  • Files Patched:  ${applyResult.filesPatched}`);
        if (applyResult.journalPath) {
            console.log(`  • Rollback Log:   ${colors.cyan(applyResult.journalPath)}`);
        }
        console.log(`\n${colors.dim('To revert changes at any time, run `arch-refactor undo`')}`);
    }
    else {
        console.log(colors.red(`\n❌ Error applying refactor: ${applyResult.message}`));
        process.exit(1);
    }
}
async function handleDetectClones(targetPath, flags) {
    printBanner();
    const minLines = typeof flags['min-lines'] === 'string' ? parseInt(flags['min-lines'], 10) : 3;
    console.log(colors.bold(`\n🔬 Analyzing repository for AST code clones & redundant utilities at: ${colors.cyan(targetPath)}...`));
    const startTime = Date.now();
    const config = {
        rootPath: targetPath,
        minLines,
    };
    const result = index_1.RefactoringEngine.detectClones(config);
    const duration = Date.now() - startTime;
    console.log(`\n${colors.green('✔')} Clone analysis completed in ${colors.bold(`${duration}ms`)}\n`);
    console.log(`  • Duplicate Clusters Found: ${colors.bold(colors.yellow(result.totalClusters.toString()))}`);
    console.log(`  • Total Cloned Blocks:      ${colors.bold(colors.red(result.totalClonesFound.toString()))}`);
    console.log(`  • Potential Lines Saved:    ${colors.bold(colors.green(result.totalLinesSaved.toString()))}\n`);
    if (result.clusters.length === 0) {
        console.log(colors.green('✔ No significant duplicated code blocks detected. Repository utilities are clean!'));
        return;
    }
    result.clusters.forEach((cluster, idx) => {
        console.log(colors.bold(colors.cyan(`[Cluster #${idx + 1}] Duplicate Function/Block: "${cluster.suggestedModuleName}" (${cluster.instanceCount} occurrences, saves ~${cluster.potentialLinesSaved} lines)`)));
        console.log(`  💡 Suggested Extraction Target: ${colors.green(cluster.suggestedTargetPath)}`);
        console.log(`  Locations:`);
        cluster.instances.forEach((inst) => {
            console.log(`    • ${colors.yellow(inst.relativePath)}:${inst.startLine}-${inst.endLine}`);
        });
        console.log('');
    });
}
async function handleUndo(targetPath) {
    printBanner();
    console.log(colors.bold(`\n⏪ Rolling back last refactor transaction...`));
    const result = index_1.RefactoringEngine.rollback(undefined, targetPath);
    if (result.success) {
        console.log(`\n${colors.green(colors.bold('✔ Rollback Completed Successfully!'))}`);
        console.log(`  • Transaction ID:  ${colors.bold(result.transactionId)}`);
        console.log(`  • Restored Files:  ${colors.bold(result.restoredFilesCount.toString())}`);
        console.log(`  • Message:         ${result.message}`);
    }
    else {
        console.log(colors.red(`\n❌ Rollback failed: ${result.message}`));
        process.exit(1);
    }
}
async function main() {
    const args = process.argv.slice(2);
    const { flags, positional } = parseArgs(args);
    if (flags['version'] || flags['v']) {
        const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, '../package.json'), 'utf8'));
        console.log(`arch-refactor v${pkg.version}`);
        return;
    }
    if (flags['help'] || flags['h'] || positional.length === 0) {
        printUsage();
        return;
    }
    const command = positional[0];
    const targetPath = path.resolve(positional[1] || process.cwd());
    try {
        switch (command) {
            case 'scan':
                await handleScan(targetPath);
                break;
            case 'plan':
                await handlePlan(targetPath, flags);
                break;
            case 'diff':
                await handleDiff(targetPath, flags);
                break;
            case 'apply':
                await handleApply(targetPath, flags);
                break;
            case 'detect-clones':
                await handleDetectClones(targetPath, flags);
                break;
            case 'undo':
            case 'rollback':
                await handleUndo(targetPath);
                break;
            default:
                console.log(colors.red(`Unknown command: "${command}"`));
                printUsage();
                process.exit(1);
        }
    }
    catch (err) {
        console.error(colors.red(`\nFatal Error: ${err?.message || err}`));
        process.exit(1);
    }
}
main();
