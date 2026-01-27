using System;
using System.IO;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

class GenericCounter
{
    static void Main(string[] args)
    {
        if (args.Length == 0)
        {
            Console.WriteLine("Usage: Analyzer [<path-to-source.cs> ...]");
            return;
        }
		var genericTypes = 0;
		var nonGenericTypes = 0;

		var genericFunctions = 0;
		var nonGenericFunctions = 0;

		var casts = 0;

	foreach (var path in args) {
		if (!File.Exists(path))
		{
		    Console.WriteLine($"File not found: {path}");
		    return;
		}

		var code = File.ReadAllText(path);
		var tree = CSharpSyntaxTree.ParseText(code);
		var root = tree.GetCompilationUnitRoot();

		// Count generic type declarations
		genericTypes += root.DescendantNodes().OfType<TypeDeclarationSyntax>()
		    .Count(t => t.TypeParameterList != null && t.TypeParameterList.Parameters.Count > 0);
		nonGenericTypes += root.DescendantNodes().OfType<TypeDeclarationSyntax>().Count();

		// Count generic methods and local functions
		genericFunctions += root.DescendantNodes().OfType<MethodDeclarationSyntax>()
		    .Count(m => m.TypeParameterList != null && m.TypeParameterList.Parameters.Count > 0);
		genericFunctions += root.DescendantNodes().OfType<LocalFunctionStatementSyntax>()
		    .Count(f => f.TypeParameterList != null && f.TypeParameterList.Parameters.Count > 0);
		nonGenericFunctions += root.DescendantNodes().OfType<BaseMethodDeclarationSyntax>()
		    .Count();
		nonGenericFunctions += root.DescendantNodes().OfType<LocalFunctionStatementSyntax>()
		    .Count();

		casts += root.DescendantNodes().OfType<CastExpressionSyntax>().Count();
	}

	nonGenericTypes -= genericTypes;
	nonGenericFunctions -= genericFunctions;
	Console.WriteLine($"{genericTypes},{nonGenericTypes},{genericFunctions},{nonGenericFunctions},{casts}");
    }
}
