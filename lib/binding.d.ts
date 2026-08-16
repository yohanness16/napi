interface NativeBinding {
    scanRepository(config: any): any;
    generatePlan(config: any): any;
    previewDiff(plan: any): any;
    applyRefactor(plan: any, options?: any): any;
    rollbackRefactor(journalPath?: string, rootPath?: string): any;
    detectClones(config: any): any;
    getGitStatus(rootPath: string): any;
}
export declare const native: NativeBinding;
export {};
//# sourceMappingURL=binding.d.ts.map