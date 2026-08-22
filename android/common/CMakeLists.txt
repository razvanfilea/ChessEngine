cmake_minimum_required(VERSION 3.20)
set(PROJECT_NAME "chess")
project(${PROJECT_NAME})

add_subdirectory(src/main/cpp/chess)

add_library(
        ${PROJECT_NAME}
        SHARED

        src/main/cpp/Main.cpp
        ${SOURCE_FILES}
)

target_compile_features(${PROJECT_NAME} PUBLIC ${COMPILE_FEATURES})

target_compile_options(${PROJECT_NAME} PUBLIC
        -stdlib=libc++
        ${FLAGS_COMPILE}
        )

set(FLAGS_RELWITHDEBINFO_COMPILE ${FLAGS_RELWITHDEBINFO_COMPILE} "-mllvm" "-polly")
set(FLAGS_RELEASE_COMPILE ${FLAGS_RELEASE_COMPILE} "-mllvm" "-polly")

target_compile_options(${PROJECT_NAME} PUBLIC "$<$<CONFIG:DEBUG>:${FLAGS_DEBUG_COMPILE}>")
target_compile_options(${PROJECT_NAME} PUBLIC "$<$<CONFIG:RELWITHDEBINFO>:${FLAGS_RELWITHDEBINFO_COMPILE}>")
target_compile_options(${PROJECT_NAME} PUBLIC "$<$<CONFIG:RELEASE>:${FLAGS_RELEASE_COMPILE}>")

find_library(
        log-lib
        log
)

target_link_libraries(
        ${PROJECT_NAME}
        ${log-lib}
)

