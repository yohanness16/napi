const { test, describe, before, after } = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { RefactoringEngine } = require('../lib/index');

describe('Module 3 & 4: Architectural Planner & Diff Preview', () => {
  let tempDir;

  before(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'arch-plan-test-'));

    const srcDir = path.join(tempDir, 'src');
    const compDir = path.join(srcDir, 'components');
    const authDir = path.join(srcDir, 'auth');

    fs.mkdirSync(compDir, { recursive: true });
    fs.mkdirSync(authDir, { recursive: true });

    // Components and auth logic
    fs.writeFileSync(
      path.join(compDir, 'UserProfileCard.tsx'),
      `import { getSession } from '../auth/sessionService';\nexport const UserProfileCard = () => <div>{getSession()}</div>;`
    );
    fs.writeFileSync(
      path.join(authDir, 'sessionService.ts'),
      `export function getSession(): string { return 'user_123'; }`
    );
  });

  after(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  test('generates FeatureBased migration plan with KebabCase naming', () => {
    const plan = RefactoringEngine.plan({
      rootPath: tempDir,
      targetArchitecture: 'FeatureBased',
      namingConvention: 'KebabCase',
    });

    assert.ok(plan.summary.totalFilesMoved > 0);
    assert.ok(plan.fileMoves.length > 0);

    const userCardMove = plan.fileMoves.find((m) => m.originalRelativePath.includes('UserProfileCard.tsx'));
    assert.ok(userCardMove);
    assert.ok(userCardMove.newRelativePath.includes('user-profile-card.tsx'));

    const sessionMove = plan.fileMoves.find((m) => m.originalRelativePath.includes('sessionService.ts'));
    assert.ok(sessionMove);
    assert.ok(sessionMove.newRelativePath.includes('session-service.ts'));
  });

  test('generates unified diff previews for planned refactoring changes', () => {
    const plan = RefactoringEngine.plan({
      rootPath: tempDir,
      targetArchitecture: 'FeatureBased',
      namingConvention: 'KebabCase',
    });

    const diffPreview = RefactoringEngine.previewDiff(plan);

    assert.ok(diffPreview.totalFilesChanged > 0);
    assert.ok(diffPreview.fileDiffs.length > 0);

    const cardDiff = diffPreview.fileDiffs.find((d) => d.relativePath.includes('UserProfileCard.tsx'));
    assert.ok(cardDiff);
    assert.ok(cardDiff.unifiedDiff.includes('---'));
    assert.ok(cardDiff.unifiedDiff.includes('+++'));
  });
});
