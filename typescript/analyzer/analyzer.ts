import ts from "typescript";
import fs from "fs";

function countGenerics(filePath: string) {
  const sourceCode = fs.readFileSync(filePath, "utf8");

  const sourceFile = ts.createSourceFile(
    filePath,
    sourceCode,
    ts.ScriptTarget.Latest,
    false,
  );

  let genericTypes = 0;
  let nonGenericTypes = 0;
  let genericFunctions = 0;
  let nonGenericFunctions = 0;

  function visit(node: ts.Node) {
    if (
      ts.isInterfaceDeclaration(node) ||
      ts.isClassDeclaration(node) ||
      ts.isTypeAliasDeclaration(node)
    ) {
      if (node.typeParameters && node.typeParameters.length > 0) {
        genericTypes++;
      } else {
        nonGenericTypes++;
      }
    }

    if (
      ts.isFunctionDeclaration(node) ||
      ts.isMethodDeclaration(node) ||
      ts.isConstructorDeclaration(node) ||
      ts.isArrowFunction(node) ||
      ts.isFunctionExpression(node)
    ) {
      if (node.typeParameters && node.typeParameters.length > 0) {
        genericFunctions++;
      } else {
        nonGenericFunctions++;
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);

  return {
    genericTypes,
    nonGenericTypes,
    genericFunctions,
    nonGenericFunctions,
  };
}

if (process.argv.length < 3) {
  console.error("Usage: ts-node analyzer.ts <path-to-file.ts> ...");
  process.exit(1);
}

let genericTypes = 0;
let nonGenericTypes = 0;
let genericFunctions = 0;
let nonGenericFunctions = 0;
for (let i = 2; i < process.argv.length; i++) {
  const filePath = process.argv[i];
  try {
    const result = countGenerics(filePath);
    genericTypes += result.genericTypes;
    nonGenericTypes += result.nonGenericTypes;
    genericFunctions += result.genericFunctions;
    nonGenericFunctions += result.nonGenericFunctions;
  } catch (err) {
    console.error(`unable to parse ${filePath}`);
    continue;
  }
}

console.log(`${genericTypes},${nonGenericTypes},${genericFunctions},${nonGenericFunctions}`);

