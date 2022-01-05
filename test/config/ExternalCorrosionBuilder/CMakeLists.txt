cmake_minimum_required(VERSION 3.19)
project(ExternalCorrosionBuilder LANGUAGES C)

add_subdirectory(../../.. corrosion)

get_property(
    BUILDER_EXE_LOCATION
    TARGET Corrosion::Builder PROPERTY IMPORTED_LOCATION
)

if (NOT BUILDER_EXE_LOCATION STREQUAL CORROSION_BUILDER_EXECUTABLE)
    message(
        FATAL_ERROR
        "\
Corrosion Builder not Imported Correctly:
    Corrosion::Builder IMPORTED_LOCATION: ${BUILDER_EXE_LOCATION}
    CORROSION_BUILDER_EXECUTABLE:         ${CORROSION_BUILDER_EXECUTABLE}")
endif()
