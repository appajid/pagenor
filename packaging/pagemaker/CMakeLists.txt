# Build the separately licensed PageMaker process; never link it into Pagenor Rust.
cmake_minimum_required(VERSION 3.20)
cmake_policy(SET CMP0091 NEW)
project(pagenor_pagemaker_helper LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
if(MSVC)
  add_compile_options(/J /utf-8 /bigobj)
  add_compile_definitions(_USE_MATH_DEFINES _CRT_SECURE_NO_WARNINGS NOMINMAX)
else()
  add_compile_options(-funsigned-char)
endif()
set(rv "${HELPER_SOURCES}/librevenge-0.0.6")
set(pm "${HELPER_SOURCES}/libpagemaker-0.0.4")
set(ZLIB_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
add_subdirectory("${HELPER_SOURCES}/zlib-1.3.1" "${CMAKE_BINARY_DIR}/zlib" EXCLUDE_FROM_ALL)
add_library(revenge STATIC
  "${rv}/src/lib/RVNGBinaryData.cpp" "${rv}/src/lib/RVNGMemoryStream.cpp"
  "${rv}/src/lib/RVNGProperty.cpp" "${rv}/src/lib/RVNGPropertyList.cpp"
  "${rv}/src/lib/RVNGPropertyListVector.cpp" "${rv}/src/lib/RVNGString.cpp"
  "${rv}/src/lib/RVNGStringVector.cpp" "${rv}/src/lib/RVNGSVGDrawingGenerator.cpp"
  "${rv}/src/lib/RVNGRawGeneratorBase.cpp" "${rv}/src/lib/RVNGRawDrawingGenerator.cpp"
  "${rv}/src/lib/RVNGStreamImplementation.cpp"
  "${rv}/src/lib/RVNGOLEStream.cpp" "${rv}/src/lib/RVNGZipStream.cpp")
target_include_directories(revenge PUBLIC "${rv}/inc" "${BOOST_INCLUDE_DIR}")
target_link_libraries(revenge PRIVATE zlibstatic)
target_compile_definitions(revenge PRIVATE BOOST_ERROR_CODE_HEADER_ONLY)
add_library(pagemaker STATIC
  "${pm}/src/lib/OutputShape.cpp" "${pm}/src/lib/PMDCollector.cpp"
  "${pm}/src/lib/PMDParser.cpp" "${pm}/src/lib/PMDTypes.cpp"
  "${pm}/src/lib/PMDocument.cpp" "${pm}/src/lib/geometry.cpp"
  "${pm}/src/lib/libpagemaker_utils.cpp")
target_include_directories(pagemaker PUBLIC "${pm}/inc")
target_link_libraries(pagemaker PUBLIC revenge)
add_executable(pmd2raw-byte-v2 "${pm}/src/conv/raw/pmd2raw.cpp")
target_compile_definitions(pmd2raw-byte-v2 PRIVATE VERSION="0.0.4-pagenor-byte-v2")
target_link_libraries(pmd2raw-byte-v2 PRIVATE pagemaker)
install(TARGETS pmd2raw-byte-v2 RUNTIME DESTINATION helpers)
