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

    // Phase 3: Structs & Classes
    val v1 = Vector(3.0, 4.0)
    check(Math.abs(v1.magnitude() - 5.0) < 1e-6)
    check(v1.toStringRepr() == "Vector(3, 4)")
    v1.scale(2.0)
    check(Math.abs(v1.magnitude() - 10.0) < 1e-6)
    check(v1.toStringRepr() == "Vector(6, 8)")

    val v2 = Vector(1.0, 2.0)
    val dotProduct = v1.dot(v2)
    check(Math.abs(dotProduct - 22.0) < 1e-6)

    // AutoCloseable .use { ... }
    Vector(5.0, 12.0).use { v ->
        check(Math.abs(v.magnitude() - 13.0) < 1e-6)
    }

    // Stale handle protection: after close(), methods throw IllegalStateException
    v1.close()
    expectFailure<IllegalStateException> { v1.magnitude() }

    // Double close idempotency
    v1.close()

    v2.close()

    // Phase 4: Simple Enums
    check(RustApi.checkStatus(Status.Pending) == Status.Active)
    check(RustApi.checkStatus(Status.Active) == Status.Completed)
    check(RustApi.checkStatus(Status.Completed) == Status.Completed)
    check(RustApi.checkStatus(Status.Failed) == Status.Pending)
    check(Status.Pending.ordinal == 0)
    check(Status.Active.ordinal == 1)
    check(Status.Completed.ordinal == 2)
    check(Status.Failed.ordinal == 3)

    // Phase 4: Data-bearing Enums / Sealed Classes
    check(RustApi.describeShape(Shape.Circle(5.0)) == "Circle(5.0)")
    check(RustApi.describeShape(Shape.Rectangle(4.0, 6.0)) == "Rect(4.0x6.0)")
    check(RustApi.describeShape(Shape.Point) == "Point")

    val createdCircle = RustApi.makeCircle(10.5)
    check(createdCircle is Shape.Circle && Math.abs(createdCircle.radius - 10.5) < 1e-6)

    val shapes: List<Shape> = listOf(RustApi.makeCircle(10.5), Shape.Rectangle(2.0, 3.0), Shape.Point)
    val descs = shapes.map { s ->
        when (s) {
            is Shape.Circle -> "circle:${s.radius}"
            is Shape.Rectangle -> "rect:${s.width}x${s.height}"
            Shape.Point -> "point"
        }
    }
    check(descs == listOf("circle:10.5", "rect:2.0x3.0", "point"))

    // Phase 4: Data Models / Structs
    val user1 = UserProfile(101L, "babar", "babar@example.com", Status.Active)
    val echoedUser1 = RustApi.createUser(user1)
    check(echoedUser1 == user1)
    check(echoedUser1.id == 101L)
    check(echoedUser1.username == "babar")
    check(echoedUser1.email == "babar@example.com")
    check(echoedUser1.status == Status.Active)

    val user2 = UserProfile(102L, "guest", null, Status.Pending)
    val echoedUser2 = RustApi.createUser(user2)
    check(echoedUser2 == user2)
    check(echoedUser2.email == null)

    val canvas = Canvas("MyCanvas", Shape.Circle(2.5), Status.Active)
    val canvasDesc = RustApi.inspectCanvas(canvas)
    check(canvasDesc.contains("MyCanvas") && canvasDesc.contains("Circle") && canvasDesc.contains("Active"))

    val admin = RustApi.optUser(1L)
    check(admin != null && admin.username == "admin" && admin.status == Status.Active)
    check(RustApi.optUser(99L) == null)

    // Phase 4: String Arrays & Slices
    val names = arrayOf("Alice", "Bob", "Charlie", "David", "Alexander")
    val filtered = RustApi.filterNames(names, "Al")
    check(filtered.contentEquals(arrayOf("Alice", "Alexander")))
    check(RustApi.filterNames(emptyArray(), "test").isEmpty())

    // Phase 5: Callbacks - Synchronous single-method trailing lambda (SAM conversion)
    val progressEvents = mutableListOf<String>()
    RustApi.download("https://example.com/data.bin") { current, total, msg ->
        progressEvents.add("$current/$total: $msg")
    }
    check(progressEvents.size == 2)
    check(progressEvents[0] == "10/100: Downloading https://example.com/data.bin")
    check(progressEvents[1] == "100/100: Downloaded https://example.com/data.bin successfully")

    // Phase 5: Callbacks - Synchronous with return value (filter)
    val itemsToFilter = arrayOf("apple", "banana", "avocado", "cherry", "apricot")
    val keptItems = RustApi.filterStrings(itemsToFilter) { item ->
        item.startsWith("a")
    }
    check(keptItems.contentEquals(arrayOf("apple", "avocado", "apricot")))

    // Phase 5: Callbacks - Cross-thread from Rust background thread (std::thread::spawn)
    val bgProgress = java.util.concurrent.CopyOnWriteArrayList<String>()
    RustApi.runBackgroundTask { current, total, msg ->
        bgProgress.add("$current/$total: $msg")
    }
    check(bgProgress.size == 2)
    check(bgProgress[0] == "50/100: Background task working")
    check(bgProgress[1] == "100/100: Background task finished")

    // Phase 5: Callbacks - Multi-method interface implementation
    var started = false
    var completedResult: String? = null
    var errorCode = 0
    var errorMessage: String? = null

    RustApi.executeTask(object : TaskListener {
        override fun onStart() { started = true }
        override fun onComplete(result: String) { completedResult = result }
        override fun onError(code: Int, message: String) { errorCode = code; errorMessage = message }
    }, succeed = true)
    check(started && completedResult == "All operations completed successfully" && errorCode == 0)

    started = false
    completedResult = null
    RustApi.executeTask(object : TaskListener {
        override fun onStart() { started = true }
        override fun onComplete(result: String) { completedResult = result }
        override fun onError(code: Int, message: String) { errorCode = code; errorMessage = message }
    }, succeed = false)
    check(started && completedResult == null && errorCode == 404 && errorMessage == "Operation not found")

    // Phase 5: Callbacks - Optional callback parameter
    var optFired = false
    RustApi.optCallback { _, _, _ -> optFired = true }
    check(optFired)

    optFired = false
    RustApi.optCallback(null)
    check(!optFired)

    // Phase 5: Callbacks - Exception propagation & JVM recovery
    expectFailure<RuntimeException> {
        RustApi.callCallbackThatFails { _, _, _ ->
            throw IllegalArgumentException("intentional Kotlin callback failure")
        }
    }
    // Verify JVM remains healthy after callback exception
    check(RustApi.add(10, 20) == 30)

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
    println("Ketox JVM smoke tests passed: primitives, strings, nulls, Unicode, limits, panics, threads, Option, Result, ByteArrays, IntArrays, Classes, Structs, Enums, SealedClasses, Models, Collections, Callbacks, CrossThread, Exceptions, Lifecycle")
}
