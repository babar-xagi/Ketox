package dev.ketox.example

private inline fun <reified T : Throwable> expectFailure(block: () -> Unit) {
    try {
        block()
    } catch (error: Throwable) {
        check(error is T) { "Expected ${T::class.java.simpleName}, got $error" }
        return
    }
    error("Expected ${T::class.java.simpleName}")
}

fun main() {
    check(RustApi.hello("Kotlin") == "Hello, Kotlin from Rust!")
    check(RustApi.add(20, 22) == 42)
    check(RustApi.add(Int.MAX_VALUE, 1) == Int.MIN_VALUE)
    check(RustApi.invert(false))
    check(!RustApi.invert(true))
    for (value in listOf(Byte.MIN_VALUE, 0.toByte(), Byte.MAX_VALUE)) check(RustApi.echoByte(value) == value)
    for (value in listOf(Short.MIN_VALUE, 0.toShort(), Short.MAX_VALUE)) check(RustApi.echoShort(value) == value)
    for (value in listOf(Long.MIN_VALUE, 0L, Long.MAX_VALUE)) check(RustApi.echoLong(value) == value)
    for (value in listOf(-0.0f, Float.MIN_VALUE, Float.MAX_VALUE, Float.POSITIVE_INFINITY, Float.NaN)) {
        check(RustApi.echoFloat(value).toBits() == value.toBits())
    }
    for (value in listOf(-0.0, Double.MIN_VALUE, Double.MAX_VALUE, Double.NEGATIVE_INFINITY, Double.NaN)) {
        check(RustApi.echoDouble(value).toBits() == value.toBits())
    }
    for (value in listOf("", "a\u0000b", "Kotlin 🦀 नमस्ते 中文", "x".repeat(100_000))) {
        check(RustApi.echo(value) == value)
    }
    RustApi.noop()
    expectFailure<RuntimeException> { RustApi.fail() }
    check(RustApi.add(2, 3) == 5) // JVM remains usable after the panic.
    expectFailure<IllegalArgumentException> { RustApi.echo("\uD800") }
    expectFailure<IllegalArgumentException> { RustApi.echo("\uDC00") }
    // Reflection bypasses Kotlin's non-null call-site rules, exercising JNI.
    val echo = RustApi::class.java.getDeclaredMethod("echo", String::class.java)
    try {
        echo.invoke(RustApi, null)
        error("Expected a null-input failure")
    } catch (error: java.lang.reflect.InvocationTargetException) {
        check(error.cause is NullPointerException)
    }
    // Phase 2: Option<String>
    check(RustApi.findUser(42) == "Alice")
    check(RustApi.findUser(1) == null)
    check(RustApi.greetOpt(null) == "Hello, stranger!")
    check(RustApi.greetOpt("Bob") == "Hello, Bob!")

    // Phase 2: Result<T, E>
    check(RustApi.divide(10, 2) == 5)
    expectFailure<RuntimeException> { RustApi.divide(10, 0) }

    // Phase 2: Byte Arrays & Slices
    val inputBytes = byteArrayOf(1, 2, 3, 4)
    val processedBytes = RustApi.processBytes(inputBytes)
    check(processedBytes.contentEquals(byteArrayOf((1 xor 0x5a).toByte(), (2 xor 0x5a).toByte(), (3 xor 0x5a).toByte(), (4 xor 0x5a).toByte())))

    // Phase 2: Primitive Arrays & Slices
    check(RustApi.sumNumbers(intArrayOf(10, 20, 30)) == 60L)
    check(RustApi.sumNumbers(intArrayOf()) == 0L)

    // Phase 2: Boxed Option Primitives
    check(RustApi.optAdd(10, 20) == 30)
    check(RustApi.optAdd(10, null) == 10)
    check(RustApi.optAdd(null, 20) == 20)
    check(RustApi.optAdd(null, null) == null)

    val threads = (1..4).map { worker ->
        Thread {
            repeat(100) { check(RustApi.echo("thread-$worker-🦀") == "thread-$worker-🦀") }
        }
    }
    val failures = java.util.concurrent.CopyOnWriteArrayList<Throwable>()
    threads.forEach { thread ->
        thread.uncaughtExceptionHandler = Thread.UncaughtExceptionHandler { _, error -> failures.add(error) }
        thread.start()
    }
    threads.forEach { it.join() }
    check(failures.isEmpty()) { "Concurrent JNI calls failed: $failures" }
    println(RustApi.hello("Kotlin"))
    println("Ketox JVM smoke tests passed: primitives, strings, nulls, Unicode, limits, panics, threads, Option, Result, ByteArrays, IntArrays")
}
