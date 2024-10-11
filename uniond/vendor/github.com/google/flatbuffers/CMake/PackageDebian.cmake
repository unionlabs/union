# ------------------- Debianization ---------------------
if (UNIX)

    # Set build environment
    SET(CPACK_GENERATOR "TGZ;DEB")
    SET(CPACK_SOURCE_TGZ "ON")

    # Common package information
    SET(CPACK_PACKAGE_DESCRIPTION_SUMMARY
        "FlatBuffers is an efficient cross platform serialization library for C++, with support for Java, C# and Go. It was created at Google specifically for game development and other performance-critical applications.")
    SET(CPACK_DEBIAN_PACKAGE_HOMEPAGE "https://github.com/google/flatbuffers")
    SET(CPACK_DEBIAN_PACKAGE_MAINTAINER "Vitaly Isaev <vitalyisaev2@gmail.com>")

    SET(CPACK_PACKAGE_VERSION_MAJOR ${VERSION_MAJOR})
    SET(CPACK_PACKAGE_VERSION_MINOR ${VERSION_MINOR})
    SET(CPACK_PACKAGE_VERSION_PATCH ${VERSION_PATCH})
    SET(CPACK_PACKAGE_VERSION "${VERSION_MAJOR}.${VERSION_MINOR}.${VERSION_PATCH}-${VERSION_COMMIT}")
    SET(CPACK_DEBIAN_PACKAGE_VERSION "${CPACK_PACKAGE_VERSION}")

    # Package name
    SET(CPACK_DEBIAN_PACKAGE_NAME "flatbuffers")
    SET(CPACK_RESOURCE_FILE_LICENSE ${CMAKE_SOURCE_DIR}/LICENSE)
    set(CPACK_DEBIAN_FILE_NAME "DEB-DEFAULT")

endif(UNIX)
