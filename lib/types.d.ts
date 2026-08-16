export type FrameworkType = 'NextAppRouter' | 'NextPagesRouter' | 'Remix' | 'Vite' | 'React' | 'Vue' | 'NestJs' | 'Express' | 'Generic';
export type ArchitectureTarget = 'FeatureBased' | 'DomainDrivenDesign' | 'Layered' | 'Custom';
export type NamingConvention = 'KebabCase' | 'PascalCase' | 'CamelCase' | 'SnakeCase' | 'Preserve';
export type ImportKind = 'StaticImport' | 'DynamicImport' | 'RequireCall' | 'ExportFrom' | 'ExportAll' | 'TypeOnlyImport';
export interface SpanInfo {
    start: number;
    end: number;
    line: number;
    column: number;
}
export interface ImportDeclarationInfo {
    specifier: string;
    rawSpecifier: string;
    span: SpanInfo;
    kind: ImportKind;
    resolvedPath?: string | null;
    isExternal: boolean;
    isTypeOnly: boolean;
}
export interface FrameworkBoundaryInfo {
    isBoundary: boolean;
    isProtectedRoute: boolean;
    boundaryType: string;
    description: string;
    directive?: string | null;
}
export interface FileInfo {
    path: string;
    relativePath: string;
    fileName: string;
    extension: string;
    sizeBytes: number;
    lineCount: number;
    frameworkBoundary: FrameworkBoundaryInfo;
    imports: ImportDeclarationInfo[];
    exportedSymbols: string[];
}
export interface TsConfigInfo {
    baseUrl?: string | null;
    paths: Record<string, string[]>;
}
export interface ScanConfig {
    rootPath: string;
    ignorePatterns?: string[];
    tsconfigPath?: string;
}
export interface CircularCycle {
    files: string[];
    cycleLength: number;
}
export interface DependencyGraphNode {
    filePath: string;
    relativePath: string;
    dependencies: string[];
    dependents: string[];
    fanIn: number;
    fanOut: number;
    isCircular: boolean;
    isOrphan: boolean;
}
export interface DependencyGraphResult {
    totalNodes: number;
    totalEdges: number;
    nodes: DependencyGraphNode[];
    circularCycles: CircularCycle[];
    orphanFiles: string[];
}
export interface RepositoryScanResult {
    rootPath: string;
    framework: FrameworkType;
    frameworkDescription: string;
    totalFiles: number;
    totalLines: number;
    files: FileInfo[];
    dependencyGraph: DependencyGraphResult;
    tsconfig: TsConfigInfo;
}
export interface FileMoveAction {
    originalPath: string;
    originalRelativePath: string;
    newPath: string;
    newRelativePath: string;
    reason: string;
    isProtectedFrameworkFile: boolean;
}
export interface PatchAction {
    filePath: string;
    spanStart: number;
    spanEnd: number;
    originalSpecifier: string;
    replacementSpecifier: string;
    reason: string;
}
export interface PlanConfig {
    rootPath: string;
    targetArchitecture: ArchitectureTarget;
    namingConvention: NamingConvention;
    customFeatureMappings?: Record<string, string>;
    tsconfigPath?: string;
}
export interface RefactorSummary {
    totalFilesMoved: number;
    totalImportsPatched: number;
    totalProtectedFiles: number;
    targetArchitecture: ArchitectureTarget;
    namingConvention: NamingConvention;
}
export interface RefactorPlan {
    rootPath: string;
    targetArchitecture: ArchitectureTarget;
    namingConvention: NamingConvention;
    fileMoves: FileMoveAction[];
    patches: PatchAction[];
    protectedFiles: string[];
    summary: RefactorSummary;
}
export interface DiffHunk {
    oldStart: number;
    oldLines: number;
    newStart: number;
    newLines: number;
    header: string;
    lines: string[];
}
export interface FileDiff {
    filePath: string;
    relativePath: string;
    isNewFile: boolean;
    isDeletedFile: boolean;
    isMoved: boolean;
    oldPath?: string | null;
    newPath?: string | null;
    unifiedDiff: string;
    additions: number;
    deletions: number;
    hunks: DiffHunk[];
}
export interface DiffPreviewResult {
    totalFilesChanged: number;
    totalAdditions: number;
    totalDeletions: number;
    fileDiffs: FileDiff[];
}
export interface ApplyOptions {
    dryRun?: boolean;
    force?: boolean;
    skipGitCheck?: boolean;
    journalDir?: string;
}
export interface ApplyResult {
    success: boolean;
    filesMoved: number;
    filesPatched: number;
    journalPath?: string | null;
    transactionId: string;
    message: string;
}
export interface RollbackResult {
    success: boolean;
    restoredFilesCount: number;
    transactionId: string;
    message: string;
}
export interface GitStatusResult {
    isGitRepo: boolean;
    isClean: boolean;
    gitRoot?: string | null;
    branch?: string | null;
    modifiedFiles: string[];
    untrackedFiles: string[];
    warning?: string | null;
}
export interface CodeCloneInstance {
    filePath: string;
    relativePath: string;
    startLine: number;
    endLine: number;
    startByte: number;
    endByte: number;
    functionName?: string | null;
    codeSnippet: string;
}
export interface CloneCluster {
    clusterId: string;
    hash: string;
    instanceCount: number;
    linesPerInstance: number;
    potentialLinesSaved: number;
    astNodeCount: number;
    instances: CodeCloneInstance[];
    suggestedModuleName: string;
    suggestedTargetPath: string;
}
export interface CloneDetectionConfig {
    rootPath: string;
    minLines?: number;
    minAstNodes?: number;
    ignorePatterns?: string[];
}
export interface CloneDetectionResult {
    totalClonesFound: number;
    totalClusters: number;
    totalLinesSaved: number;
    clusters: CloneCluster[];
}
//# sourceMappingURL=types.d.ts.map