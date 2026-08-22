# Flags
set(CMAKE_CXX_EXTENSIONS OFF PARENT_SCOPE)

set(COMPILE_FEATURES "cxx_std_20" PARENT_SCOPE)

set(FLAGS_COMPILE
        -fno-exceptions -fno-rtti
        -Wall -Wextra -Wshadow -Wnon-virtual-dtor -Wold-style-cast -Wunused -Woverloaded-virtual
        -Wpedantic -Wnull-dereference
        PARENT_SCOPE
        )

set(FLAGS_DEBUG_COMPILE PARENT_SCOPE)
set(FLAGS_RELWITHDEBINFO_COMPILE "-UNDEBUG" "-O3" "-flto" PARENT_SCOPE)
set(FLAGS_RELEASE_COMPILE "-DNDEBUG" "-O3" "-flto" PARENT_SCOPE)

set(FLAGS_DEBUG_LINK "-fsanitize=undefined" PARENT_SCOPE)
set(FLAGS_RELEASE_LINK PARENT_SCOPE)

# Files
set(ROOT ${CMAKE_CURRENT_SOURCE_DIR})
set(SOURCE_FILES
        ${SOURCE_FILES}
        ${ROOT}/BoardManager.cpp
        ${ROOT}/Stats.cpp
        ${ROOT}/Uci.cpp
        ${ROOT}/Board.cpp
        ${ROOT}/algorithm/Attacks.cpp
        ${ROOT}/algorithm/Evaluation.cpp
        ${ROOT}/MoveGen.cpp
        ${ROOT}/MoveOrdering.cpp
        ${ROOT}/PawnStructureTable.cpp
        ${ROOT}/algorithm/Search.cpp
        ${ROOT}/Tests.cpp
        ${ROOT}/TranspositionTable.cpp
        ${ROOT}/Zobrist.cpp
        ${ROOT}/persistence/FenParser.cpp
        ${ROOT}/polyglot/PolyBook.cpp
        PARENT_SCOPE
        )
