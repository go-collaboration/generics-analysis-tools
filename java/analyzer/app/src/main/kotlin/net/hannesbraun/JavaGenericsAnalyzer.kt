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
import com.github.javaparser.ast.type.ClassOrInterfaceType
import com.github.javaparser.ast.type.TypeParameter
import kotlin.collections.first
import kotlin.io.path.Path
import kotlin.jvm.optionals.getOrNull

data class Counter(var count: MutableList<Long> = mutableListOf()) {
    fun count(i: Int) {
        while (i >= count.size) {
            count.add(0)
        }
        count[i] = count[i].plus(1)
    }

    override fun toString(): String {
        var out = "0:0"
        if (!count.isEmpty()) {
            out = count.first().toString()
            for (i in 1 until count.size) {
                out += ":${count[i]}"
            }
            if (count.size == 1) out += ":0"
        }
        return out
    }
}

data class TypeBoundCounters(
    var nonTrivialTypeBounds: Long = 0,
    var trivialTypeBounds: Long = 0,
    var nonTrivialTypeBoundsClasses: Long = 0,
    var trivialTypeBoundsClasses: Long = 0,
    var nonTrivialTypeBoundsInterfaces: Long = 0,
    var trivialTypeBoundsInterfaces: Long = 0,
    var nonTrivialTypeBoundsStaticMethods: Long = 0,
    var trivialTypeBoundsStaticMethods: Long = 0,
    var nonTrivialTypeBoundsNonStaticMethods: Long = 0,
    var trivialTypeBoundsNonStaticMethods: Long = 0
) {
    fun count(typeParameter: TypeParameter, declType: DeclType) {
        val bound = typeParameter.typeBound
        var trivial = true
        if (bound != null && !bound.isEmpty()) {
            val first = bound.first()
            if (first is ClassOrInterfaceType) {
                trivial =
                    (first.nameWithScope == "java.lang.Object" || first.nameWithScope == "Object") && bound.size == 1
            }
        }

        if (trivial) trivialTypeBounds++ else nonTrivialTypeBounds++

        when (declType) {
            DeclType.CLASS -> if (trivial) trivialTypeBoundsClasses++ else nonTrivialTypeBoundsClasses++
            DeclType.INTERFACE -> if (trivial) trivialTypeBoundsInterfaces++ else nonTrivialTypeBoundsInterfaces++
            DeclType.STATIC_METHOD -> if (trivial) trivialTypeBoundsStaticMethods++ else nonTrivialTypeBoundsStaticMethods++
            DeclType.NON_STATIC_METHOD -> if (trivial) trivialTypeBoundsNonStaticMethods++ else nonTrivialTypeBoundsNonStaticMethods++
        }
    }
}

enum class DeclType {
    CLASS, INTERFACE, STATIC_METHOD, NON_STATIC_METHOD
}


fun main(args: Array<String>) {
    var parseErrors = 0
    var genericFun = 0
    var nonGenericFun = 0
    var genericTy = 0
    var nonGenericTy = 0
    var genericClasses = 0
    var genericInterfaces = 0
    var genericNonStaticMethods = 0
    var genericStaticMethods = 0
    val typeParameterCounter = Counter()
    val typeParameterCounterClasses = Counter()
    val typeParameterCounterInterfaces = Counter()
    val typeParameterCounterStaticMethods = Counter()
    val typeParameterCounterNonStaticMethods = Counter()
    val typeBoundCounters = TypeBoundCounters()

    var casts = 0
    var instanceOfs = 0

    val config = ParserConfiguration().setLanguageLevel(ParserConfiguration.LanguageLevel.JAVA_25)
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
                        if (it.isGeneric) {
                            genericTy++
                            typeParameterCounter.count(it.typeParameters.size)
                            if (it is ClassOrInterfaceDeclaration && it.isInterface) {
                                genericInterfaces++
                                typeParameterCounterInterfaces.count(it.typeParameters.size)
                                it.typeParameters.forEach { typeParameter ->
                                    typeBoundCounters.count(typeParameter, DeclType.INTERFACE)
                                }
                            } else {
                                genericClasses++
                                typeParameterCounterClasses.count(it.typeParameters.size)
                                it.typeParameters.forEach { typeParameter ->
                                    typeBoundCounters.count(typeParameter, DeclType.CLASS)
                                }
                            }


                        } else {
                            nonGenericTy++
                        }
                    }
                }
                for (fn in funTypes) {
                    compilationUnit.findAll(fn).stream().forEach {
                        if (it.isGeneric) {
                            genericFun++
                            typeParameterCounter.count(it.typeParameters.size)
                            typeParameterCounter.count(it.typeParameters.size)
                            if (it is MethodDeclaration) {
                                if (it.isStatic) {
                                    genericStaticMethods++
                                    typeParameterCounterStaticMethods.count(it.typeParameters.size)
                                    it.typeParameters.forEach { typeParameter ->
                                        typeBoundCounters.count(typeParameter, DeclType.STATIC_METHOD)
                                    }
                                } else {
                                    genericNonStaticMethods++
                                    typeParameterCounterNonStaticMethods.count(it.typeParameters.size)
                                    it.typeParameters.forEach { typeParameter ->
                                        typeBoundCounters.count(typeParameter, DeclType.NON_STATIC_METHOD)
                                    }
                                }
                            } else {
                                // Constructor
                                assert(it is ConstructorDeclaration || it is CompactConstructorDeclaration)
                                genericNonStaticMethods++
                                typeParameterCounterNonStaticMethods.count(it.typeParameters.size)
                                it.typeParameters.forEach { typeParameter ->
                                    typeBoundCounters.count(typeParameter, DeclType.NON_STATIC_METHOD)
                                }
                            }
                        } else {
                            nonGenericFun++
                        }
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
    println("$parseErrors,$genericTy,$nonGenericTy,$genericClasses,$genericInterfaces,$genericFun,$nonGenericFun,$genericNonStaticMethods,$genericStaticMethods,$casts,$instanceOfs,$typeParameterCounter,$typeParameterCounterClasses,$typeParameterCounterInterfaces,$typeParameterCounterNonStaticMethods,$typeParameterCounterStaticMethods,${typeBoundCounters.nonTrivialTypeBounds},${typeBoundCounters.trivialTypeBounds},${typeBoundCounters.nonTrivialTypeBoundsClasses},${typeBoundCounters.trivialTypeBoundsClasses},${typeBoundCounters.nonTrivialTypeBoundsInterfaces},${typeBoundCounters.trivialTypeBoundsInterfaces},${typeBoundCounters.nonTrivialTypeBoundsNonStaticMethods},${typeBoundCounters.trivialTypeBoundsNonStaticMethods},${typeBoundCounters.nonTrivialTypeBoundsStaticMethods},${typeBoundCounters.trivialTypeBoundsStaticMethods}")
}
