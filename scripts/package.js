#!/usr/bin/env node

/**
 * Multi-platform packaging script for oh-my-file
 * Generates installers for Windows (.msi/.nsis), macOS (.dmg), and Linux (.appimage/.deb)
 */

import { spawn, spawnSync } from 'child_process';
import { platform, arch } from 'os';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, '..');

const currentPlatform = platform();
const currentArch = arch();
const isWindows = currentPlatform === 'win32';

// Platform installer configurations
const platformConfigs = {
  win32: {
    name: 'Windows',
    targets: ['msi', 'nsis'],
    description: 'Generates .msi (Windows Installer) and portable .exe',
  },
  darwin: {
    name: 'macOS',
    targets: ['dmg', 'app'],
    description: 'Generates .dmg (Disk Image) and .app (Application Bundle)',
  },
  linux: {
    name: 'Linux',
    targets: ['appimage', 'deb'],
    description: 'Generates .AppImage (portable) and .deb (Debian package)',
  },
};

const command = isWindows ? 'npm.cmd' : 'npm';

async function runCommand(args, description) {
  return new Promise((resolve, reject) => {
    console.log(`\n📝 ${description}...`);
    const child = spawn(command, args, {
      cwd: projectRoot,
      stdio: 'inherit',
      shell: isWindows,
    });

    child.on('error', reject);
    child.on('exit', (code) => {
      if (code !== 0) {
        reject(new Error(`${description} failed with code ${code}`));
      } else {
        resolve();
      }
    });
  });
}

function showHelp() {
  console.log(`
📦 oh-my-file Packaging Script
==============================

Usage: node scripts/package.js [TARGET] [OPTIONS]

Targets:
  all          Build for current platform (default)
  win32        Build for Windows (.msi, .nsis)
  darwin       Build for macOS (.dmg, .app)
  linux        Build for Linux (.appimage, .deb)

Options:
  --no-build   Skip frontend rebuild
  --help       Show this help message

Examples:
  node scripts/package.js              # Build for current platform
  node scripts/package.js win32        # Build for Windows
  node scripts/package.js linux        # Build for Linux
  node scripts/package.js all --no-build

Current Environment:
  Platform: ${platformConfigs[currentPlatform]?.name || currentPlatform}
  Architecture: ${currentArch}

Available targets for this platform:
  ${platformConfigs[currentPlatform]?.targets.join(', ') || 'Unknown'}
  `);
}

async function main() {
  const args = process.argv.slice(2);

  if (args.includes('--help')) {
    showHelp();
    process.exit(0);
  }

  const targetPlatform = args[0] === 'all' || !args[0] ? currentPlatform : args[0];
  const skipBuild = args.includes('--no-build');
  const config = platformConfigs[targetPlatform];

  if (!config) {
    console.error('❌ Invalid platform:', targetPlatform);
    console.log('\nAvailable platforms:', Object.keys(platformConfigs).join(', '));
    showHelp();
    process.exit(1);
  }

  console.log(`
╔══════════════════════════════════════╗
║   oh-my-file Packaging Tool         ║
╚══════════════════════════════════════╝

🖥️  Target: ${config.name}
📦 Formats: ${config.targets.join(', ')}
ℹ️  ${config.description}
  `);

  if (targetPlatform !== currentPlatform) {
    console.warn(`
⚠️  CROSS-COMPILATION WARNING
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
You're building for ${config.name} on ${platformConfigs[currentPlatform]?.name || currentPlatform}.
Cross-compilation may not work for some targets.

For best results, run this script on the target platform:
  - Windows: Run on Windows machine
  - macOS: Run on macOS machine
  - Linux: Run on Linux machine
    `);

    const shouldContinue = false; // In real usage, you might prompt here
    if (shouldContinue === false) {
      console.log('⏭️  Proceeding anyway...\n');
    }
  }

  try {
    // Check for required tools
    console.log('🔍 Checking dependencies...');

    if (targetPlatform === 'darwin' && currentPlatform !== 'darwin') {
      console.error('❌ Cannot build for macOS on non-macOS platform');
      console.error('   Xcode and associated tools required');
      process.exit(1);
    }

    if (targetPlatform === 'linux' && currentPlatform !== 'linux') {
      console.error('❌ Cannot build for Linux on non-Linux platform');
      console.error('   Linux-specific build tools required');
      process.exit(1);
    }

    // Build frontend if needed
    if (!skipBuild && !fs.existsSync(path.join(projectRoot, 'dist'))) {
      await runCommand(['run', 'build'], 'Building frontend');
    } else if (skipBuild) {
      console.log('⏭️  Skipping frontend build');
    } else {
      console.log('✓ Frontend already built');
    }

    // Run Tauri build
    console.log(`\n🔧 Building Tauri application for ${config.name}...`);
    await runCommand(['run', 'tauri', 'build'], `Tauri build (${config.name})`);

    // Show output directory
    const bundleDir = path.join(projectRoot, 'src-tauri', 'target', 'release', 'bundle');
    const targetDir = path.join(bundleDir, targetPlatform === 'win32' ? 'nsis' : targetPlatform === 'darwin' ? 'macos' : 'linux');

    console.log(`
╔══════════════════════════════════════╗
║        ✅ Build Complete!            ║
╚══════════════════════════════════════╝

📁 Output Location:
   ${bundleDir}

🎯 Generated Files (${config.name}):
   ${config.targets.map(t => `• ${t.toUpperCase()}`).join('\n   ')}

📦 Next Steps:
   1. Test the installers
   2. Sign the binaries (recommended for distribution)
   3. Upload to release server
    `);

  } catch (err) {
    console.error(`
╔══════════════════════════════════════╗
║        ❌ Build Failed!              ║
╚══════════════════════════════════════╝

Error: ${err.message}
    `);
    process.exit(1);
  }
}

main();
