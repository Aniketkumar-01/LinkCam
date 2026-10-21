cmake_minimum_required(VERSION 3.20)
project(LinkCamDShow LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(LinkCamDShow64 SHARED
    LinkCamDShow.cpp
    LinkCamDShow.def
)

target_link_libraries(LinkCamDShow64 PRIVATE
    strmiids
    ole32
    oleaut32
    user32
    advapi32
    mfplat
    mfuuid
)

set_target_properties(LinkCamDShow64 PROPERTIES
    OUTPUT_NAME "LinkCamDShow64"
    PREFIX ""
)
