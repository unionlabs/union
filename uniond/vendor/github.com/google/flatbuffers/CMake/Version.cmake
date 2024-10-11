set(VERSION_MAJOR 24)
set(VERSION_MINOR 3)
set(VERSION_PATCH 25)
set(VERSION_COMMIT 0)

if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/.git")
  find_program(GIT git)
  if(GIT)
    execute_process(
      COMMAND ${GIT} describe --tags
      WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
      OUTPUT_VARIABLE GIT_DESCRIBE_DIRTY
      OUTPUT_STRIP_TRAILING_WHITESPACE
      RESULT_VARIABLE GIT_DESCRIBE_RESULT
    )

    if(GIT_DESCRIBE_RESULT EQUAL 0)
      # Test if the most recent Git tag matches the pattern "v<major>.<minor>.<patch>*"
      if(GIT_DESCRIBE_DIRTY MATCHES "^v[0-9]+\\.[0-9]+\\.[0-9]+.*")
        string(REGEX REPLACE "^v([0-9]+)\\..*" "\\1" VERSION_MAJOR "${GIT_DESCRIBE_DIRTY}")
        string(REGEX REPLACE "^v[0-9]+\\.([0-9]+).*" "\\1" VERSION_MINOR "${GIT_DESCRIBE_DIRTY}")
        string(REGEX REPLACE "^v[0-9]+\\.[0-9]+\\.([0-9]+).*" "\\1" VERSION_PATCH "${GIT_DESCRIBE_DIRTY}")
        string(REGEX REPLACE "^v[0-9]+\\.[0-9]+\\.[0-9]+\\-([0-9]+).*" "\\1" VERSION_COMMIT "${GIT_DESCRIBE_DIRTY}")
        # If the tag points to the commit, then only the tag is shown in "git describe"
        if(VERSION_COMMIT STREQUAL GIT_DESCRIBE_DIRTY)
          set(VERSION_COMMIT 0)
        endif()
      else()
        message(WARNING "\"${GIT_DESCRIBE_DIRTY}\" does not match pattern v<major>.<minor>.<patch>-<commit>")
      endif()
    else()
      message(WARNING "git describe failed with exit code: ${GIT_DESCRIBE_RESULT}\nMake sure you cloned with tags or run 'git fetch --tags'.")
    endif()
  else()
    message(WARNING "git is not found")
  endif()
endif()

message(STATUS "Proceeding with version: ${VERSION_MAJOR}.${VERSION_MINOR}.${VERSION_PATCH}.${VERSION_COMMIT}")
