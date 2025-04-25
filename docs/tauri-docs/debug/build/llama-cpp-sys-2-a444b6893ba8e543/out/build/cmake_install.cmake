# Install script for directory: C:/Users/AntonMoryakov/.cargo/git/checkouts/llama-cpp-rs-8f51a6be7089e2e6/aa257c0/llama-cpp-sys-2/llama.cpp

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "Release")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/ggml/cmake_install.cmake")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/src/cmake_install.cmake")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/common/cmake_install.cmake")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  if(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Dd][Ee][Bb][Uu][Gg])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/src/Debug/llama.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ee][Aa][Ss][Ee])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/src/Release/llama.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Mm][Ii][Nn][Ss][Ii][Zz][Ee][Rr][Ee][Ll])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/src/MinSizeRel/llama.lib")
  elseif(CMAKE_INSTALL_CONFIG_NAME MATCHES "^([Rr][Ee][Ll][Ww][Ii][Tt][Hh][Dd][Ee][Bb][Ii][Nn][Ff][Oo])$")
    file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib" TYPE STATIC_LIBRARY FILES "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/src/RelWithDebInfo/llama.lib")
  endif()
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE FILE FILES
    "C:/Users/AntonMoryakov/.cargo/git/checkouts/llama-cpp-rs-8f51a6be7089e2e6/aa257c0/llama-cpp-sys-2/llama.cpp/include/llama.h"
    "C:/Users/AntonMoryakov/.cargo/git/checkouts/llama-cpp-rs-8f51a6be7089e2e6/aa257c0/llama-cpp-sys-2/llama.cpp/include/llama-cpp.h"
    )
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/cmake/llama" TYPE FILE FILES
    "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/llama-config.cmake"
    "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/llama-version.cmake"
    )
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/bin" TYPE FILE PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE GROUP_READ GROUP_EXECUTE WORLD_READ WORLD_EXECUTE FILES "C:/Users/AntonMoryakov/.cargo/git/checkouts/llama-cpp-rs-8f51a6be7089e2e6/aa257c0/llama-cpp-sys-2/llama.cpp/convert_hf_to_gguf.py")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib/pkgconfig" TYPE FILE FILES "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/llama.pc")
endif()

if(CMAKE_INSTALL_COMPONENT)
  set(CMAKE_INSTALL_MANIFEST "install_manifest_${CMAKE_INSTALL_COMPONENT}.txt")
else()
  set(CMAKE_INSTALL_MANIFEST "install_manifest.txt")
endif()

string(REPLACE ";" "\n" CMAKE_INSTALL_MANIFEST_CONTENT
       "${CMAKE_INSTALL_MANIFEST_FILES}")
file(WRITE "C:/Users/AntonMoryakov/project/simple-local-llm/docs/tauri-docs/debug/build/llama-cpp-sys-2-a444b6893ba8e543/out/build/${CMAKE_INSTALL_MANIFEST}"
     "${CMAKE_INSTALL_MANIFEST_CONTENT}")
