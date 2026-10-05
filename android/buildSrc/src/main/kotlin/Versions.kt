object Versions {
    object App {
        private const val major = 2
        private const val minor = 0
        private const val patch = 0

        const val code = major * 100 + minor * 10 + patch
        const val name = "$major.$minor.$patch"
    }

    object Sdk {
        const val min = 23
        const val wearOsMin = 25
        const val compile = 37
        const val target = 36
    }
}
