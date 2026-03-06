package net.hannesbraun

import com.github.javaparser.JavaParser
import com.github.javaparser.ParserConfiguration
import com.github.javaparser.ast.body.CallableDeclaration
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration
import com.github.javaparser.ast.body.CompactConstructorDeclaration
import com.github.javaparser.ast.body.ConstructorDeclaration
import com.github.javaparser.ast.body.MethodDeclaration
import com.github.javaparser.ast.body.RecordDeclaration
import com.github.javaparser.ast.expr.CastExpr
import com.github.javaparser.ast.expr.InstanceOfExpr
import kotlin.io.path.Path
import kotlin.jvm.optionals.getOrNull

fun main(args: Array<String>) {
    var parseErrors = 0
    var genericFun = 0
    var nonGenericFun = 0
    var genericTy = 0
    var nonGenericTy = 0
    var casts = 0
    var instanceOfs = 0

    val config = ParserConfiguration().setLanguageLevel(ParserConfiguration.LanguageLevel.JAVA_21)
    val funTypes = listOf(
        CallableDeclaration::class.java,
        CompactConstructorDeclaration::class.java,
    )
    val tyTypes = listOf(
        ClassOrInterfaceDeclaration::class.java, RecordDeclaration::class.java
    )

    args.forEach { path ->
        try {
            val parser = JavaParser(config)
            val compilationUnit = parser.parse(Path(path)).result.getOrNull()

            if (compilationUnit != null) {
                for (ty in tyTypes) {
                    compilationUnit.findAll(ty).stream().forEach {
                        if (it.isGeneric) genericTy++ else nonGenericTy++
                    }
                }
                for (fn in funTypes) {
                    compilationUnit.findAll(fn).stream().forEach {
                        if (it.isGeneric) genericFun++ else nonGenericFun++
                    }
                }

                compilationUnit.findAll(CastExpr::class.java).forEach {
                    if (!it.type.isPrimitiveType) casts++
                }
                instanceOfs += compilationUnit.findAll(InstanceOfExpr::class.java).size
            } else {
                parseErrors++
                System.err.println("unable to parse $path")
            }
        } catch (_: StackOverflowError) {
            parseErrors++
            System.err.println("unable to analyze $path due to stack overflow")
        }
    }
    println("$parseErrors,$genericTy,$nonGenericTy,$genericFun,$nonGenericFun,$casts,$instanceOfs")
}
