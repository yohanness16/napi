const { test, describe, before, after } = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { RefactoringEngine } = require('../lib/index');

describe('Module 5: Code Clone & Utility Deduplication Detector', () => {
  let tempDir;

  before(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'arch-clones-test-'));

    const mod1 = path.join(tempDir, 'featureA.ts');
    const mod2 = path.join(tempDir, 'featureB.ts');

    // Duplicate utility functions across 2 modules
    const duplicateFunc = `
export function formatCurrency(amount: number, currency: string): string {
    const formatted = amount.toFixed(2);
    const prefix = currency === 'USD' ? '$' : '€';
    return prefix + formatted;
}
`;

    fs.writeFileSync(mod1, `const a = 10;\n${duplicateFunc}\nexport const x = 1;`);
    fs.writeFileSync(mod2, `const b = 20;\n${duplicateFunc}\nexport const y = 2;`);
  });

  after(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  test('detects code clone clusters and suggests deduplication target', () => {
    const result = RefactoringEngine.detectClones({
      rootPath: tempDir,
      minLines: 3,
      minAstNodes: 5,
    });

    assert.ok(result.totalClusters >= 1);
    assert.ok(result.totalClonesFound >= 2);
    assert.ok(result.totalLinesSaved > 0);

    const cluster = result.clusters[0];
    assert.strictEqual(cluster.instances.length, 2);
    assert.strictEqual(cluster.suggestedModuleName, 'formatCurrency');
    assert.ok(cluster.suggestedTargetPath.includes('format-currency.ts'));
  });
});
