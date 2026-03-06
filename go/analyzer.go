package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"reflect"
)

func isTrivialTypeBound(expr ast.Expr) bool {
	trivial := false
	switch t := expr.(type) {
	case *ast.Ident:
		trivial = t.Name == "any"
	case *ast.InterfaceType:
		trivial = t.Methods.NumFields() == 0
	case *ast.BinaryExpr:
		if t.Op != token.OR {
			fmt.Fprintf(os.Stderr, "type bound analysis may be wrong for: %v\n", expr)
		}
		trivial = false
	case *ast.SelectorExpr:
		trivial = false
	case *ast.UnaryExpr:
		if t.Op != token.TILDE {
			fmt.Fprintf(os.Stderr, "type bound analysis may be wrong for: %v\n", expr)
		}
		trivial = isTrivialTypeBound(t.X)
	case *ast.ArrayType:
		trivial = false
	case *ast.MapType:
		trivial = false
	case *ast.FuncType:
		trivial = false
	case *ast.StarExpr:
		// trivial = isTrivialTypeBound(t.X)
		trivial = false
	case *ast.IndexExpr:
		trivial = false
	case *ast.IndexListExpr:
		trivial = false
	case *ast.StructType:
		trivial = false
	case *ast.ChanType:
		trivial = false
	default:
		fmt.Fprintf(os.Stderr, "unable to analyze type bound specification of type %v: %v\n", reflect.TypeOf(expr), expr)
	}
	return trivial
}

func countTrivialTypeBounds(fieldList *ast.FieldList) (int, int) {
	nonTrivialTypeBounds := 0
	trivialTypeBounds := 0
	for _, typeParameter := range fieldList.List {
		if isTrivialTypeBound(typeParameter.Type) {
			trivialTypeBounds++
		} else {
			nonTrivialTypeBounds++
		}
	}
	return nonTrivialTypeBounds, trivialTypeBounds
}

func main() {
	parseErrors := 0
	genericTy := 0
	nonGenericTy := 0
	genericFun := 0
	nonGenericFun := 0
	nonTrivialTypeBounds := 0
	trivialTypeBounds := 0
	typeAssertions := 0
	typeSwitches := 0

	for _, path := range os.Args[1:] {
		src, _ := os.ReadFile(path)
		fset := token.NewFileSet()
		file, err := parser.ParseFile(fset, "", src, parser.AllErrors)
		if err != nil {
			parseErrors++
			fmt.Fprintln(os.Stderr, path)
			fmt.Fprintln(os.Stderr, err)
			continue
		}

		ast.Inspect(file, func(n ast.Node) bool {
			switch node := n.(type) {
			case *ast.FuncDecl:
				if node.Recv == nil {
					// Only functions are relevant
					if node.Type.TypeParams != nil && len(node.Type.TypeParams.List) > 0 {
						genericFun++
						nt, t := countTrivialTypeBounds(node.Type.TypeParams)
						nonTrivialTypeBounds += nt
						trivialTypeBounds += t
					} else {
						nonGenericFun++
					}
				}
			case *ast.TypeSpec:
				if node.TypeParams != nil && len(node.TypeParams.List) > 0 {
					genericTy++
					nt, t := countTrivialTypeBounds(node.TypeParams)
					nonTrivialTypeBounds += nt
					trivialTypeBounds += t
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

	fmt.Printf("%d,%d,%d,%d,%d,%d,%d,%d,%d\n", parseErrors, genericTy, nonGenericTy, genericFun, nonGenericFun, nonTrivialTypeBounds, trivialTypeBounds, typeAssertions, typeSwitches)
}
