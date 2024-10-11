if (UNIX)
    set(CPACK_GENERATOR "RPM")
    set(CPACK_SOURCE_TGZ "ON")

    set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "FlatBuffers serialization library and schema compiler.")
    
    set(CPACK_RPM_PACKAGE_HOMEPAGE "https://github.com/google/flatbuffers")
    set(CPACK_RPM_PACKAGE_MAINTAINER "Marc Butler <mockbutler@gmail.com>")

    set(CPACK_PACKAGE_VERSION_MAJOR ${VERSION_MAJOR})
    set(CPACK_PACKAGE_VERSION_MINOR ${VERSION_MINOR})
    set(CPACK_PACKAGE_VERSION_PATCH ${VERSION_PATCH})
    set(CPACK_PACKAGE_VERSION "${VERSION_MAJOR}.${VERSION_MINOR}.${VERSION_PATCH}-${VERSION_COMMIT}")
    set(CPACK_RPM_PACKAGE_VERSION "${CPACK_PACKAGE_VERSION}")

    set(CPACK_RPM_PACKAGE_NAME "flatbuffers")

    # Assume this is not a cross compilation build.
    if(NOT CPACK_RPM_PACKAGE_ARCHITECTURE)
        set(CPACK_RPM_PACKAGE_ARCHITECTURE "${CMAKE_SYSTEM_PROCESSOR}")
    endif(NOT CPACK_RPM_PACKAGE_ARCHITECTURE)

    set(CPACK_RPM_PACKAGE_VENDOR "Google, Inc.")
    set(CPACK_RPM_PACKAGE_LICENSE "Apache 2.0")
    set(CPACK_RESOURCE_FILE_LICENSE ${CMAKE_SOURCE_DIR}/LICENSE)
    set(CPACK_PACKAGE_DESCRIPTION_FILE ${CMAKE_SOURCE_DIR}/CMake/DESCRIPTION.txt)

    # This may reduce rpm compatiblity with very old systems.
    set(CPACK_RPM_COMPRESSION_TYPE lzma)
    
    set(CPACK_RPM_PACKAGE_NAME "flatbuffers")
    set(CPACK_PACKAGE_FILE_NAME
        "${CPACK_RPM_PACKAGE_NAME}_${CPACK_RPM_PACKAGE_VERSION}_${CPACK_RPM_PACKAGE_ARCHITECTURE}")
    if(NOT DEFINED ${CPACK_PACKAGING_INSTALL_PREFIX})
       # Default packaging install prefix on RedHat systems is /usr.
       # This is the assumed value when this variable is not defined.
       # There is currently a conflict with
       # /usr/${CMAKE_INSTALL_LIBDIR}/cmake which is installed by default
       # by other packages on RedHat (most notably cmake-filesystem). Ensure
       # that on these systems, flatbuffers does not package this path.
       # This patch is required for cmake pre-3.17.
       list(APPEND CPACK_RPM_EXCLUDE_FROM_AUTO_FILELIST_ADDITION "/usr/${CMAKE_INSTALL_LIBDIR}/cmake")
   endif()
endif(UNIX)
