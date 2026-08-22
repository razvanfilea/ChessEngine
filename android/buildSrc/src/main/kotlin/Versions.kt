object Versions {
    object App {
        private const val major = 1
        private const val minor = 3
        private const val patch = 0

        const val code = major * 100 + minor * 10 + patch
        const val name = "$major.$minor.$patch"
    }

    object Sdk {
        const val min = 21
        const val wearOsMin = 25
        const val compile = 35
        const val target = 35
    }
}
