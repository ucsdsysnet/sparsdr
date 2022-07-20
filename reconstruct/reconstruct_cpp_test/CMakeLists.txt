cmake_minimum_required(VERSION 3.0)

add_executable(reconstruct_cpp_test main.cpp)
target_link_libraries(reconstruct_cpp_test PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../target/release/libsparsdr_reconstruct.a)
# Other libraries that sparsdr_reconstruct depends on
find_package(Threads REQUIRED)
target_link_libraries(reconstruct_cpp_test PRIVATE Threads::Threads rt dl fftw3 fftw3f)

target_include_directories(reconstruct_cpp_test PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../target/release)
