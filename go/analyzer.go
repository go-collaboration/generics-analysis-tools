package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
)

func main() {
	genericTy := 0
	nonGenericTy := 0
	genericFun := 0
	nonGenericFun := 0
	typeAssertions := 0
	typeSwitches := 0

	for _, path := range os.Args[1:] {
		src, _ := os.ReadFile(path)
		fset := token.NewFileSet()
		file, err := parser.ParseFile(fset, "", src, parser.AllErrors)
		if err != nil {
			fmt.Fprintln(os.Stderr, path)
			fmt.Fprintln(os.Stderr, err)
			continue
		}

		ast.Inspect(file, func(n ast.Node) bool {
			switch node := n.(type) {
			case *ast.FuncDecl:
				if node.Recv == nil {
					if node.Type.TypeParams != nil && len(node.Type.TypeParams.List) > 0 {
						genericFun++
					} else {
						nonGenericFun++
					}
				} else {
					if node.Recv.List != nil {
						receiverType := node.Recv.List[0].Type
						if starExpr, ok := receiverType.(*ast.StarExpr); ok {
							receiverType = starExpr.X
						}
						if _, ok := receiverType.(*ast.IndexExpr); ok {
							genericFun++
						} else if _, ok := receiverType.(*ast.IndexListExpr); ok {
							genericFun++
						} else {
							nonGenericFun++
						}
					} else {
						nonGenericFun++
					}
				}

			case *ast.TypeSpec:
				if node.TypeParams != nil && len(node.TypeParams.List) > 0 {
					genericTy++
				} else {
					nonGenericTy++
				}
			case *ast.TypeAssertExpr:
				typeAssertions++
			case *ast.TypeSwitchStmt:
				typeSwitches++
			}
			return true
		})
	}

	fmt.Printf("%d,%d,%d,%d,%d,%d\n", genericTy, nonGenericTy, genericFun, nonGenericFun, typeAssertions, typeSwitches)
}
