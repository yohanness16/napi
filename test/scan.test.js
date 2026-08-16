const { test, describe, before, after } = require('node:test');
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { RefactoringEngine } = require('../lib/index');

describe('Module 1 & 2: Scanner & Framework Boundary Detector', () => {
  let tempDir;

  before(() => {
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'arch-scan-test-'));

    // Create a mock Next.js App Router structure
    const appDir = path.join(tempDir, 'app');
    const dashboardDir = path.join(appDir, 'dashboard');
    const componentsDir = path.join(tempDir, 'components');
    const utilsDir = path.join(tempDir, 'utils');

    fs.mkdirSync(dashboardDir, { recursive: true });
    fs.mkdirSync(componentsDir, { recursive: true });
    fs.mkdirSync(utilsDir, { recursive: true });

    // Next.js package.json & tsconfig.json
    fs.writeFileSync(
      path.join(tempDir, 'package.json'),
      JSON.stringify({ name: 'mock-next-app', dependencies: { next: '14.0.0', react: '18.0.0' } })
    );
    fs.writeFileSync(
      path.join(tempDir, 'tsconfig.json'),
      JSON.stringify({
        compilerOptions: {
          baseUrl: '.',
          paths: {
            '@/*': ['./*'],
          },
        },
      })
    );

    // Protected route files
    fs.writeFileSync(
      path.join(appDir, 'layout.tsx'),
      `export default function RootLayout({ children }: { children: React.ReactNode }) { return <html><body>{children}</body></html>; }`
    );
    fs.writeFileSync(
      path.join(appDir, 'page.tsx'),
      `import { Header } from '../components/Header';\nexport default function HomePage() { return <div><Header /></div>; }`
    );
    fs.writeFileSync(
      path.join(dashboardDir, 'page.tsx'),
      `"use client";\nimport { formatDate } from '@/utils/date';\nexport default function DashboardPage() { return <div>Date: {formatDate()}</div>; }`
    );

    // Refactorable components and utilities
    fs.writeFileSync(
      path.join(componentsDir, 'Header.tsx'),
      `export const Header = () => <header>Logo</header>;`
    );
    fs.writeFileSync(
      path.join(utilsDir, 'date.ts'),
      `export function formatDate(): string { return new Date().toISOString(); }`
    );
  });

  after(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  test('detects Next.js App Router framework and rules correctly', () => {
    const result = RefactoringEngine.scan({ rootPath: tempDir });

    assert.strictEqual(result.framework, 'NextAppRouter');
    assert.strictEqual(result.totalFiles, 5);

    // Verify protected boundaries
    const layout = result.files.find((f) => f.relativePath.includes('layout.tsx'));
    assert.ok(layout);
    assert.strictEqual(layout.frameworkBoundary.isProtectedRoute, true);
    assert.strictEqual(layout.frameworkBoundary.boundaryType, 'NextAppRouter:layout');

    const dashboardPage = result.files.find((f) => f.relativePath === 'app/dashboard/page.tsx' || f.relativePath === 'app\\dashboard\\page.tsx');
    assert.ok(dashboardPage);
    assert.strictEqual(dashboardPage.frameworkBoundary.isProtectedRoute, true);
    assert.strictEqual(dashboardPage.frameworkBoundary.directive, 'use client');

    // Verify non-protected module
    const header = result.files.find((f) => f.relativePath.includes('Header.tsx'));
    assert.ok(header);
    assert.strictEqual(header.frameworkBoundary.isProtectedRoute, false);
  });

  test('resolves tsconfig path aliases and relative imports in dependency graph', () => {
    const result = RefactoringEngine.scan({ rootPath: tempDir });

    assert.ok(result.dependencyGraph.totalEdges >= 2);

    // Check alias resolution for @/utils/date
    const dashboardPage = result.files.find((f) => f.relativePath.includes('dashboard/page.tsx') || f.relativePath.includes('dashboard\\page.tsx'));
    assert.ok(dashboardPage);
    const dateImport = dashboardPage.imports.find((i) => i.specifier === '@/utils/date');
    assert.ok(dateImport);
    assert.ok(dateImport.resolvedPath && dateImport.resolvedPath.includes('utils/date.ts') || dateImport.resolvedPath.includes('utils\\date.ts'));
  });
});
