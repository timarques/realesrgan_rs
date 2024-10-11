cmake_minimum_required(VERSION 3.25)
project(realesrgan-wrapper)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Threads)
find_package(OpenMP)
find_package(Vulkan REQUIRED)
find_package(ncnn REQUIRED)

set(CMAKE_BUILD_TYPE Release)
set(CMAKE_CXX_FLAGS "-O3 ${CMAKE_CXX_FLAGS}")
set(CMAKE_CXX_STANDARD 11)

include(CheckIPOSupported)
check_ipo_supported(RESULT ipo_supported)
if(ipo_supported)
    set(CMAKE_POLICY_DEFAULT_CMP0069 NEW)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
else()
    message(WARNING "IPO is not supported")
endif()

if(OPENMP_FOUND)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${OpenMP_C_FLAGS}")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${OpenMP_CXX_FLAGS}")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} ${OpenMP_EXE_LINKER_FLAGS}")
endif()

macro(realesrgan_add_shader SHADER_SRC)
    set(SHADER_SRC_FULLPATH ${CMAKE_CURRENT_SOURCE_DIR}/shaders/${SHADER_SRC})

    get_filename_component(SHADER_SRC_NAME_WE ${CMAKE_CURRENT_SOURCE_DIR}/shaders/${SHADER_SRC} NAME_WE)
    set(SHADER_SPV_HEX_FILE ${CMAKE_CURRENT_BINARY_DIR}/${SHADER_SRC_NAME_WE}.spv.hex.h)
    add_custom_command(
        OUTPUT ${SHADER_SPV_HEX_FILE}
        COMMAND Vulkan::glslangValidator
        ARGS -V -s -x -o ${SHADER_SPV_HEX_FILE} ${SHADER_SRC_FULLPATH}
        DEPENDS ${SHADER_SRC_FULLPATH}
        COMMENT "Building SPIR-V module ${SHADER_SRC_NAME_WE}.spv"
        VERBATIM
    )
    set_source_files_properties(${SHADER_SPV_HEX_FILE} PROPERTIES GENERATED TRUE)
    list(APPEND SHADER_SPV_HEX_FILES ${SHADER_SPV_HEX_FILE})

    # fp16 storage
    set(SHADER_fp16s_SRC_NAME_WE "${SHADER_SRC_NAME_WE}_fp16s")

    set(SHADER_fp16s_SPV_HEX_FILE ${CMAKE_CURRENT_BINARY_DIR}/${SHADER_fp16s_SRC_NAME_WE}.spv.hex.h)
    add_custom_command(
        OUTPUT ${SHADER_fp16s_SPV_HEX_FILE}
        COMMAND Vulkan::glslangValidator
        ARGS -DNCNN_fp16_storage=1 -V -s -x -o ${SHADER_fp16s_SPV_HEX_FILE} ${SHADER_SRC_FULLPATH}
        DEPENDS ${SHADER_SRC_FULLPATH}
        COMMENT "Building SPIR-V module ${SHADER_fp16s_SRC_NAME_WE}.spv"
        VERBATIM
    )
    set_source_files_properties(${SHADER_fp16s_SPV_HEX_FILE} PROPERTIES GENERATED TRUE)
    list(APPEND SHADER_SPV_HEX_FILES ${SHADER_fp16s_SPV_HEX_FILE})

    # int8 storage
    set(SHADER_int8s_SRC_NAME_WE "${SHADER_SRC_NAME_WE}_int8s")

    set(SHADER_int8s_SPV_HEX_FILE ${CMAKE_CURRENT_BINARY_DIR}/${SHADER_int8s_SRC_NAME_WE}.spv.hex.h)
    add_custom_command(
        OUTPUT ${SHADER_int8s_SPV_HEX_FILE}
        COMMAND Vulkan::glslangValidator
        ARGS -DNCNN_fp16_storage=1 -DNCNN_int8_storage=1 -V -s -x -o ${SHADER_int8s_SPV_HEX_FILE} ${SHADER_SRC_FULLPATH}
        DEPENDS ${SHADER_SRC_FULLPATH}
        COMMENT "Building SPIR-V module ${SHADER_int8s_SRC_NAME_WE}.spv"
        VERBATIM
    )
    set_source_files_properties(${SHADER_int8s_SPV_HEX_FILE} PROPERTIES GENERATED TRUE)
    list(APPEND SHADER_SPV_HEX_FILES ${SHADER_int8s_SPV_HEX_FILE})
endmacro()

set(SHADER_SPV_HEX_FILES)

realesrgan_add_shader(realesrgan_preproc.comp)
realesrgan_add_shader(realesrgan_postproc.comp)
realesrgan_add_shader(realesrgan_preproc_tta.comp)
realesrgan_add_shader(realesrgan_postproc_tta.comp)

add_custom_target(generate-spirv DEPENDS ${SHADER_SPV_HEX_FILES})

include_directories(${CMAKE_SOURCE_DIR}/cpp)
include_directories(${CMAKE_CURRENT_BINARY_DIR})

set(REALESRGAN_LIBS ncnn ${Vulkan_LIBRARY})
if(OPENMP_FOUND)
    list(APPEND REALESRGAN_LIBS ${OpenMP_CXX_LIBRARIES})
endif()

add_library(realesrgan-wrapper STATIC ${CMAKE_SOURCE_DIR}/cpp/wrapper.cpp ${CMAKE_SOURCE_DIR}/cpp/realesrgan.cpp)
add_dependencies(realesrgan-wrapper generate-spirv)
target_link_libraries(realesrgan-wrapper ${REALESRGAN_LIBS})
install(TARGETS realesrgan-wrapper ARCHIVE DESTINATION lib)