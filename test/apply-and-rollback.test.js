const { test, describe, before, after } = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { RefactoringEngine } = require('../lib/index');

describe('Module 6: Safety, Transaction Engine & Rollback Journal', () => {
  let tempDir;
  let compPath;
  let hookPath;

  before(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'arch-apply-test-'));

    const srcDir = path.join(tempDir, 'src');
    const compDir = path.join(srcDir, 'components');
    const hooksDir = path.join(srcDir, 'hooks');

    fs.mkdirSync(compDir, { recursive: true });
    fs.mkdirSync(hooksDir, { recursive: true });

    compPath = path.join(compDir, 'UserButton.tsx');
    hookPath = path.join(hooksDir, 'useAuth.ts');

    fs.writeFileSync(
      hookPath,
      `export function useAuth() { return { isAuth: true }; }`
    );
    fs.writeFileSync(
      compPath,
      `import { useAuth } from '../hooks/useAuth';\nexport const UserButton = () => <button>{useAuth().isAuth ? 'Logout' : 'Login'}</button>;`
    );
  });

  after(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  test('applies refactoring atomically, moves files, patches AST imports, and records journal', () => {
    const plan = RefactoringEngine.plan({
      rootPath: tempDir,
      targetArchitecture: 'FeatureBased',
      namingConvention: 'KebabCase',
    });

    const applyResult = RefactoringEngine.apply(plan, {
      dryRun: false,
      force: true,
      skipGitCheck: true,
    });

    assert.strictEqual(applyResult.success, true);
    assert.ok(applyResult.filesMoved > 0);
    assert.ok(applyResult.journalPath);
    assert.ok(fs.existsSync(applyResult.journalPath));

    // Verify original files were moved
    assert.strictEqual(fs.existsSync(compPath), false);
    assert.strictEqual(fs.existsSync(hookPath), false);

    // Verify new file exists and import has been patched
    const movedButton = path.join(tempDir, 'src', 'shared', 'components', 'user-button.tsx');
    assert.ok(fs.existsSync(movedButton));

    const content = fs.readFileSync(movedButton, 'utf8');
    assert.ok(!content.includes('../hooks/useAuth'));
    assert.ok(content.includes('use-auth'));
  });

  test('rolls back entire refactor restoring original files and removing new files', () => {
    const rollbackResult = RefactoringEngine.rollback(undefined, tempDir);

    assert.strictEqual(rollbackResult.success, true);
    assert.ok(rollbackResult.restoredFilesCount > 0);

    // Original files must be restored
    assert.ok(fs.existsSync(compPath));
    assert.ok(fs.existsSync(hookPath));

    const restoredButtonContent = fs.readFileSync(compPath, 'utf8');
    assert.ok(restoredButtonContent.includes("import { useAuth } from '../hooks/useAuth'"));

    // Moved file should be deleted
    const movedButton = path.join(tempDir, 'src', 'shared', 'components', 'user-button.tsx');
    assert.strictEqual(fs.existsSync(movedButton), false);
  });
});
