package roundhouse

// Not mapped to an existing JVM exception: a `rescue NoMethodError` would then catch that unrelated type too
open class NoMethodError(message: String? = null) : RuntimeException(message)

open class NameError(message: String? = null) : RuntimeException(message)

// Not inline `let` chains at each site: nested inside an inlined forEach they crash the JVM backend's inliner
fun <T> rhListSlice(list: List<T>, start: Long, length: Long): MutableList<T>? {
    val b = start.toInt()
    if (b < 0 || b > list.size || length < 0) return null
    return list.subList(b, minOf(b + length.toInt(), list.size)).toMutableList()
}

fun rhStrAt(s: String, index: Long): String? {
    val i = index.toInt()
    return if (i < 0 || i >= s.length) null else s.substring(i, i + 1)
}
