import * as path from 'path';
import * as fs from 'fs';

interface NativeBinding {
  scanRepository(config: any): any;
  generatePlan(config: any): any;
  previewDiff(plan: any): any;
  applyRefactor(plan: any, options?: any): any;
  rollbackRefactor(journalPath?: string, rootPath?: string): any;
  detectClones(config: any): any;
  getGitStatus(rootPath: string): any;
}

let nativeBinding: NativeBinding | null = null;

function loadBinding(): NativeBinding {
  if (nativeBinding) {
    return nativeBinding;
  }

  const rootDir = path.resolve(__dirname, '..');
  const platform = process.platform;
  const arch = process.arch;

  // Candidate paths for local and package installations
  const candidates: string[] = [
    // Direct local root build
    path.join(rootDir, 'arch_refactor_engine.node'),
    path.join(rootDir, `arch_refactor_engine.${platform}-${arch}-gnu.node`),
    path.join(rootDir, `arch_refactor_engine.${platform}-${arch}.node`),
    // Platform packages in npm/
    path.join(rootDir, 'npm', `${platform}-${arch}-gnu`, `arch_refactor_engine.${platform}-${arch}-gnu.node`),
    path.join(rootDir, 'npm', `${platform}-${arch}`, `arch_refactor_engine.${platform}-${arch}.node`),
  ];

  // Also check specific target triples
  if (platform === 'linux') {
    if (arch === 'x64') {
      candidates.push(path.join(rootDir, 'arch_refactor_engine.linux-x64-gnu.node'));
      candidates.push(path.join(rootDir, 'arch_refactor_engine.linux-x64-musl.node'));
    } else if (arch === 'arm64') {
      candidates.push(path.join(rootDir, 'arch_refactor_engine.linux-arm64-gnu.node'));
    }
  } else if (platform === 'darwin') {
    if (arch === 'arm64') {
      candidates.push(path.join(rootDir, 'arch_refactor_engine.darwin-arm64.node'));
    } else if (arch === 'x64') {
      candidates.push(path.join(rootDir, 'arch_refactor_engine.darwin-x64.node'));
    }
  } else if (platform === 'win32' && arch === 'x64') {
    candidates.push(path.join(rootDir, 'arch_refactor_engine.win32-x64-msvc.node'));
  }

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      try {
        nativeBinding = require(candidate);
        return nativeBinding!;
      } catch (err) {
        // Continue to try next candidate
      }
    }
  }

  // Try standard npm package requirement
  try {
    const pkgName = `@arch-refactor/engine-${platform}-${arch}`;
    nativeBinding = require(pkgName);
    return nativeBinding!;
  } catch (_) {
    // Try with -gnu suffix on linux
    if (platform === 'linux') {
      try {
        const pkgName = `@arch-refactor/engine-linux-${arch}-gnu`;
        nativeBinding = require(pkgName);
        return nativeBinding!;
      } catch (_) {}
    }
  }

  throw new Error(
    `Failed to load native arch_refactor_engine addon for platform ${platform} (${arch}).\n` +
      `Ensure that you have run \`npm run build:native\` or installed the proper platform binary.`
  );
}

export const native = loadBinding();
