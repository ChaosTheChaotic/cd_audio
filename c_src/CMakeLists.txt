project(cd C)

find_package(PkgConfig REQUIRED)
pkg_check_modules(LIBCDIO REQUIRED libcdio)
pkg_check_modules(LIBCDIO_CDDA REQUIRED libcdio_cdda)

include_directories(${LIBCDIO_INCLUDE_DIRS} ${LIBCDIO_CDDA_INCLUDE_DIRS})

add_library(cd STATIC cd.c)
target_link_libraries(cd ${LIBCDIO_LIBRARIES} ${LIBCDIO_CDDA_LIBRARIES})

set_target_properties(cd PROPERTIES
    ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
)

install(TARGETS cd
    ARCHIVE DESTINATION ${CMAKE_BINARY_DIR}/lib
)
