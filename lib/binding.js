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
exports.native = void 0;
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
let nativeBinding = null;
function loadBinding() {
    if (nativeBinding) {
        return nativeBinding;
    }
    const rootDir = path.resolve(__dirname, '..');
    const platform = process.platform;
    const arch = process.arch;
    // Candidate paths for local and package installations
    const candidates = [
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
        }
        else if (arch === 'arm64') {
            candidates.push(path.join(rootDir, 'arch_refactor_engine.linux-arm64-gnu.node'));
        }
    }
    else if (platform === 'darwin') {
        if (arch === 'arm64') {
            candidates.push(path.join(rootDir, 'arch_refactor_engine.darwin-arm64.node'));
        }
        else if (arch === 'x64') {
            candidates.push(path.join(rootDir, 'arch_refactor_engine.darwin-x64.node'));
        }
    }
    else if (platform === 'win32' && arch === 'x64') {
        candidates.push(path.join(rootDir, 'arch_refactor_engine.win32-x64-msvc.node'));
    }
    for (const candidate of candidates) {
        if (fs.existsSync(candidate)) {
            try {
                nativeBinding = require(candidate);
                return nativeBinding;
            }
            catch (err) {
                // Continue to try next candidate
            }
        }
    }
    // Try standard npm package requirement
    try {
        const pkgName = `@arch-refactor/engine-${platform}-${arch}`;
        nativeBinding = require(pkgName);
        return nativeBinding;
    }
    catch (_) {
        // Try with -gnu suffix on linux
        if (platform === 'linux') {
            try {
                const pkgName = `@arch-refactor/engine-linux-${arch}-gnu`;
                nativeBinding = require(pkgName);
                return nativeBinding;
            }
            catch (_) { }
        }
    }
    throw new Error(`Failed to load native arch_refactor_engine addon for platform ${platform} (${arch}).\n` +
        `Ensure that you have run \`npm run build:native\` or installed the proper platform binary.`);
}
exports.native = loadBinding();
//# sourceMappingURL=binding.js.map