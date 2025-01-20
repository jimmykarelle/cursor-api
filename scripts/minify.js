#!/usr/bin/env node

const { minify: minifyHtml } = require('html-minifier-terser');
const { minify: minifyJs } = require('terser');
const CleanCSS = require('clean-css');
const fs = require('fs');
const path = require('path');

// 配置选项
const options = {
  collapseWhitespace: true,
  removeComments: true,
  removeEmptyAttributes: true,
  removeOptionalTags: true,
  removeRedundantAttributes: true,
  removeScriptTypeAttributes: true,
  removeStyleLinkTypeAttributes: true,
  minifyCSS: true,
  minifyJS: true,
  processScripts: ['application/json'],
};

// CSS 压缩选项
const cssOptions = {
  level: 2
};

// 处理文件
async function minifyFile(inputPath, outputPath) {
  try {
    const ext = path.extname(inputPath).toLowerCase();
    const content = fs.readFileSync(inputPath, 'utf8');
    let minified;

    switch (ext) {
      case '.html':
        minified = await minifyHtml(content, options);
        break;
      case '.js':
        const result = await minifyJs(content);
        minified = result.code;
        break;
      case '.css':
        minified = new CleanCSS(cssOptions).minify(content).styles;
        break;
      default:
        throw new Error(`Unsupported file type: ${ext}`);
    }

    fs.writeFileSync(outputPath, minified);
    console.log(`✓ Minified ${path.basename(inputPath)} -> ${path.basename(outputPath)}`);
  } catch (err) {
    console.error(`✗ Error processing ${inputPath}:`, err);
    process.exit(1);
  }
}

// 主函数
async function main() {
  // 获取命令行参数，跳过前两个参数（node和脚本路径）
  const files = process.argv.slice(2);

  if (files.length === 0) {
    console.error('No input files specified');
    process.exit(1);
  }

  const staticDir = path.join(__dirname, '..', 'static');

  for (const file of files) {
    const inputPath = path.join(staticDir, file);
    const ext = path.extname(file);
    const outputPath = path.join(
      staticDir,
      file.replace(ext, `.min${ext}`)
    );
    await minifyFile(inputPath, outputPath);
  }
}

main();