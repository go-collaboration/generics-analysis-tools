package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"reflect"
)

type Counter []int

func (c *Counter) Count(i int) {
	for len(*c) <= i {
		*c = append(*c, 0)
	}
	(*c)[i]++
}

func (c *Counter) String() string {
	out := "0:0"
	if len(*c) != 0 {
		out = fmt.Sprintf("%d", (*c)[0])
	}
	for i := 1; i < len(*c); i++ {
		out += fmt.Sprintf(":%d", (*c)[i])
	}
	if len(*c) == 1 {
		out += ":0"
	}
	return out
}

func isTrivialTypeBound(expr ast.Expr) bool {
	trivial := false
	switch t := expr.(type) {
	case *ast.Ident:
		trivial = t.Name == "any"
	case *ast.InterfaceType:
		trivial = t.Methods.NumFields() == 0 // interface{}
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
		trivial = false
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
	genericStructs := 0
	genericInterfaces := 0
	genericTyAliases := 0
	genericOther := 0
	nonTrivialTypeBounds := 0
	trivialTypeBounds := 0
	nonTrivialTypeBoundsStructs := 0
	trivialTypeBoundsStructs := 0
	nonTrivialTypeBoundsInterfaces := 0
	trivialTypeBoundsInterfaces := 0
	nonTrivialTypeBoundsFunctions := 0
	trivialTypeBoundsFunctions := 0
	nonTrivialTypeBoundsTyAliases := 0
	trivialTypeBoundsTyAliases := 0
	nonTrivialTypeBoundsOther := 0
	trivialTypeBoundsOther := 0
	typeAssertions := 0
	typeSwitches := 0
	typeParameterCounter := Counter([]int{})
	typeParameterCounterStructs := Counter([]int{})
	typeParameterCounterInterfaces := Counter([]int{})
	typeParameterCounterFunctions := Counter([]int{})
	typeParameterCounterTyAliases := Counter([]int{})
	typeParameterCounterOther := Counter([]int{})

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
						nonTrivialTypeBoundsFunctions += nt
						trivialTypeBoundsFunctions += t
						typeParameterCounter.Count(node.Type.TypeParams.NumFields())
						typeParameterCounterFunctions.Count(node.Type.TypeParams.NumFields())
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
					typeParameterCounter.Count(node.TypeParams.NumFields())
					if node.Assign.IsValid() {
						// Type alias
						nonTrivialTypeBoundsTyAliases += nt
						trivialTypeBoundsTyAliases += t
						genericTyAliases++
						typeParameterCounterTyAliases.Count(node.TypeParams.NumFields())
					} else {
						switch node.Type.(type) {
						case *ast.StructType:
							nonTrivialTypeBoundsStructs += nt
							trivialTypeBoundsStructs += t
							genericStructs++
							typeParameterCounterStructs.Count(node.TypeParams.NumFields())
						case *ast.InterfaceType:
							nonTrivialTypeBoundsInterfaces += nt
							trivialTypeBoundsInterfaces += t
							genericInterfaces++
							typeParameterCounterInterfaces.Count(node.TypeParams.NumFields())
						default:
							nonTrivialTypeBoundsOther += nt
							trivialTypeBoundsOther += t
							genericOther++
							typeParameterCounterOther.Count(node.TypeParams.NumFields())
						}
					}
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

	fmt.Printf("%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%s,%s,%s,%s,%s,%s\n", parseErrors, genericTy, nonGenericTy, genericStructs, genericInterfaces, genericTyAliases, genericOther, genericFun, nonGenericFun, nonTrivialTypeBounds, trivialTypeBounds, nonTrivialTypeBoundsStructs, trivialTypeBoundsStructs, nonTrivialTypeBoundsInterfaces, trivialTypeBoundsInterfaces, nonTrivialTypeBoundsFunctions, trivialTypeBoundsFunctions, nonTrivialTypeBoundsTyAliases, trivialTypeBoundsTyAliases, nonTrivialTypeBoundsOther, trivialTypeBoundsOther, typeAssertions, typeSwitches, typeParameterCounter.String(), typeParameterCounterStructs.String(), typeParameterCounterInterfaces.String(), typeParameterCounterFunctions.String(), typeParameterCounterTyAliases.String(), typeParameterCounterOther.String())
}
